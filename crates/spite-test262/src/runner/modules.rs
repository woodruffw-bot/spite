//! Host-provided sources, with shared static/dynamic identity caches per Realm.

use super::*;
use spite_parser::ast::{Module, ModuleRequest};
use spite_runtime::ModuleHandle;
use std::collections::{HashMap, HashSet};

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

pub(super) enum ModuleLoadError {
    Preparation(Error),
    Resolution(Error),
}

impl From<Error> for ModuleLoadError {
    fn from(error: Error) -> Self {
        Self::Preparation(error)
    }
}

impl ModuleLoadError {
    pub fn into_error(self) -> Error {
        match self {
            Self::Preparation(error) | Self::Resolution(error) => error,
        }
    }
}

struct Node {
    identifier: String,
    source: String,
    handle: ModuleHandle,
    requests: Vec<ModuleRequest>,
    dependencies: Vec<ModuleHandle>,
    expanded: bool,
    attribute_error: Option<Error>,
    next: usize,
}

// Each execution variant creates a fresh Realm and therefore a fresh cache.
#[derive(Default)]
pub(super) struct ModuleCache {
    nodes: Vec<Node>,
    identifiers: HashMap<String, usize>,
    records: HashMap<ModuleHandle, usize>,
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
fn supported_attributes(request: &ModuleRequest) -> Result<(), Error> {
    if request.attributes.is_empty() {
        return Ok(());
    }
    // The conformance host uses HostGetSupportedImportAttributes' empty default.
    Err(Error::Exception {
        kind: spite_runtime::ExceptionKind::SyntaxError,
        span: Span::new(0, 0),
        message: "unsupported import attribute key".into(),
    })
}

impl ModuleCache {
    fn insert(
        &mut self,
        realm: &mut Realm,
        source: ModuleSource,
        parsed: Option<&Module>,
    ) -> Result<usize, Error> {
        if let Some(&index) = self.identifiers.get(&source.identifier) {
            if self.nodes[index].source != source.source {
                return Err(host(format!(
                    "module identity changed source: {}",
                    source.identifier
                )));
            }
            return Ok(index);
        }
        let handle = match parsed {
            Some(module) => realm.create_module_from_ast(module)?,
            None => realm.create_module(&source.source)?,
        };
        let requests = realm.module_requests(&handle)?;
        self.nodes.try_reserve(1).map_err(|_| capacity())?;
        self.identifiers.try_reserve(1).map_err(|_| capacity())?;
        self.records.try_reserve(1).map_err(|_| capacity())?;
        let index = self.nodes.len();
        self.identifiers.insert(source.identifier.clone(), index);
        self.records.insert(handle.clone(), index);
        self.nodes.push(Node {
            identifier: source.identifier,
            source: source.source,
            handle,
            requests,
            dependencies: Vec::new(),
            expanded: false,
            attribute_error: None,
            next: 0,
        });
        Ok(index)
    }
}

impl ModuleLoader<'_> {
    pub fn load_graph(
        &mut self,
        realm: &mut Realm,
        module: &Module,
        source: &str,
        cache: &mut ModuleCache,
    ) -> Result<ModuleHandle, ModuleLoadError> {
        let root = cache.insert(
            realm,
            ModuleSource {
                identifier: self.root_identifier.into(),
                source: source.into(),
            },
            Some(module),
        )?;
        self.publish_graph(realm, root, cache)
    }

    fn publish_graph(
        &mut self,
        realm: &mut Realm,
        root: usize,
        cache: &mut ModuleCache,
    ) -> Result<ModuleHandle, ModuleLoadError> {
        // A failed graph must not block subsequent unrelated imports. Retain
        // source identities globally, but expand only this root's dependencies.
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
            if let Some(error) = &cache.nodes[index].attribute_error {
                return Err(ModuleLoadError::Resolution(error.clone()));
            }
            if !cache.nodes[index].expanded {
                let requests = cache.nodes[index].requests.clone();
                let mut dependencies = Vec::new();
                dependencies
                    .try_reserve_exact(requests.len())
                    .map_err(|_| capacity())?;
                for request in requests {
                    if let Err(error) = supported_attributes(&request) {
                        cache.nodes[index].attribute_error = Some(error.clone());
                        return Err(ModuleLoadError::Resolution(error));
                    }
                    let loaded =
                        (self.load)(&cache.nodes[index].identifier, &request).map_err(host)?;
                    let target = cache.insert(realm, loaded, None)?;
                    dependencies.push(cache.nodes[target].handle.clone());
                }
                cache.nodes[index].dependencies = dependencies;
                cache.nodes[index].expanded = true;
            }
            pending
                .try_reserve(cache.nodes[index].dependencies.len())
                .map_err(|_| capacity())?;
            pending.extend(
                cache.nodes[index]
                    .dependencies
                    .iter()
                    .rev()
                    .map(|record| cache.records[record]),
            );
        }
        let root = cache.nodes[root].handle.clone();
        realm
            .load_module_graph(&root, |referrer, request| {
                let node = &mut cache.nodes[cache.records[referrer]];
                debug_assert_eq!(&node.requests[node.next], request);
                let target = node.dependencies[node.next].clone();
                node.next += 1;
                Ok(target)
            })
            .map_err(ModuleLoadError::Resolution)?;
        Ok(root)
    }

    pub fn service_requests(
        &mut self,
        realm: &mut Realm,
        cache: &mut ModuleCache,
    ) -> Result<(), Error> {
        while let Some(request) = realm.take_module_import_request() {
            supported_attributes(request.request())?;
            let referrer = match request.referrer() {
                None => self.root_identifier,
                Some(record) => {
                    &cache.nodes[*cache
                        .records
                        .get(record)
                        .ok_or_else(|| host("unknown dynamic import referrer".into()))?]
                    .identifier
                }
            };
            let loaded = (self.load)(referrer, request.request()).map_err(host)?;
            let target = cache.insert(realm, loaded, None)?;
            match self.publish_graph(realm, target, cache) {
                Ok(target) => realm.finish_module_import(request, Ok(target))?,
                Err(ModuleLoadError::Resolution(error)) => {
                    let Some(reason) = realm.exception_value(&error)? else {
                        return Err(error);
                    };
                    realm.finish_module_import(request, Err(reason))?;
                }
                Err(ModuleLoadError::Preparation(error)) => return Err(error),
            }
        }
        Ok(())
    }
}

impl Runner {
    // Loading-time attribute errors and linking errors satisfy resolution
    // negatives. Dependency parsing and body throws cannot supply the exception.
    pub(super) fn run_module_resolution(
        &self,
        module: &Module,
        source: &str,
        negative: &Negative,
        loader: Option<&mut ModuleLoader<'_>>,
    ) -> Outcome {
        let mut realm = Realm::new(self.limits);
        let loaded = match loader {
            Some(loader) => {
                loader.load_graph(&mut realm, module, source, &mut ModuleCache::default())
            }
            None => realm
                .create_module_from_ast(module)
                .map_err(ModuleLoadError::Preparation),
        };
        let resolution = match loaded {
            Ok(record) => realm.link_module(&record),
            Err(ModuleLoadError::Resolution(error)) => Err(error),
            Err(ModuleLoadError::Preparation(error)) => {
                return host_failure(&error, Stage::Resolution).unwrap_or_else(|| {
                    failed(
                        Stage::Resolution,
                        format!("dependency preparation failed before linking: {error}"),
                    )
                });
            }
        };
        match resolution {
            Ok(()) => failed(
                Stage::Resolution,
                "expected a resolution exception, but linking completed normally",
            ),
            Err(error) => {
                if let Some(outcome) = host_failure(&error, Stage::Resolution) {
                    return outcome;
                }
                match exception_constructor_name(&mut realm, &error) {
                    Ok(Some(name)) if name == JsString::from(negative.error_type.as_str()) => {
                        Outcome::Passed
                    }
                    Ok(name) => failed(
                        Stage::Resolution,
                        format!(
                            "resolution exception with constructor name {name:?} does not match {}",
                            negative.error_type
                        ),
                    ),
                    Err(error) => host_failure(&error, Stage::Resolution).unwrap_or_else(|| {
                        failed(
                            Stage::Resolution,
                            format!("resolution exception inspection failed: {error}"),
                        )
                    }),
                }
            }
        }
    }
}
