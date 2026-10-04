//! Arrow cover grammar for simple parameters and expression bodies (15.3).

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
        let parenthesized = self.eat("(");
        let mut parameters = Vec::new();
        if !parenthesized || !self.at(")") {
            loop {
                if self.at("...") || self.at("[") || self.at("{") {
                    return Err(
                        self.unsupported("rest and binding-pattern parameters are not implemented")
                    );
                }
                let token = self.bump();
                let Kind::Word(name) = token.kind else {
                    return Err(early(token.span, "invalid arrow parameter"));
                };
                if reserved(&name) {
                    return Err(early(token.span, "invalid arrow binding identifier"));
                }
                parameters.push(Binding {
                    name,
                    span: token.span,
                    initializer: None,
                });
                if self.at("=") {
                    return Err(self.unsupported("default parameters are not implemented"));
                }
                if !parenthesized || !self.eat(",") || self.at(")") {
                    break;
                }
            }
        }
        if parenthesized {
            self.expect(")")?;
        }
        self.expect("=>")?;
        if self.at("{") {
            return Err(self.unsupported("arrow block bodies are not implemented"));
        }
        let body = self.expression(2)?;
        let span = Span::new(start, body.span.end);
        let source = FunctionSource {
            text: self.source.clone(),
            span,
        };
        self.make_expr(
            ExprKind::Arrow {
                parameters: parameters.into(),
                body: std::rc::Rc::new(body),
                source,
            },
            span,
        )
        .map(Some)
    }
}
