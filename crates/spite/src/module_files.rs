//! Explicit filesystem paths for the command-line Module host.

use spite::ast::ModuleRequest;
use spite::{DiagnosticKind, Error, ExceptionKind, JsString, ModuleHandle, Realm, Span};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

struct Node {
    record: ModuleHandle,
    directory: PathBuf,
    dependencies: Vec<(ModuleRequest, ModuleHandle)>,
    expanded: bool,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum ModuleType {
    JavaScript,
    Json,
}

impl ModuleType {
    fn from_request(request: &ModuleRequest) -> Result<Self, Error> {
        let mut kind = Self::JavaScript;
        for attribute in &request.attributes {
            if attribute.key != JsString::from("type") {
                return Err(Error::Exception {
                    kind: ExceptionKind::SyntaxError,
                    span: Span::new(0, 0),
                    message: "unsupported import attribute key".into(),
                });
            }
            if attribute.value != JsString::from("json") {
                return Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    span: Span::new(0, 0),
                    message: "unsupported module type".into(),
                });
            }
            kind = Self::Json;
        }
        Ok(kind)
    }
}

#[derive(Default)]
pub(super) struct FileModules {
    nodes: Vec<Node>,
    files: HashMap<(PathBuf, ModuleType), usize>,
    records: HashMap<ModuleHandle, usize>,
    script_directory: PathBuf,
    max_source_bytes: Option<usize>,
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
    pub(super) fn for_script(
        filename: Option<&Path>,
        max_source_bytes: Option<usize>,
    ) -> Result<Self, Error> {
        let filename = filename.map(canonical).transpose()?;
        let script_directory = match filename {
            Some(path) => path.parent().expect("canonical file parent").to_path_buf(),
            None => PathBuf::from("."),
        };
        Ok(Self {
            script_directory,
            max_source_bytes,
            ..Self::default()
        })
    }

    pub(super) fn load(
        realm: &mut Realm,
        source: &str,
        filename: Option<&Path>,
        max_source_bytes: Option<usize>,
    ) -> Result<(Self, ModuleHandle), Error> {
        let filename = filename.map(canonical).transpose()?;
        let directory = match &filename {
            Some(path) => path.parent().expect("canonical file parent").to_path_buf(),
            None => PathBuf::from("."),
        };
        let root = realm.create_module(source)?;
        let mut loader = Self {
            script_directory: PathBuf::from("."),
            max_source_bytes,
            ..Self::default()
        };
        loader.insert(root, directory, filename, ModuleType::JavaScript)?;
        let root = loader.publish_graph(realm, 0)?;
        Ok((loader, root))
    }

    fn insert(
        &mut self,
        record: ModuleHandle,
        directory: PathBuf,
        filename: Option<PathBuf>,
        kind: ModuleType,
    ) -> Result<usize, Error> {
        self.nodes.try_reserve(1).map_err(|_| capacity())?;
        self.records.try_reserve(1).map_err(|_| capacity())?;
        if filename.is_some() {
            self.files.try_reserve(1).map_err(|_| capacity())?;
        }
        let index = self.nodes.len();
        self.records.insert(record.clone(), index);
        if let Some(filename) = filename {
            self.files.insert((filename, kind), index);
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
        // Select the host module type before filesystem access or cache lookup.
        // An explicit type chooses JSON; absent attributes select JavaScript.
        let kind = ModuleType::from_request(request)?;
        let specifier = String::from_utf16(request.specifier.code_units())
            .map_err(|_| host("Module file paths require valid Unicode".into()))?;
        let path = canonical(&directory.join(specifier))?;
        if let Some(&index) = self.files.get(&(path.clone(), kind)) {
            return Ok(index);
        }
        let file = fs::File::open(&path)
            .map_err(|error| host(format!("cannot read Module {}: {error}", path.display())))?;
        let source =
            super::read_source(file, self.max_source_bytes).map_err(|error| match error {
                Error::Host { message, .. } => {
                    host(format!("cannot read Module {}: {message}", path.display()))
                }
                error => error,
            })?;
        let record = match kind {
            ModuleType::JavaScript => realm.create_module(&source)?,
            ModuleType::Json => realm.create_json_module(&source)?,
        };
        let directory = path.parent().expect("canonical file parent").to_path_buf();
        self.insert(record, directory, Some(path), kind)
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
                None => self.script_directory.clone(),
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
