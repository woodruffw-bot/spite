//! ImportCall argument grammar, source ranges and lexical goals (13.3.10).

use crate::{Diagnostic, Expr, ExprKind, Parser, Span};

impl Parser {
    pub(super) fn import_call(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.bump().span.start;
        self.expect("(")?;
        let specifier = Box::new(self.expression_with_in(2, true)?);
        let options = if self.eat(",") && !self.at(")") {
            let options = Box::new(self.expression_with_in(2, true)?);
            self.eat(",");
            Some(options)
        } else {
            None
        };
        self.expect(")")?;
        let end = self.tokens[self.index - 1].span.end;
        self.make_expr(
            ExprKind::ImportCall { specifier, options },
            Span::new(start, end),
        )
    }
}
