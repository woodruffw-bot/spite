//! Host-provided static sources, with identity caching before graph publication.

use super::*;
use spite_parser::ast::{Module, ModuleRequest};
use spite_runtime::ModuleHandle;
use std::collections::HashMap;

/// A host-resolved Source Text Module and its stable identity.
#[derive(Clone, Debug)]
pub struct ModuleSource {
    /// Equal identifiers reuse one record, including the root identifier.
    pub identifier: String,
    /// Unmodified UTF-8 Module text. Repeated identities must have equal sources.
    pub source: String,
}

pub(super) struct ModuleLoader<'a> {
    pub root_identifier: &'a str,
    pub load: &'a mut dyn FnMut(&str, &ModuleRequest) -> Result<ModuleSource, String>,
}

struct Node {
    identifier: String,
    source: String,
    handle: ModuleHandle,
    requests: Vec<ModuleRequest>,
    dependencies: Vec<ModuleHandle>,
    next: usize,
}

fn capacity() -> Error {
    Error::Limit {
        span: Span::new(0, 0),
        message: "module source capacity exceeded".into(),
    }
}
fn host(message: String) -> Error {
    Error::Host {
        span: Span::new(0, 0),
        message,
    }
}

impl ModuleLoader<'_> {
    pub fn load_graph(
        &mut self,
        realm: &mut Realm,
        module: &Module,
        source: &str,
    ) -> Result<ModuleHandle, Error> {
        let root = realm.create_module_from_ast(module)?;
        let mut nodes = vec![Node {
            identifier: self.root_identifier.into(),
            source: source.into(),
            handle: root.clone(),
            requests: realm.module_requests(&root)?,
            dependencies: Vec::new(),
            next: 0,
        }];
        let mut identifiers = HashMap::new();
        identifiers.try_reserve(1).map_err(|_| capacity())?;
        identifiers.insert(self.root_identifier.to_owned(), 0usize);
        let mut index = 0;
        while index < nodes.len() {
            let requests = nodes[index].requests.clone();
            nodes[index]
                .dependencies
                .try_reserve_exact(requests.len())
                .map_err(|_| capacity())?;
            for request in requests {
                if !request.attributes.is_empty() {
                    return Err(Error::Unsupported {
                        span: Span::new(0, 0),
                        message: "Test262 module import attributes are not implemented".into(),
                    });
                }
                let loaded = (self.load)(&nodes[index].identifier, &request).map_err(host)?;
                let target = if let Some(target) = identifiers.get(&loaded.identifier) {
                    let target = *target;
                    if nodes[target].source != loaded.source {
                        return Err(host(format!(
                            "module identity changed source: {}",
                            loaded.identifier
                        )));
                    }
                    target
                } else {
                    let handle = realm.create_module(&loaded.source)?;
                    let requests = realm.module_requests(&handle)?;
                    nodes.try_reserve(1).map_err(|_| capacity())?;
                    identifiers.try_reserve(1).map_err(|_| capacity())?;
                    let target = nodes.len();
                    identifiers.insert(loaded.identifier.clone(), target);
                    nodes.push(Node {
                        identifier: loaded.identifier,
                        source: loaded.source,
                        handle,
                        requests,
                        dependencies: Vec::new(),
                        next: 0,
                    });
                    target
                };
                let dependency = nodes[target].handle.clone();
                nodes[index].dependencies.push(dependency);
            }
            index += 1;
        }
        let mut records = HashMap::new();
        records.try_reserve(nodes.len()).map_err(|_| capacity())?;
        for (index, node) in nodes.iter().enumerate() {
            records.insert(node.handle.clone(), index);
        }
        realm.load_module_graph(&root, |referrer, request| {
            let node = &mut nodes[records[referrer]];
            debug_assert_eq!(&node.requests[node.next], request);
            let target = node.dependencies[node.next].clone();
            node.next += 1;
            Ok(target)
        })?;
        Ok(root)
    }
}
