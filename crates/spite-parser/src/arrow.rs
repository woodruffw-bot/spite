//! Arrow cover grammar for identifier parameters and concise bodies (15.3).

use super::*;

pub(super) type Head = std::rc::Rc<cover::Probe<std::rc::Rc<[Parameter]>>>;

impl Parser {
    // Refine the parameter grammar before deciding whether this is an arrow.
    // Defaults use the ordinary expression grammar and select their own goals.
    fn arrow_head(&mut self, start: usize) -> Result<Option<Head>, Diagnostic> {
        match self.token_at(start).map(|token| &token.kind) {
            Some(Kind::Word(_)) => {
                if !self
                    .token_at(start + 1)
                    .is_some_and(|token| token.kind == Kind::Punct("=>"))
                {
                    return Ok(None);
                }
            }
            Some(Kind::Punct("(")) => {}
            _ => return Ok(None),
        }
        let key = self.cover_key(start);
        if let Some(head) = self.arrow_heads.get(&key) {
            return Ok(head.clone());
        }
        let probe = self.probe_cover(|parser| {
            parser.index = start;
            let parameters = if parser.at("(") {
                parser.formal_parameters("invalid arrow binding identifier")?
            } else {
                vec![Parameter::Ordinary(parser.formal_parameter(
                    "invalid arrow binding identifier",
                    false,
                )?)]
                .into()
            };
            if !parser.at("=>") {
                return Err(parser.error("not an arrow head"));
            }
            Ok(parameters)
        });
        let head = match probe {
            Ok(head) => Some(head),
            Err(error) if error.kind == DiagnosticKind::Syntax => None,
            Err(error) => return Err(error),
        };
        self.arrow_heads.insert(key, head.clone());
        Ok(head)
    }

    pub(super) fn arrow_expression(&mut self) -> Result<Option<Expr>, Diagnostic> {
        if self.at("async")
            && self
                .token_at(self.index + 1)
                .is_some_and(|token| !token.newline)
            && self.arrow_head(self.index + 1)?.is_some()
        {
            return Err(self.unsupported("async arrow functions are not implemented"));
        }
        let Some(head) = self.arrow_head(self.index)? else {
            return Ok(None);
        };
        let start = self.current().span.start;
        if self.tokens[head.end].newline {
            return Err(early(
                self.tokens[head.end].span,
                "line terminator before arrow",
            ));
        }
        self.consume_cover(&head)?;
        let parameters = head.value.clone();
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
