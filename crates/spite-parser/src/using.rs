//! Contextual synchronous resource declarations (14.3.1, 14.7.4–5, 16.1.1).

use super::*;

impl Parser {
    pub(super) fn at_using_declaration(&mut self, for_header: bool) -> bool {
        if !self.at("using") || self.current().escaped {
            return false;
        }
        let Some(next) = self.token_at(self.index + 1) else {
            return false;
        };
        if next.newline || !matches!(next.kind, Kind::Word(_)) {
            return false;
        }
        if for_header && !next.escaped && matches!(&next.kind, Kind::Word(name) if name == "of") {
            // ForDeclaration's lookahead excludes `using of`. A three-clause
            // initializer can still declare `of` when followed by `=`.
            return self
                .token_at(self.index + 2)
                .is_some_and(|token| token.kind == Kind::Punct("="));
        }
        true
    }

    pub(super) fn using_bindings(
        &mut self,
        for_header: bool,
    ) -> Result<Vec<BindingElement>, Diagnostic> {
        self.expect("using")?;
        let mut bindings = Vec::new();
        loop {
            let pattern = self.binding_identifier()?;
            if matches!(&pattern.kind, BindingPatternKind::Identifier(name) if name == "let") {
                return Err(early(pattern.span, "invalid binding identifier"));
            }
            let initializer = if self.eat("=") {
                Some(self.expression_with_in(2, !for_header)?)
            } else {
                None
            };
            if initializer.is_none() && !(for_header && (self.at("of") || self.at("in"))) {
                return Err(self.error("using requires an initializer"));
            }
            bindings.push(BindingElement {
                pattern,
                initializer,
            });
            if !self.eat(",") {
                break;
            }
        }
        Ok(bindings)
    }
}

pub(super) fn reject_direct_using(
    statements: &[Statement],
    message: &str,
) -> Result<(), Diagnostic> {
    if let Some(statement) = statements
        .iter()
        .find(|statement| matches!(statement.kind, StatementKind::Using(_)))
    {
        return Err(early(
            Span::new(statement.span.start, statement.span.start + 5),
            message,
        ));
    }
    Ok(())
}
