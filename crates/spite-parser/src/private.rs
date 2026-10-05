//! PrivateIdentifier spelling, lexical class scope, and early errors (15.7/16.1).

use crate::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct PrivateScope {
    declarations: BTreeMap<String, (bool, PropertyKind)>,
    uses: Vec<PrivateIdentifier>,
}

impl Parser {
    pub(super) fn private_identifier(&mut self) -> Result<PrivateIdentifier, Diagnostic> {
        let hash = self.bump();
        self.private_identifier_after_hash(hash)
    }

    pub(super) fn private_identifier_after_hash(
        &mut self,
        hash: Token,
    ) -> Result<PrivateIdentifier, Diagnostic> {
        debug_assert_eq!(hash.kind, Kind::Punct("#"));
        let token = self.bump();
        if hash.span.end != token.span.start {
            return Err(early(
                hash.span,
                "private identifier cannot contain whitespace",
            ));
        }
        let name = match token.kind {
            Kind::Word(name) => name,
            Kind::Literal(Literal::Null) => "null".into(),
            Kind::Literal(Literal::Boolean(value)) => if value { "true" } else { "false" }.into(),
            _ => return Err(early(token.span, "expected a private identifier name")),
        };
        Ok(PrivateIdentifier {
            name: format!("#{name}"),
            span: Span::new(hash.span.start, token.span.end),
        })
    }

    pub(super) fn declare_private_identifier(
        &mut self,
        name: &PrivateIdentifier,
        is_static: bool,
        kind: PropertyKind,
    ) -> Result<(), Diagnostic> {
        if name.name == "#constructor" {
            return Err(early(name.span, "invalid private class element name"));
        }
        let scope = self.private_scopes.last_mut().expect("class private scope");
        if let Some(&(previous_static, previous_kind)) = scope.declarations.get(&name.name) {
            if previous_static != is_static
                || !matches!(
                    (previous_kind, kind),
                    (PropertyKind::Getter, PropertyKind::Setter)
                        | (PropertyKind::Setter, PropertyKind::Getter)
                )
            {
                return Err(early(name.span, "duplicate private class element"));
            }
            // Mark the completed accessor pair so a third declaration fails.
            scope
                .declarations
                .insert(name.name.clone(), (is_static, PropertyKind::Data));
        } else {
            scope
                .declarations
                .insert(name.name.clone(), (is_static, kind));
        }
        Ok(())
    }

    pub(super) fn use_private_identifier(
        &mut self,
        name: &PrivateIdentifier,
    ) -> Result<(), Diagnostic> {
        let Some(scope) = self.private_scopes.last_mut() else {
            return Err(early(
                name.span,
                "private identifier is not declared in an enclosing class",
            ));
        };
        scope.uses.push(name.clone());
        Ok(())
    }

    pub(super) fn finish_private_scope(&mut self, scope: PrivateScope) -> Result<(), Diagnostic> {
        // Defer unresolved names to the containing class, whose declarations
        // may occur later. Functions do not break private lexical scope.
        for name in scope.uses {
            if !scope.declarations.contains_key(&name.name) {
                self.use_private_identifier(&name)?;
            }
        }
        Ok(())
    }
}

pub(super) fn is_private_reference(expression: &Expr) -> bool {
    match &expression.kind {
        ExprKind::Member(_, PropertyName::Private(_)) => true,
        ExprKind::OptionalChain { steps, .. } => steps.last().is_some_and(|step| {
            matches!(step.kind, ChainStepKind::Property(PropertyName::Private(_)))
        }),
        ExprKind::Parenthesized(inner) => is_private_reference(inner),
        _ => false,
    }
}
