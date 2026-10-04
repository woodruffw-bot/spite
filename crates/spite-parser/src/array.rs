//! Array literals and elisions (13.2.4); spread awaits iterator semantics.

use crate::{Diagnostic, Expr, ExprKind, Parser, Span};

impl Parser {
    pub(super) fn array_literal(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        let mut elements = Vec::new();
        while !self.at("]") {
            if self.eat(",") {
                elements.push(None);
                continue;
            }
            if self.at("...") {
                return Err(self.unsupported("array spread is not implemented"));
            }
            // ElementList permits AssignmentExpression[+In], not an ungrouped
            // comma expression. The separator after an element adds no hole.
            elements.push(Some(self.expression_with_in(2, true)?));
            if !self.eat(",") {
                break;
            }
        }
        self.expect("]")?;
        let end = self.tokens[self.index - 1].span.end;
        self.make_expr(ExprKind::Array(elements), Span::new(start, end))
    }
}
