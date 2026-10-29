//! Side-effect-free export names and ResolveExport, using explicit stacks (16.2.1).

use super::*;
use crate::object::{Budget, Error as StoreError};
use spite_core::JsString;
use spite_parser::ast::{ModuleEntries, ModuleRequest};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;

pub(super) struct ModuleInfo {
    pub record: ObjectHandle,
    pub entries: ModuleEntries,
    pub dependencies: Vec<usize>,
}

impl ModuleInfo {
    pub fn dependency(&self, request: &ModuleRequest) -> usize {
        let index = self
            .entries
            .requested_modules
            .iter()
            .position(|candidate| candidate == request)
            .expect("normalized module request");
        self.dependencies[index]
    }
}

pub(super) struct ModuleCatalog {
    pub nodes: Vec<ModuleInfo>,
    pub positions: HashMap<ObjectHandle, usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ExportResolution {
    Missing,
    Ambiguous,
    Binding(ModuleBinding),
}

enum ResolveFrame {
    Visit(usize, JsString),
    Star {
        node: usize,
        name: JsString,
        next: usize,
        candidate: Option<ModuleBinding>,
    },
}

fn push<T>(stack: &mut Vec<T>, value: T, budget: &mut Budget) -> Result<(), StoreError> {
    budget.charge(1)?;
    stack
        .try_reserve(1)
        .map_err(|_| spite_heap::Error::Capacity)?;
    stack.push(value);
    Ok(())
}

impl ModuleCatalog {
    pub fn resolve(
        &self,
        node: usize,
        name: &JsString,
        budget: &mut Budget,
    ) -> Result<ExportResolution, StoreError> {
        let mut frames = Vec::new();
        push(&mut frames, ResolveFrame::Visit(node, name.clone()), budget)?;
        let mut visited = HashSet::new();
        let mut result = None;
        while let Some(frame) = frames.pop() {
            budget.charge(1)?;
            match frame {
                ResolveFrame::Visit(node, name) => {
                    budget.charge(name.len() + 1)?;
                    visited
                        .try_reserve(1)
                        .map_err(|_| spite_heap::Error::Capacity)?;
                    if !visited.insert((node, name.clone())) {
                        result = Some(ExportResolution::Missing);
                        continue;
                    };
                    let info = &self.nodes[node];
                    budget.charge(info.entries.local_export_entries.len())?;
                    if let Some(entry) = info
                        .entries
                        .local_export_entries
                        .iter()
                        .find(|entry| entry.exported == name)
                    {
                        budget.charge(entry.local.len() + 1)?;
                        result = Some(ExportResolution::Binding(ModuleBinding {
                            module: info.record.clone(),
                            name: Some(Rc::from(
                                String::from_utf16(entry.local.code_units())
                                    .expect("local binding name"),
                            )),
                        }));
                        continue;
                    }
                    budget.charge(info.entries.indirect_export_entries.len())?;
                    if let Some(entry) = info
                        .entries
                        .indirect_export_entries
                        .iter()
                        .find(|entry| entry.exported == name)
                    {
                        let target = info.dependency(&entry.request);
                        if let Some(imported) = &entry.imported {
                            push(
                                &mut frames,
                                ResolveFrame::Visit(target, imported.clone()),
                                budget,
                            )?;
                            result = None;
                        } else {
                            result = Some(ExportResolution::Binding(ModuleBinding {
                                module: self.nodes[target].record.clone(),
                                name: None,
                            }));
                        }
                        continue;
                    }
                    if name == JsString::from("default") {
                        result = Some(ExportResolution::Missing);
                        continue;
                    };
                    push(
                        &mut frames,
                        ResolveFrame::Star {
                            node,
                            name,
                            next: 0,
                            candidate: None,
                        },
                        budget,
                    )?;
                    result = None;
                }
                ResolveFrame::Star {
                    node,
                    name,
                    next,
                    mut candidate,
                } => {
                    if let Some(returned) = result.take() {
                        match returned {
                            ExportResolution::Ambiguous => {
                                result = Some(ExportResolution::Ambiguous);
                                continue;
                            }
                            ExportResolution::Binding(binding) => {
                                if candidate
                                    .as_ref()
                                    .is_some_and(|previous| previous != &binding)
                                {
                                    result = Some(ExportResolution::Ambiguous);
                                    continue;
                                };
                                candidate = Some(binding);
                            }
                            ExportResolution::Missing => (),
                        }
                    }
                    let info = &self.nodes[node];
                    if let Some(request) = info.entries.star_export_entries.get(next) {
                        let target = info.dependency(request);
                        push(
                            &mut frames,
                            ResolveFrame::Star {
                                node,
                                name: name.clone(),
                                next: next + 1,
                                candidate,
                            },
                            budget,
                        )?;
                        push(&mut frames, ResolveFrame::Visit(target, name), budget)?;
                    } else {
                        result = Some(
                            candidate.map_or(ExportResolution::Missing, ExportResolution::Binding),
                        )
                    };
                }
            }
        }
        Ok(result.expect("export resolution"))
    }

    pub fn export_names(
        &self,
        root: usize,
        budget: &mut Budget,
    ) -> Result<Vec<JsString>, StoreError> {
        let mut visited = Vec::new();
        visited
            .try_reserve_exact(self.nodes.len())
            .map_err(|_| spite_heap::Error::Capacity)?;
        visited.resize(self.nodes.len(), false);
        let mut stack = Vec::new();
        push(&mut stack, root, budget)?;
        let mut names = BTreeSet::new();
        while let Some(node) = stack.pop() {
            if visited[node] {
                continue;
            };
            visited[node] = true;
            let info = &self.nodes[node];
            for name in info
                .entries
                .local_export_entries
                .iter()
                .map(|entry| &entry.exported)
                .chain(
                    info.entries
                        .indirect_export_entries
                        .iter()
                        .map(|entry| &entry.exported),
                )
            {
                budget.charge(name.len() + 1)?;
                if node == root || *name != JsString::from("default") {
                    names.insert(name.clone());
                }
            }
            for request in info.entries.star_export_entries.iter().rev() {
                push(&mut stack, info.dependency(request), budget)?;
            }
        }
        Ok(names.into_iter().collect())
    }
}

impl Realm {
    pub(super) fn module_catalog(
        &mut self,
        root: &ObjectHandle,
        span: Span,
    ) -> Result<ModuleCatalog, Error> {
        let mut records = Vec::new();
        records.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "module catalog capacity exceeded".into(),
        })?;
        records.push(root.clone());
        let mut positions = HashMap::new();
        positions.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "module catalog capacity exceeded".into(),
        })?;
        positions.insert(root.clone(), 0);
        let mut nodes = Vec::new();
        let mut next = 0;
        while next < records.len() {
            let record = &records[next];
            let (entries, dependencies) = self.object_work(span, |objects, budget| {
                let source = objects.module_source(record)?;
                budget.charge(source.imports().len() + source.exports().len() + 1)?;
                Ok((
                    source.entries(),
                    objects.module_dependencies(record)?.map(<[_]>::to_vec),
                ))
            })?;
            for request in &entries.requested_modules {
                self.check_import_attributes(
                    &request.attributes,
                    crate::ExceptionKind::SyntaxError,
                    span,
                )?;
            }
            let dependencies = match dependencies {
                Some(dependencies) => dependencies,
                None if entries.requested_modules.is_empty() => Vec::new(),
                None => {
                    return Err(Self::unsupported(
                        span,
                        "module dependencies have not been loaded",
                    ));
                }
            };
            debug_assert_eq!(entries.requested_modules.len(), dependencies.len());
            let mut indices = Vec::new();
            indices
                .try_reserve_exact(dependencies.len())
                .map_err(|_| Error::Limit {
                    span,
                    message: "module catalog capacity exceeded".into(),
                })?;
            for dependency in dependencies {
                let index = if let Some(index) = positions.get(&dependency) {
                    *index
                } else {
                    let index = records.len();
                    records.try_reserve(1).map_err(|_| Error::Limit {
                        span,
                        message: "module catalog capacity exceeded".into(),
                    })?;
                    positions.try_reserve(1).map_err(|_| Error::Limit {
                        span,
                        message: "module catalog capacity exceeded".into(),
                    })?;
                    positions.insert(dependency.clone(), index);
                    records.push(dependency);
                    index
                };
                indices.push(index);
            }
            nodes.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "module catalog capacity exceeded".into(),
            })?;
            nodes.push(ModuleInfo {
                record: records[next].clone(),
                entries,
                dependencies: indices,
            });
            next += 1;
        }
        Ok(ModuleCatalog { nodes, positions })
    }
}
