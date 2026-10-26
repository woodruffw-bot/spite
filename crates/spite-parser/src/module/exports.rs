//! Export records and Module exported-name/binding early errors (16.2.3).

use super::*;

impl Parser {
    pub(super) fn export_declaration(
        &mut self,
    ) -> Result<(ExportDeclaration, Option<Statement>), Diagnostic> {
        let start = self.bump().span.start;
        let kind = if self.eat("*") {
            let exported = if self.eat("as") {
                Some(self.module_export_name()?)
            } else {
                None
            };
            self.expect("from")?;
            let request = self.export_request()?;
            ExportKind::Star { exported, request }
        } else if self.eat("{") {
            let mut specifiers = Vec::new();
            while !self.at("}") {
                let name = self.module_export_name()?;
                let exported = if self.eat("as") {
                    self.module_export_name()?
                } else {
                    name.clone()
                };
                specifiers.push(ExportSpecifier { name, exported });
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("}")?;
            let request = if self.eat("from") {
                Some(self.export_request()?)
            } else {
                None
            };
            if request.is_none() {
                for entry in &specifiers {
                    if entry.name.quoted {
                        return Err(early(
                            entry.name.span,
                            "local export name must be an IdentifierName",
                        ));
                    }
                    let name = String::from_utf16(entry.name.value.code_units())
                        .expect("well-formed module name");
                    if reserved(&name) || strict_reserved(&name) || name == "await" {
                        return Err(early(entry.name.span, "invalid local export name"));
                    }
                }
            }
            ExportKind::Named {
                specifiers,
                request,
            }
        } else if self.at("var")
            || self.at("let")
            || self.at("const")
            || self.at("function")
            || self.at("class")
            || self.at_async_function()
        {
            // ExportDeclaration : export VariableStatement / Declaration (16.2.3).
            // Keep the ordinary declaration in the shared Module scope validator.
            let statement = self.statement(true)?;
            if !matches!(
                statement.kind,
                StatementKind::Var(_)
                    | StatementKind::Lexical { .. }
                    | StatementKind::Function(_)
                    | StatementKind::Class(_)
            ) {
                return Err(early(statement.span, "expected an export declaration"));
            }
            let mut names: Vec<_> = block_lexical_names(&statement)
                .map(|(name, span)| ModuleExportName {
                    value: JsString::from(name),
                    span,
                    quoted: false,
                })
                .collect();
            let mut variables = Vec::new();
            statement.collect_var_declarations(&mut variables);
            names.extend(variables.into_iter().map(|name| ModuleExportName {
                value: JsString::from(name.name),
                span: name.span,
                quoted: false,
            }));
            let declaration = ExportDeclaration {
                kind: ExportKind::Declaration { names },
                span: Span::new(start, statement.span.end),
            };
            return Ok((declaration, Some(statement)));
        } else if self.at("default") {
            return self.default_export(start);
        } else {
            return Err(self.error("expected an export declaration"));
        };
        self.semicolon()?;
        let end = self.tokens[self.index - 1].span.end;
        Ok((
            ExportDeclaration {
                kind,
                span: Span::new(start, end),
            },
            None,
        ))
    }

    fn default_export(
        &mut self,
        start: usize,
    ) -> Result<(ExportDeclaration, Option<Statement>), Diagnostic> {
        let keyword = self.bump().span;
        let body_start = self.current().span.start;
        // 16.2.3: declaration lookahead excludes function/async function/class
        // from the AssignmentExpression alternative. Only expressions need ASI.
        let kind = if self.at("function") || self.at_async_function() {
            StatementKind::Function(self.ordinary_function(false)?)
        } else if self.at("class") {
            StatementKind::Class(self.class_definition(false)?)
        } else {
            let expression = self.expression(2)?;
            self.semicolon()?;
            StatementKind::Expression(expression)
        };
        let end = self.tokens[self.index - 1].span.end;
        let statement = Statement {
            kind,
            span: Span::new(body_start, end),
        };
        let local = block_lexical_names(&statement)
            .next()
            .map(|(name, span)| ModuleExportName {
                value: JsString::from(name),
                span,
                quoted: false,
            });
        Ok((
            ExportDeclaration {
                kind: ExportKind::Default { local, keyword },
                span: Span::new(start, end),
            },
            Some(statement),
        ))
    }

    fn export_request(&mut self) -> Result<ModuleRequest, Diagnostic> {
        let specifier = self.import_string()?;
        let attributes = self.import_attributes()?;
        Ok(ModuleRequest {
            specifier,
            attributes,
        })
    }
}

pub(super) fn validate(
    statements: &[Statement],
    imports: &[ImportDeclaration],
    exports: &[ExportDeclaration],
) -> Result<(), Diagnostic> {
    let mut exported = BTreeSet::new();
    for declaration in exports {
        match &declaration.kind {
            ExportKind::Default { keyword, .. } => insert(
                &mut exported,
                &ModuleExportName {
                    value: JsString::from("default"),
                    span: *keyword,
                    quoted: false,
                },
            )?,
            ExportKind::Declaration { names } => {
                for name in names {
                    insert(&mut exported, name)?;
                }
            }
            ExportKind::Named { specifiers, .. } => {
                for entry in specifiers {
                    insert(&mut exported, &entry.exported)?;
                }
            }
            ExportKind::Star {
                exported: Some(name),
                ..
            } => insert(&mut exported, name)?,
            ExportKind::Star { exported: None, .. } => (),
        }
    }
    let mut declared = BTreeSet::new();
    for declaration in imports {
        declared.extend(
            declaration
                .bindings
                .iter()
                .map(|binding| JsString::from(binding.local.as_str())),
        );
    }
    let mut variables = Vec::new();
    for statement in statements {
        declared.extend(block_lexical_names(statement).map(|(name, _)| JsString::from(name)));
        variables.clear();
        statement.collect_var_declarations(&mut variables);
        declared.extend(variables.iter().map(|binding| JsString::from(binding.name)));
    }
    for declaration in exports {
        if let ExportKind::Named {
            specifiers,
            request: None,
        } = &declaration.kind
        {
            for entry in specifiers {
                if !declared.contains(&entry.name.value) {
                    return Err(early(
                        entry.name.span,
                        "exported binding is not declared in the module",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn insert(names: &mut BTreeSet<JsString>, name: &ModuleExportName) -> Result<(), Diagnostic> {
    if !names.insert(name.value.clone()) {
        return Err(early(name.span, "duplicate exported name"));
    }
    Ok(())
}
