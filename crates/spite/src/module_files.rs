//! Explicit filesystem paths for the command-line Module host.

use spite::{DiagnosticKind, Error, ExceptionKind, ModuleHandle, Realm, Span, ast::ModuleRequest};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

struct Node {
    record: ModuleHandle,
    directory: PathBuf,
    dependencies: Vec<(ModuleRequest, ModuleHandle)>,
    expanded: bool,
}

pub(super) struct FileModules {
    nodes: Vec<Node>,
    files: HashMap<PathBuf, usize>,
    records: HashMap<ModuleHandle, usize>,
}

fn host(message: String) -> Error {
    Error::Host {
        span: Span::new(0, 0),
        message,
    }
}

fn capacity() -> Error {
    Error::Limit {
        span: Span::new(0, 0),
        message: "module file loader capacity exceeded".into(),
    }
}

fn canonical(path: &Path) -> Result<PathBuf, Error> {
    fs::canonicalize(path)
        .map_err(|error| host(format!("cannot resolve Module {}: {error}", path.display())))
}

impl FileModules {
    pub(super) fn load(
        realm: &mut Realm,
        source: &str,
        filename: Option<&Path>,
    ) -> Result<(Self, ModuleHandle), Error> {
        let filename = filename.map(canonical).transpose()?;
        let directory = match &filename {
            Some(path) => path.parent().expect("canonical file parent").to_path_buf(),
            None => PathBuf::from("."),
        };
        let root = realm.create_module(source)?;
        let mut loader = Self {
            nodes: Vec::new(),
            files: HashMap::new(),
            records: HashMap::new(),
        };
        loader.insert(root, directory, filename)?;
        let root = loader.publish_graph(realm, 0)?;
        Ok((loader, root))
    }

    fn insert(
        &mut self,
        record: ModuleHandle,
        directory: PathBuf,
        filename: Option<PathBuf>,
    ) -> Result<usize, Error> {
        self.nodes.try_reserve(1).map_err(|_| capacity())?;
        self.records.try_reserve(1).map_err(|_| capacity())?;
        if filename.is_some() {
            self.files.try_reserve(1).map_err(|_| capacity())?;
        }
        let index = self.nodes.len();
        self.records.insert(record.clone(), index);
        if let Some(filename) = filename {
            self.files.insert(filename, index);
        }
        self.nodes.push(Node {
            record,
            directory,
            dependencies: Vec::new(),
            expanded: false,
        });
        Ok(index)
    }

    fn resolve(
        &mut self,
        realm: &mut Realm,
        directory: &Path,
        request: &ModuleRequest,
    ) -> Result<usize, Error> {
        // The host supports no import-attribute keys. Static requests must reject
        // before file access; dynamic outer requests are already checked by Runtime.
        if !request.attributes.is_empty() {
            return Err(Error::Exception {
                kind: ExceptionKind::SyntaxError,
                span: Span::new(0, 0),
                message: "unsupported import attribute key".into(),
            });
        }
        let specifier = String::from_utf16(request.specifier.code_units())
            .map_err(|_| host("Module file paths require valid Unicode".into()))?;
        let path = canonical(&directory.join(specifier))?;
        if let Some(&index) = self.files.get(&path) {
            return Ok(index);
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| host(format!("cannot read Module {}: {error}", path.display())))?;
        let record = realm.create_module(&source)?;
        let directory = path.parent().expect("canonical file parent").to_path_buf();
        self.insert(record, directory, Some(path))
    }

    fn publish_graph(&mut self, realm: &mut Realm, root: usize) -> Result<ModuleHandle, Error> {
        // Expand only this root: a failed earlier graph must not block an unrelated
        // import. Publish each dependency list only after its preparation succeeds.
        let mut pending = Vec::new();
        pending.try_reserve(1).map_err(|_| capacity())?;
        pending.push(root);
        let mut seen = HashSet::new();
        while let Some(index) = pending.pop() {
            if seen.contains(&index) {
                continue;
            }
            seen.try_reserve(1).map_err(|_| capacity())?;
            seen.insert(index);
            if !self.nodes[index].expanded {
                let requests = realm.module_requests(&self.nodes[index].record)?;
                let directory = self.nodes[index].directory.clone();
                let mut dependencies = Vec::new();
                dependencies
                    .try_reserve_exact(requests.len())
                    .map_err(|_| capacity())?;
                for request in requests {
                    let dependency = self.resolve(realm, &directory, &request)?;
                    dependencies.push((request, self.nodes[dependency].record.clone()));
                }
                self.nodes[index].dependencies = dependencies;
                self.nodes[index].expanded = true;
            }
            pending
                .try_reserve(self.nodes[index].dependencies.len())
                .map_err(|_| capacity())?;
            pending.extend(
                self.nodes[index]
                    .dependencies
                    .iter()
                    .rev()
                    .map(|(_, record)| self.records[record]),
            );
        }
        let root = self.nodes[root].record.clone();
        realm.load_module_graph(&root, |referrer, request| {
            Ok(self.nodes[self.records[referrer]]
                .dependencies
                .iter()
                .find(|(loaded, _)| loaded == request)
                .expect("prepared static request")
                .1
                .clone())
        })?;
        Ok(root)
    }

    pub(super) fn service_requests(&mut self, realm: &mut Realm) -> Result<(), Error> {
        while let Some(request) = realm.take_module_import_request() {
            let directory = match request.referrer() {
                Some(record) => self.nodes[*self
                    .records
                    .get(record)
                    .ok_or_else(|| host("unknown Module import referrer".into()))?]
                .directory
                .clone(),
                None => PathBuf::from("."),
            };
            let result = self
                .resolve(realm, &directory, request.request())
                .and_then(|index| self.publish_graph(realm, index));
            match result {
                Ok(record) => realm.finish_module_import(request, Ok(record))?,
                Err(error) => {
                    let error = match error {
                        Error::Parse(diagnostic) if diagnostic.kind == DiagnosticKind::Syntax => {
                            Error::Exception {
                                kind: ExceptionKind::SyntaxError,
                                span: diagnostic.span,
                                message: diagnostic.message,
                            }
                        }
                        error => error,
                    };
                    let Some(reason) = realm.exception_value(&error)? else {
                        return Err(error);
                    };
                    realm.finish_module_import(request, Err(reason))?;
                }
            }
        }
        Ok(())
    }
}
