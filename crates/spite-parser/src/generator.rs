//! YieldExpression grammar and context boundaries (15.5).

use super::*;

impl Parser {
    pub(super) fn yield_expression(&mut self) -> Result<Expr, Diagnostic> {
        if matches!(self.yield_context, YieldContext::Parameters) {
            return Err(self.error("yield is not allowed in generator parameters"));
        }
        let start = self.current().span.start;
        self.expect("yield")?;
        let bare = self.current().newline
            || matches!(self.current().kind, Kind::Eof)
            || [";", "}", ")", "]", ",", ":"]
                .iter()
                .any(|end| self.at(end));
        let delegated = !bare && self.eat("*");
        let argument = if bare {
            None
        } else {
            Some(Box::new(self.expression(2)?))
        };
        let end = argument
            .as_ref()
            .map_or(self.tokens[self.index - 1].span.end, |argument| {
                argument.span.end
            });
        self.make_expr(
            ExprKind::Yield {
                delegated,
                argument,
            },
            Span::new(start, end),
        )
    }
}
