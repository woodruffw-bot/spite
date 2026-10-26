//! Host-supplied dependency loading, separate from linking/evaluation (16.2.1).

use super::*;
use spite_parser::ast::ModuleRequest;
use std::collections::HashSet;

struct LoadNode {
    module: ModuleHandle,
    requests: Vec<ModuleRequest>,
    next: usize,
    dependencies: Vec<ObjectHandle>,
}

fn capacity_error() -> Error {
    Error::Limit {
        span: Span::new(0, 0),
        message: "module graph capacity exceeded".into(),
    }
}

impl Realm {
    /// Returns distinct static requests in source order, with canonical attributes.
    pub fn module_requests(&self, module: &ModuleHandle) -> Result<Vec<ModuleRequest>, Error> {
        Ok(self
            .objects
            .module_source(module.root.handle())
            .map_err(Error::InvalidObject)?
            .requested_modules())
    }

    /// Loads a dependency graph using records supplied by the embedding host.
    ///
    /// The resolver receives a referrer and each distinct request in depth-first
    /// source order. Record identity handles shared dependencies and cycles.
    /// Successfully loaded records are cached, so repeated loading calls no
    /// resolver. Errors leave new dependency links unchanged. No body executes;
    /// graph linking and dependency evaluation remain unsupported.
    pub fn load_module_graph(
        &mut self,
        root: &ModuleHandle,
        mut resolve: impl FnMut(&ModuleHandle, &ModuleRequest) -> Result<ModuleHandle, Error>,
    ) -> Result<(), Error> {
        if self
            .objects
            .module_dependencies(root.root.handle())
            .map_err(Error::InvalidObject)?
            .is_some()
        {
            return Ok(());
        };
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        let span = Span::new(0, 0);
        let mut nodes = Vec::new();
        nodes.try_reserve(1).map_err(|_| capacity_error())?;
        nodes.push(self.module_load_node(root.clone())?);
        let mut seen = HashSet::new();
        seen.try_reserve(1).map_err(|_| capacity_error())?;
        seen.insert(root.root.handle().clone());
        let mut stack = Vec::new();
        stack.try_reserve(1).map_err(|_| capacity_error())?;
        stack.push(0usize);
        while let Some(&index) = stack.last() {
            if nodes[index].next == nodes[index].requests.len() {
                stack.pop();
                continue;
            };
            let request = nodes[index].requests[nodes[index].next].clone();
            nodes[index].next += 1;
            self.tick(span)?;
            let dependency = resolve(&nodes[index].module, &request)?;
            let record = dependency.root.handle();
            // Validate ownership before installing any graph edge.
            let loaded = self
                .objects
                .module_dependencies(record)
                .map_err(Error::InvalidObject)?
                .is_some();
            nodes[index].dependencies.push(record.clone());
            if !loaded && !seen.contains(record) {
                seen.try_reserve(1).map_err(|_| capacity_error())?;
                seen.insert(record.clone());
                nodes.try_reserve(1).map_err(|_| capacity_error())?;
                stack.try_reserve(1).map_err(|_| capacity_error())?;
                stack.push(nodes.len());
                nodes.push(self.module_load_node(dependency)?);
            }
        }
        let mut graph = Vec::new();
        graph
            .try_reserve_exact(nodes.len())
            .map_err(|_| capacity_error())?;
        for node in nodes {
            graph.push((node.module.root.handle().clone(), node.dependencies));
        }
        // Validate/prepay the entire batch before publishing dependency links.
        self.object_work(span, |objects, budget| {
            objects.install_module_graph(graph, budget)
        })
    }

    fn module_load_node(&mut self, module: ModuleHandle) -> Result<LoadNode, Error> {
        let requests = self.object_work(Span::new(0, 0), |objects, budget| {
            let source = objects.module_source(module.root.handle())?;
            let count = source
                .imports()
                .len()
                .checked_add(source.exports().len())
                .ok_or(crate::object::Error::WorkLimit)?;
            let levels = count.max(1).ilog2() as usize + 1;
            budget.charge(
                count
                    .checked_add(1)
                    .and_then(|count| count.checked_mul(levels))
                    .ok_or(crate::object::Error::WorkLimit)?,
            )?;
            for request in
                source
                    .imports()
                    .iter()
                    .map(|declaration| &declaration.request)
                    .chain(source.exports().iter().filter_map(
                        |declaration| match &declaration.kind {
                            ExportKind::Named {
                                request: Some(request),
                                ..
                            }
                            | ExportKind::Star { request, .. } => Some(request),
                            _ => None,
                        },
                    ))
            {
                budget.charge(request.specifier.len())?;
                for attribute in &request.attributes {
                    budget.charge(attribute.key.len())?;
                    budget.charge(attribute.value.len() + 1)?;
                }
            }
            let requests = source.requested_modules();
            Ok(requests)
        })?;
        let mut dependencies = Vec::new();
        dependencies
            .try_reserve_exact(requests.len())
            .map_err(|_| capacity_error())?;
        Ok(LoadNode {
            module,
            requests,
            next: 0,
            dependencies,
        })
    }
}
