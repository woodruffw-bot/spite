//! Explicit filesystem paths for the command-line Module host.

use spite::{Error, ExceptionKind, ModuleHandle, Realm, Span, ast::ModuleRequest};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

struct Node {
    record: ModuleHandle,
    directory: PathBuf,
    dependencies: Vec<(ModuleRequest, ModuleHandle)>,
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

pub(super) fn load(
    realm: &mut Realm,
    source: &str,
    filename: Option<&Path>,
) -> Result<ModuleHandle, Error> {
    let filename = filename.map(canonical).transpose()?;
    let directory = match &filename {
        Some(path) => path.parent().expect("canonical file parent").to_path_buf(),
        None => PathBuf::from("."),
    };
    let root = realm.create_module(source)?;
    let mut files = HashMap::new();
    if let Some(filename) = filename {
        files.try_reserve(1).map_err(|_| capacity())?;
        files.insert(filename, 0usize);
    }
    let mut nodes = Vec::new();
    nodes.try_reserve(1).map_err(|_| capacity())?;
    nodes.push(Node {
        record: root.clone(),
        directory,
        dependencies: Vec::new(),
    });
    let mut index = 0;
    while index < nodes.len() {
        let requests = realm.module_requests(&nodes[index].record)?;
        nodes[index]
            .dependencies
            .try_reserve_exact(requests.len())
            .map_err(|_| capacity())?;
        for request in requests {
            // This host supports the empty import-attribute key set. Reject
            // unknown keys before reading a file, as InnerModuleLoading does.
            if !request.attributes.is_empty() {
                return Err(Error::Exception {
                    kind: ExceptionKind::SyntaxError,
                    span: Span::new(0, 0),
                    message: "unsupported import attribute key".into(),
                });
            }
            let specifier = String::from_utf16(request.specifier.code_units())
                .map_err(|_| host("Module file paths require valid Unicode".into()))?;
            let path = canonical(&nodes[index].directory.join(specifier))?;
            let dependency = if let Some(&existing) = files.get(&path) {
                existing
            } else {
                let source = fs::read_to_string(&path).map_err(|error| {
                    host(format!("cannot read Module {}: {error}", path.display()))
                })?;
                let record = realm.create_module(&source)?;
                let directory = path.parent().expect("canonical file parent").to_path_buf();
                nodes.try_reserve(1).map_err(|_| capacity())?;
                files.try_reserve(1).map_err(|_| capacity())?;
                let target = nodes.len();
                nodes.push(Node {
                    record,
                    directory,
                    dependencies: Vec::new(),
                });
                files.insert(path, target);
                target
            };
            let record = nodes[dependency].record.clone();
            nodes[index].dependencies.push((request, record));
        }
        index += 1;
    }
    let mut records = HashMap::new();
    records.try_reserve(nodes.len()).map_err(|_| capacity())?;
    for (index, node) in nodes.iter().enumerate() {
        records.insert(node.record.clone(), index);
    }
    realm.load_module_graph(&root, |referrer, request| {
        Ok(nodes[records[referrer]]
            .dependencies
            .iter()
            .find(|(loaded, _)| loaded == request)
            .expect("prepared static request")
            .1
            .clone())
    })?;
    Ok(root)
}
