//! Arrow cover grammar for identifier parameters and concise bodies (15.3).

use super::*;

impl Parser {
    // Recognize a complete parameter cover before consuming it. Parenthesis
    // scans are bounded by the already bounded token stream and parser depth.
    fn arrow_head_end(&self, start: usize) -> Option<usize> {
        let token = self.tokens.get(start)?;
        let end = match &token.kind {
            Kind::Word(_) => start,
            Kind::Punct("(") => {
                let mut depth = 0usize;
                let mut end = None;
                for (index, token) in self.tokens.iter().enumerate().skip(start) {
                    match token.kind {
                        Kind::Punct("(") => depth += 1,
                        Kind::Punct(")") => {
                            depth -= 1;
                            if depth == 0 {
                                end = Some(index);
                                break;
                            }
                        }
                        Kind::Eof => break,
                        _ => {}
                    }
                }
                end?
            }
            _ => return None,
        };
        (self.tokens.get(end + 1)?.kind == Kind::Punct("=>")).then_some(end)
    }

    pub(super) fn arrow_expression(&mut self) -> Result<Option<Expr>, Diagnostic> {
        if self.at("async")
            && !self.tokens[self.index + 1].newline
            && self.arrow_head_end(self.index + 1).is_some()
        {
            return Err(self.unsupported("async arrow functions are not implemented"));
        }
        let Some(end) = self.arrow_head_end(self.index) else {
            return Ok(None);
        };
        if self.tokens[end + 1].newline {
            return Err(early(
                self.tokens[end + 1].span,
                "line terminator before arrow",
            ));
        }
        let start = self.current().span.start;
        let parameters = if self.at("(") {
            self.formal_parameters("invalid arrow binding identifier")?
        } else {
            vec![Parameter::Ordinary(self.formal_parameter(
                "invalid arrow binding identifier",
                false,
            )?)]
            .into()
        };
        self.expect("=>")?;
        // Arrow parameters inherit Await, but concise/block bodies use ~Await
        // even within static initialization (15.3 grammar).
        let previous_await = self.allow_await_identifier;
        self.allow_await_identifier = true;
        let result = (|| {
            Ok(if self.at("{") {
                let body = self.function_body()?;
                (ArrowBody::Block(body), self.tokens[self.index - 1].span.end)
            } else {
                let body = self.expression(2)?;
                let end = body.span.end;
                (ArrowBody::Expression(std::rc::Rc::new(body)), end)
            })
        })();
        self.allow_await_identifier = previous_await;
        let (body, end) = result?;
        let span = Span::new(start, end);
        let source = FunctionSource {
            text: self.source.clone(),
            span,
        };
        self.make_expr(
            ExprKind::Arrow {
                parameters,
                body,
                source,
            },
            span,
        )
        .map(Some)
    }
}
