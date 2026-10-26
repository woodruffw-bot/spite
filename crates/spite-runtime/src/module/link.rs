//! Iterative InnerModuleLinking and SCC completion (16.2.1.6.1).

use super::resolution::{ExportResolution, ModuleCatalog};
use super::*;
use crate::{ExceptionKind, object::ModuleLinkState};
use spite_parser::ast::ImportName;

struct LinkFrame {
    node: usize,
    next: usize,
}

#[cfg(test)]
mod tests;

fn capacity_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "module linking capacity exceeded".into(),
    }
}

impl Realm {
    /// Links loaded records and initializes their bindings without body evaluation.
    ///
    /// Load requests with [`Self::load_module_graph`] first. Cyclic groups finish
    /// linking together. A failure resets the active group while completed
    /// dependencies remain linked. Synchronous graphs can then be evaluated with
    /// [`Self::evaluate_module_record`].
    pub fn link_module(&mut self, module: &ModuleHandle) -> Result<(), Error> {
        let record = module.root.handle();
        if self
            .objects
            .module_link_state(record)
            .map_err(Error::InvalidObject)?
            == ModuleLinkState::Linked
        {
            return Ok(());
        };
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        self.link_module_record(record, Span::new(0, 0))
    }

    pub(super) fn link_module_record(
        &mut self,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        if self
            .objects
            .module_link_state(record)
            .map_err(Error::InvalidObject)?
            == ModuleLinkState::Linked
        {
            return Ok(());
        };
        let catalog = self.module_catalog(record, span)?;
        let count = catalog.nodes.len();
        let mut indices = Vec::new();
        indices
            .try_reserve_exact(count)
            .map_err(|_| capacity_error(span))?;
        indices.resize(count, None);
        let mut ancestors = Vec::new();
        ancestors
            .try_reserve_exact(count)
            .map_err(|_| capacity_error(span))?;
        ancestors.resize(count, 0usize);
        let mut stack = Vec::new();
        stack
            .try_reserve_exact(count)
            .map_err(|_| capacity_error(span))?;
        let mut frames = Vec::new();
        frames
            .try_reserve_exact(count)
            .map_err(|_| capacity_error(span))?;
        self.object_work(span, |_, budget| {
            budget.charge(
                count
                    .checked_mul(4)
                    .ok_or(crate::object::Error::WorkLimit)?,
            )
        })?;
        self.object_work(span, |objects, budget| {
            objects.begin_module_link(record, budget)
        })?;
        indices[0] = Some(0);
        stack.push(0usize);
        frames.push(LinkFrame { node: 0, next: 0 });
        let mut next_index = 1;
        let result = (|| {
            while let Some(frame) = frames.last_mut() {
                let node = frame.node;
                if let Some(&dependency) = catalog.nodes[node].dependencies.get(frame.next) {
                    frame.next += 1;
                    match self
                        .objects
                        .module_link_state(&catalog.nodes[dependency].record)
                        .map_err(Error::InvalidObject)?
                    {
                        ModuleLinkState::Unlinked => {
                            self.object_work(span, |objects, budget| {
                                objects.begin_module_link(&catalog.nodes[dependency].record, budget)
                            })?;
                            indices[dependency] = Some(next_index);
                            ancestors[dependency] = next_index;
                            next_index += 1;
                            stack.push(dependency);
                            frames.push(LinkFrame {
                                node: dependency,
                                next: 0,
                            });
                        }
                        ModuleLinkState::Linking => {
                            ancestors[node] = ancestors[node].min(ancestors[dependency])
                        }
                        ModuleLinkState::Linked => (),
                    }
                    continue;
                }
                self.initialize_link_environment(&catalog, node, span)?;
                if ancestors[node] == indices[node].expect("visited module") {
                    let start = stack
                        .iter()
                        .rposition(|candidate| *candidate == node)
                        .expect("active SCC root");
                    let mut members = Vec::new();
                    members
                        .try_reserve_exact(stack.len() - start)
                        .map_err(|_| capacity_error(span))?;
                    members.extend(
                        stack[start..]
                            .iter()
                            .map(|index| catalog.nodes[*index].record.clone()),
                    );
                    self.object_work(span, |objects, budget| {
                        objects.finish_module_links(&members, budget)
                    })?;
                    stack.truncate(start);
                }
                frames.pop();
                if let Some(parent) = frames.last()
                    && self
                        .objects
                        .module_link_state(&catalog.nodes[node].record)
                        .map_err(Error::InvalidObject)?
                        == ModuleLinkState::Linking
                {
                    ancestors[parent.node] = ancestors[parent.node].min(ancestors[node]);
                }
            }
            Ok(())
        })();
        if result.is_err() {
            // Release the active groups even when a host work budget is zero.
            for node in stack {
                self.objects
                    .rollback_module_links(std::slice::from_ref(&catalog.nodes[node].record))
                    .expect("active module record");
            }
        }
        result
    }

    fn initialize_link_environment(
        &mut self,
        catalog: &ModuleCatalog,
        node: usize,
        span: Span,
    ) -> Result<(), Error> {
        let info = &catalog.nodes[node];
        for entry in &info.entries.indirect_export_entries {
            let resolution = self.object_work(span, |_, budget| {
                catalog.resolve(node, &entry.exported, budget)
            })?;
            if !matches!(resolution, ExportResolution::Binding(_)) {
                return Err(Self::exception(
                    ExceptionKind::SyntaxError,
                    span,
                    "module export is missing or ambiguous",
                ));
            };
        }
        let source = self
            .objects
            .module_source(&info.record)
            .map_err(Error::InvalidObject)?
            .clone();
        let mut imports = BTreeMap::new();
        for declaration in source.imports() {
            let target = info.dependency(&declaration.request);
            for binding in &declaration.bindings {
                let resolution = match &binding.imported {
                    ImportName::Namespace => ModuleBinding {
                        module: catalog.nodes[target].record.clone(),
                        name: None,
                    },
                    ImportName::Name(name) => {
                        let resolution = self.object_work(binding.span, |_, budget| {
                            catalog.resolve(target, name, budget)
                        })?;
                        let ExportResolution::Binding(binding) = resolution else {
                            return Err(Self::exception(
                                ExceptionKind::SyntaxError,
                                binding.span,
                                "module import is missing or ambiguous",
                            ));
                        };
                        binding
                    }
                };
                if resolution.name.is_none() {
                    self.module_namespace_record(&resolution.module, binding.span)?;
                }
                self.object_work(binding.span, |_, budget| {
                    budget.charge(binding.local.len() + 1)
                })?;
                imports.insert(binding.local.clone(), resolution);
            }
        }
        self.initialize_module_record(&source, &info.record, imports)
    }
}
