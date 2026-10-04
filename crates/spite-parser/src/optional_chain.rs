//! Optional-chain syntax and flat continuation steps (13.3.10).

use crate::*;

impl Parser {
    pub(super) fn optional_chain_step(&mut self, left: Expr) -> Result<Expr, Diagnostic> {
        let start = self.bump().span.start;
        let kind = if self.at("(") {
            ChainStepKind::Call(self.arguments()?)
        } else if self.eat("[") {
            let expression = self.expression_with_in(1, true)?;
            self.expect("]")?;
            ChainStepKind::Property(PropertyName::Computed(Box::new(expression)))
        } else {
            if matches!(self.current().kind, Kind::Template { .. }) {
                return Err(self.error("optional chains cannot be tagged templates"));
            }
            let token = self.bump();
            let name = match token.kind {
                Kind::Word(name) => JsString::from(name.as_str()),
                Kind::Literal(Literal::Null) => JsString::from("null"),
                Kind::Literal(Literal::Boolean(value)) => {
                    JsString::from(if value { "true" } else { "false" })
                }
                _ => {
                    return Err(early(
                        token.span,
                        "expected an identifier name after optional dot",
                    ));
                }
            };
            ChainStepKind::Property(PropertyName::Literal(Literal::String(name)))
        };
        self.append_chain_step(
            left,
            ChainStep {
                optional: true,
                kind,
                span: Span::new(start, self.tokens[self.index - 1].span.end),
            },
        )
    }

    pub(super) fn append_chain_step(
        &self,
        left: Expr,
        step: ChainStep,
    ) -> Result<Expr, Diagnostic> {
        let span = Span::new(left.span.start, step.span.end);
        let (base, mut steps) = match left {
            Expr {
                kind: ExprKind::OptionalChain { base, steps },
                ..
            } => (base, steps),
            other => (Box::new(other), Vec::new()),
        };
        steps.try_reserve(1).map_err(|_| {
            Diagnostic::new(
                DiagnosticKind::Limit,
                span,
                "optional-chain syntax allocation failed",
            )
        })?;
        steps.push(step);
        self.make_expr(ExprKind::OptionalChain { base, steps }, span)
    }
}
