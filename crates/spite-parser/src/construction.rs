//! Arguments and new-expression precedence (13.3.1, 13.3.5, 13.3.8).

use crate::{Diagnostic, Expr, ExprKind, Parser, Span, early, member_base};

impl Parser {
    pub(super) fn arguments(&mut self) -> Result<Vec<Expr>, Diagnostic> {
        self.expect("(")?;
        let mut arguments = Vec::new();
        while !self.at(")") {
            if self.at("...") {
                return Err(self.unsupported("spread arguments are not implemented"));
            }
            arguments.push(self.expression_with_in(2, true)?);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        Ok(arguments)
    }

    pub(super) fn new_expression(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.bump().span.start;
        if self.eat(".") {
            if !self.at("target") {
                return Err(self.error("expected target after new dot"));
            }
            let span = Span::new(start, self.bump().span.end);
            // Script Contains NewTarget crosses arrows but not ordinary functions
            // (16.1.1). An ordinary function's parameters enable it as well.
            if !self.allow_new_target {
                return Err(early(
                    span,
                    "new.target requires an enclosing non-arrow function",
                ));
            }
            return self.make_expr(ExprKind::NewTarget, span);
        }
        // Member access binds inside a constructor expression, but calls do not:
        // new F.x(a).y() constructs F.x before reading/calling y. Recursive new
        // expressions consume their own argument list first: new new F()().
        let callee = self.expression(18)?;
        if !member_base(&callee) && !matches!(callee.kind, ExprKind::New { .. }) {
            return Err(early(callee.span, "new requires a constructor expression"));
        }
        let arguments = if self.at("(") {
            Some(self.arguments()?)
        } else {
            None
        };
        let end = self.tokens[self.index - 1].span.end;
        self.make_expr(
            ExprKind::New {
                callee: Box::new(callee),
                arguments,
            },
            Span::new(start, end),
        )
    }
}
