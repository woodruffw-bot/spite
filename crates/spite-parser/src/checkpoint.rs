//! Cached tokens retain the scanner and parser template context before scanning.

use super::*;

#[derive(Clone)]
pub(super) struct ScanCheckpoint {
    lexer: lexer::Checkpoint,
    template_braces: Vec<usize>,
}

impl ScanCheckpoint {
    pub(super) fn new(lexer: &Lexer, template_braces: &[usize]) -> Self {
        Self {
            lexer: lexer.checkpoint(),
            template_braces: template_braces.to_vec(),
        }
    }
}

impl Parser {
    pub(super) fn cache_next_token(&mut self, regexp: bool) -> Result<(), Diagnostic> {
        self.scan_checkpoints
            .push(ScanCheckpoint::new(&self.lexer, &self.template_braces));
        match self.scan_token(regexp) {
            Ok(token) => {
                self.tokens.push(token);
                Ok(())
            }
            Err(error) => {
                self.tokens.push(Token {
                    kind: Kind::Eof,
                    span: error.span,
                    newline: false,
                    escaped: false,
                    legacy: false,
                });
                self.lookahead_error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub(super) fn rescan_regexp(&mut self) -> Result<(), Diagnostic> {
        // ECMA-262 12 selects RegExp at this primary-expression solidus. Restore
        // trivia and template state before the token, discard the Div-goal
        // suffix, and retain the replacement token in the ordinary cache.
        let checkpoint = self.scan_checkpoints[self.index].clone();
        self.lexer.restore(checkpoint.lexer);
        self.template_braces = checkpoint.template_braces;
        self.tokens.truncate(self.index);
        self.scan_checkpoints.truncate(self.index);
        if self
            .computed_class_name
            .as_ref()
            .is_some_and(|(_, end, _)| *end > self.index)
        {
            self.computed_class_name = None;
        }
        self.cache_next_token(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regexp_rescan_replaces_a_failed_div_suffix_and_preserves_trivia() {
        let mut parser = Parser::new("(x =\n /\\)/g)\n+ 1").unwrap();
        assert!(parser.arrow_expression().unwrap().is_none());
        assert!(parser.lookahead_error.is_some());
        for _ in 0..3 {
            parser.bump();
        }
        assert_eq!(parser.current().kind, Kind::Punct("/"));
        let diagnostic = parser.regexp_diagnostic();
        assert_eq!(diagnostic.kind, DiagnosticKind::Unsupported);
        assert!(parser.current().newline);
        assert!(matches!(parser.current().kind, Kind::RegExp { .. }));
        assert!(parser.lookahead_error.is_none());
        parser.bump();
        assert!(parser.at(")"));
        parser.bump();
        assert!(parser.at("+"));
        assert!(parser.current().newline);
        parser.bump();
        assert!(matches!(
            parser.current().kind,
            Kind::Literal(Literal::Number(1.0))
        ));
        parser.bump();
        assert_eq!(parser.current().kind, Kind::Eof);
        assert_eq!(parser.scan_checkpoints.len(), parser.tokens.len());
        assert!(parser.finish(Ok(())).is_ok());
    }

    #[test]
    fn regexp_rescan_restores_template_nesting_before_cached_literal_contents() {
        let mut parser = Parser::new("`outer${ { key: /[}`]/g } }tail` + 1").unwrap();
        while !parser.at("/") {
            parser.bump();
        }
        let index = parser.index;
        let _ = parser.token_at(index + 100);
        let diagnostic = parser.regexp_diagnostic();
        assert_eq!(diagnostic.kind, DiagnosticKind::Unsupported);
        assert_eq!(parser.template_braces, [1]);
        parser.bump();
        assert!(parser.at("}"));
        assert_eq!(parser.template_braces, [0]);
        parser.bump();
        let Kind::Template {
            element,
            tail: true,
            continuation: true,
        } = &parser.current().kind
        else {
            panic!("expected outer template tail");
        };
        assert_eq!(element.raw, JsString::from("tail"));
        assert!(parser.template_braces.is_empty());
        parser.bump();
        assert!(parser.at("+"));
        assert_eq!(parser.scan_checkpoints.len(), parser.tokens.len());
        assert!(parser.finish(Ok(())).is_ok());
    }

    #[test]
    fn failed_regexp_rescans_keep_an_authoritative_error_and_eof_sentinel() {
        let mut parser = Parser::new("(x = /a\n/)").unwrap();
        assert!(parser.arrow_expression().unwrap().is_none());
        for _ in 0..3 {
            parser.bump();
        }
        let error = parser.regexp_diagnostic();
        assert_eq!(error.kind, DiagnosticKind::Syntax);
        assert_eq!(parser.current().kind, Kind::Eof);
        assert_eq!(parser.finish(Ok(())).unwrap_err(), error);
        assert_eq!(parser.scan_checkpoints.len(), parser.tokens.len());
    }
}
