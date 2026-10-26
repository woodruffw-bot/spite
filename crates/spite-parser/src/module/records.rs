//! ModuleRequests and ParseModule's import/export entry normalization (16.2).

use super::*;
use std::collections::BTreeMap;

impl Module {
    /// Constructs static module requests and normalized import/export entries.
    ///
    /// Requests retain first source occurrence order and compare decoded
    /// specifiers plus canonical attributes. ParseModule rewrites exports of
    /// imported bindings, including namespace imports, as indirect exports.
    /// This performs no host resolution, linking or evaluation.
    pub fn entries(&self) -> ModuleEntries {
        let import_entries: Vec<_> = self
            .imports
            .iter()
            .flat_map(|declaration| {
                declaration.bindings.iter().map(|binding| ImportEntry {
                    request: declaration.request.clone(),
                    imported: binding.imported.clone(),
                    local: JsString::from(binding.local.as_str()),
                })
            })
            .collect();
        let imported: BTreeMap<_, _> = import_entries
            .iter()
            .map(|entry| (entry.local.clone(), entry))
            .collect();
        let mut local_export_entries = Vec::new();
        let mut indirect_export_entries = Vec::new();
        let mut star_export_entries = Vec::new();
        for declaration in &self.exports {
            match &declaration.kind {
                ExportKind::Default { local, .. } => local_export_entries.push(LocalExportEntry {
                    local: local
                        .as_ref()
                        .map_or_else(|| JsString::from("*default*"), |name| name.value.clone()),
                    exported: JsString::from("default"),
                }),
                ExportKind::Declaration { names } => {
                    local_export_entries.extend(names.iter().map(|name| LocalExportEntry {
                        local: name.value.clone(),
                        exported: name.value.clone(),
                    }));
                }
                ExportKind::Named {
                    specifiers,
                    request: Some(request),
                } => {
                    indirect_export_entries.extend(specifiers.iter().map(|entry| {
                        IndirectExportEntry {
                            request: request.clone(),
                            imported: Some(entry.name.value.clone()),
                            exported: entry.exported.value.clone(),
                        }
                    }));
                }
                ExportKind::Named {
                    specifiers,
                    request: None,
                } => {
                    for entry in specifiers {
                        // sec-parsemodule: rewrite imported names and namespace
                        // objects to their original request, avoiding false
                        // ambiguities during ResolveExport through star graphs.
                        if let Some(import) = imported.get(&entry.name.value) {
                            indirect_export_entries.push(IndirectExportEntry {
                                request: import.request.clone(),
                                imported: match &import.imported {
                                    ImportName::Name(name) => Some(name.clone()),
                                    ImportName::Namespace => None,
                                },
                                exported: entry.exported.value.clone(),
                            });
                        } else {
                            local_export_entries.push(LocalExportEntry {
                                local: entry.name.value.clone(),
                                exported: entry.exported.value.clone(),
                            });
                        }
                    }
                }
                ExportKind::Star {
                    exported: Some(name),
                    request,
                } => {
                    indirect_export_entries.push(IndirectExportEntry {
                        request: request.clone(),
                        imported: None,
                        exported: name.value.clone(),
                    });
                }
                ExportKind::Star {
                    exported: None,
                    request,
                } => star_export_entries.push(request.clone()),
            }
        }
        ModuleEntries {
            requested_modules: self.module_requests(),
            import_entries,
            local_export_entries,
            indirect_export_entries,
            star_export_entries,
        }
    }

    fn module_requests(&self) -> Vec<ModuleRequest> {
        let mut requests: Vec<_> = self
            .imports
            .iter()
            .map(|declaration| (declaration.span.start, &declaration.request))
            .collect();
        requests.extend(self.exports.iter().filter_map(|declaration| {
            let request = match &declaration.kind {
                ExportKind::Named {
                    request: Some(request),
                    ..
                }
                | ExportKind::Star { request, .. } => request,
                _ => return None,
            };
            Some((declaration.span.start, request))
        }));
        requests.sort_by_key(|(start, _)| *start);
        let mut seen = BTreeSet::new();
        requests
            .into_iter()
            .filter_map(|(_, request)| {
                // Attribute order is canonical, so this key is ModuleRequestsEqual.
                // Borrowed keys avoid copying specifier/attribute strings for lookup.
                let key = (
                    &request.specifier,
                    request
                        .attributes
                        .iter()
                        .map(|attribute| (&attribute.key, &attribute.value))
                        .collect::<Vec<_>>(),
                );
                seen.insert(key).then(|| request.clone())
            })
            .collect()
    }
}
