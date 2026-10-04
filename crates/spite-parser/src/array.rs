//! Array literals, elisions, and spread (13.2.4).

use crate::{ArrayElement, Diagnostic, Expr, ExprKind, Parser, Span};

impl Parser {
    pub(super) fn array_literal(&mut self, start: usize) -> Result<Expr, Diagnostic> {
        let mut elements = Vec::new();
        while !self.at("]") {
            if self.eat(",") {
                elements.push(ArrayElement::Elision);
                continue;
            }
            let spread = self.eat("...");
            // ElementList permits AssignmentExpression[+In], not an ungrouped
            // comma expression. The separator after an element adds no hole.
            let expression = self.expression_with_in(2, true)?;
            elements.push(if spread {
                ArrayElement::Spread(expression)
            } else {
                ArrayElement::Expression(expression)
            });
            if !self.eat(",") {
                break;
            }
        }
        self.expect("]")?;
        let end = self.tokens[self.index - 1].span.end;
        self.make_expr(ExprKind::Array(elements), Span::new(start, end))
    }
}
