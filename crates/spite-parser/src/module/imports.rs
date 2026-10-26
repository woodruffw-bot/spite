//! ImportDeclaration, ImportEntries and WithClauseToAttributes (16.2.2).

use super::*;

impl Parser {
    pub(super) fn import_declaration(&mut self) -> Result<ImportDeclaration, Diagnostic> {
        let start = self.bump().span.start;
        let mut bindings = Vec::new();
        let specifier = if matches!(self.current().kind, Kind::Literal(Literal::String(_))) {
            self.import_string()?
        } else {
            if !self.at("*") && !self.at("{") {
                let binding = self.import_local(ImportName::Name(JsString::from("default")))?;
                bindings.push(binding);
                if self.eat(",") {
                    self.import_clause_tail(&mut bindings)?;
                }
            } else {
                self.import_clause_tail(&mut bindings)?;
            }
            self.expect("from")?;
            self.import_string()?
        };
        let attributes = self.import_attributes()?;
        self.semicolon()?;
        let end = self.tokens[self.index - 1].span.end;
        Ok(ImportDeclaration {
            request: ModuleRequest {
                specifier,
                attributes,
            },
            bindings,
            span: Span::new(start, end),
        })
    }

    fn import_clause_tail(&mut self, bindings: &mut Vec<ImportBinding>) -> Result<(), Diagnostic> {
        if self.eat("*") {
            self.expect("as")?;
            bindings.push(self.import_local(ImportName::Namespace)?);
            return Ok(());
        }
        if !self.at("{") {
            return Err(self.error("expected namespace or named imports"));
        }
        self.bump();
        while !self.at("}") {
            let aliased = matches!(self.current().kind, Kind::Literal(Literal::String(_)))
                || self.token_at(self.index + 1).is_some_and(|token| {
                    !token.escaped && matches!(&token.kind, Kind::Word(name) if name == "as")
                });
            if aliased {
                let (name, span) = self.module_name()?;
                if std::char::decode_utf16(name.code_units().iter().copied())
                    .any(|unit| unit.is_err())
                {
                    return Err(early(span, "module export name is not well-formed Unicode"));
                }
                self.expect("as")?;
                bindings.push(self.import_local(ImportName::Name(name))?);
            } else {
                let pattern = self.binding_identifier()?;
                let BindingPatternKind::Identifier(local) = pattern.kind else {
                    unreachable!("imported binding identifier")
                };
                validate_binding_name(&local, pattern.span, true)?;
                bindings.push(ImportBinding {
                    imported: ImportName::Name(JsString::from(local.as_str())),
                    local,
                    span: pattern.span,
                });
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        Ok(())
    }

    fn import_local(&mut self, imported: ImportName) -> Result<ImportBinding, Diagnostic> {
        let pattern = self.binding_identifier()?;
        let BindingPatternKind::Identifier(local) = pattern.kind else {
            unreachable!("imported binding identifier")
        };
        validate_binding_name(&local, pattern.span, true)?;
        Ok(ImportBinding {
            imported,
            local,
            span: pattern.span,
        })
    }

    fn import_string(&mut self) -> Result<JsString, Diagnostic> {
        let token = self.bump();
        if let Kind::Literal(Literal::String(value)) = token.kind {
            Ok(value)
        } else {
            Err(early(token.span, "expected module string literal"))
        }
    }

    // IdentifierName includes reserved keywords and the tokens the scanner
    // classifies as null/Boolean literals. Numeric literals are excluded.
    fn module_name(&mut self) -> Result<(JsString, Span), Diagnostic> {
        let token = self.bump();
        let value = match token.kind {
            Kind::Word(name) => JsString::from(name.as_str()),
            Kind::Literal(Literal::String(value)) => value,
            Kind::Literal(Literal::Null) => JsString::from("null"),
            Kind::Literal(Literal::Boolean(value)) => {
                JsString::from(if value { "true" } else { "false" })
            }
            _ => return Err(early(token.span, "expected module name or string literal")),
        };
        Ok((value, token.span))
    }

    fn import_attributes(&mut self) -> Result<Vec<ImportAttribute>, Diagnostic> {
        let mut attributes = Vec::new();
        if !self.eat("with") {
            return Ok(attributes);
        }
        self.expect("{")?;
        let mut keys = BTreeSet::new();
        while !self.at("}") {
            let (key, span) = self.module_name()?;
            if !keys.insert(key.clone()) {
                return Err(early(span, "duplicate import attribute key"));
            }
            self.expect(":")?;
            let value = self.import_string()?;
            attributes.push(ImportAttribute { key, value });
            if !self.eat(",") {
                break;
            }
        }
        self.expect("}")?;
        // WithClauseToAttributes canonicalizes UTF-16 keys before host requests.
        attributes.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        Ok(attributes)
    }
}
