//! Grammar-driven cover probes retain scanner results but isolate semantic uses.

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Key {
    index: usize,
    depth: usize,
    context: u8,
    private_depth: usize,
}

pub(super) struct Probe<T> {
    pub value: T,
    pub end: usize,
    private_uses: Vec<Vec<PrivateIdentifier>>,
    error: Option<Diagnostic>,
}

impl Parser {
    pub(super) fn cover_key(&self, index: usize) -> Key {
        let flags = [
            self.allow_in,
            self.allow_return,
            self.allow_new_target,
            self.allow_super_property,
            self.allow_super_call,
            self.allow_arguments,
            self.allow_await_identifier,
        ];
        Key {
            index,
            depth: self.depth,
            context: flags
                .iter()
                .enumerate()
                .fold(0, |bits, (i, set)| bits | (u8::from(*set) << i)),
            private_depth: self.private_scopes.len(),
        }
    }

    #[inline(never)]
    pub(super) fn probe_cover<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T, Diagnostic>,
    ) -> Result<std::rc::Rc<Probe<T>>, Diagnostic> {
        let start = self.index;
        let computed_name = self.computed_class_name.clone();
        let previous_error = self.lookahead_error.clone();
        let previous_scan_error = self.scan_error.clone();
        let private_uses = self.checkpoint_private_uses();
        let previous_probe = self.probing_cover;
        let previous_probe_error = self.probe_error.take();
        self.probing_cover = true;
        let result = parse(self);
        let end = self.index;
        let error = self.probe_error.take();
        let private_uses = self.restore_private_uses(&private_uses);
        self.index = start;
        self.computed_class_name = computed_name;
        self.probing_cover = previous_probe;
        self.probe_error = previous_probe_error;
        // A speculative grammar error belongs to this probe. A scanner error
        // remains authoritative, unless a RegExp rescan already replaced it.
        self.lookahead_error = if self.scan_error == previous_scan_error {
            previous_error
        } else {
            self.scan_error.clone()
        };
        result.map(|value| {
            std::rc::Rc::new(Probe {
                value,
                end,
                private_uses,
                error,
            })
        })
    }

    pub(super) fn probe_rest_continuation(&mut self, close: &str) -> Result<bool, Diagnostic> {
        if !self.probing_cover || self.at(close) {
            return Ok(false);
        }
        self.defer_cover_error(self.error(&format!("expected {close}")))?;
        if self.eat("=") {
            self.expression_with_in(2, true)?;
        }
        Ok(self.eat(","))
    }

    pub(super) fn defer_cover_error(&mut self, error: Diagnostic) -> Result<(), Diagnostic> {
        if !self.probing_cover {
            return Err(error);
        }
        if self.probe_error.is_none() {
            self.probe_error = Some(error);
        }
        Ok(())
    }

    pub(super) fn consume_cover<T>(&mut self, probe: &Probe<T>) -> Result<(), Diagnostic> {
        if let Some(error) = &probe.error {
            if !self.probing_cover {
                return Err(error.clone());
            }
            if self.probe_error.is_none() {
                self.probe_error = Some(error.clone());
            }
        }
        self.index = probe.end;
        self.replay_private_uses(&probe.private_uses);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_and_cached_heads_replay_each_private_use_once() {
        let mut parser = Parser::new("(x = this.#x) => x").unwrap();
        parser.inherit_private_names(&BTreeSet::from(["#x".into()]));
        let first = parser.arrow_expression().unwrap().unwrap();
        assert_eq!(parser.checkpoint_private_uses(), [1]);
        let memo_count = parser.arrow_heads.len();
        parser.restore_private_uses(&[0]);
        parser.index = 0;
        assert_eq!(parser.arrow_expression().unwrap().unwrap(), first);
        assert_eq!(parser.checkpoint_private_uses(), [1]);
        assert_eq!(parser.arrow_heads.len(), memo_count);
    }

    #[test]
    fn rejected_heads_restore_uses_and_computed_name_errors_but_keep_scanner_errors() {
        let mut parser = Parser::new("(x = this.#x)").unwrap();
        parser.inherit_private_names(&BTreeSet::from(["#x".into()]));
        assert!(parser.arrow_expression().unwrap().is_none());
        assert_eq!(parser.checkpoint_private_uses(), [0]);
        parser.prefix().unwrap();
        assert_eq!(parser.checkpoint_private_uses(), [1]);

        let mut parser = Parser::new("(x = class { get [1 +]() {} })").unwrap();
        assert!(parser.arrow_expression().unwrap().is_none());
        assert!(parser.lookahead_error.is_none());
        assert_eq!(parser.prefix().unwrap_err().kind, DiagnosticKind::Syntax);

        let mut parser = Parser::new("(x = 1 /*").unwrap();
        assert!(parser.arrow_expression().unwrap().is_none());
        assert_eq!(
            parser.finish(Ok(())).unwrap_err().message,
            "unterminated comment"
        );
    }

    #[test]
    fn nested_rejected_heads_reuse_contextual_results() {
        let n = 16;
        let source = format!("{}1{}", "(x=".repeat(n), ")".repeat(n));
        let mut parser = Parser::new(&source).unwrap();
        assert!(parser.arrow_expression().unwrap().is_none());
        parser.prefix().unwrap();
        // Each position can be revisited at a different grammar depth. Results
        // at the same depth/context are shared rather than branching again.
        assert!(parser.arrow_heads.len() <= n * n);
        assert!(parser.finish(Ok(())).is_ok());
    }

    #[test]
    fn selected_patterns_replay_private_uses_once_and_unselected_patterns_restore_them() {
        let mut parser = Parser::new("{[this.#x]: x}=source").unwrap();
        parser.inherit_private_names(&BTreeSet::from(["#x".into()]));
        let cover = parser.pattern_cover().unwrap().unwrap();
        assert_eq!(parser.checkpoint_private_uses(), [0]);
        let first = parser.consume_pattern_cover(&cover).unwrap();
        assert_eq!(parser.checkpoint_private_uses(), [1]);
        parser.restore_private_uses(&[0]);
        parser.index = 0;
        let cached = parser.pattern_cover().unwrap().unwrap();
        assert_eq!(parser.consume_pattern_cover(&cached).unwrap(), first);
        assert_eq!(parser.checkpoint_private_uses(), [1]);

        let mut parser = Parser::new("{[this.#x]: x}.p").unwrap();
        parser.inherit_private_names(&BTreeSet::from(["#x".into()]));
        assert!(parser.pattern_cover().unwrap().is_some());
        assert_eq!(parser.checkpoint_private_uses(), [0]);
        parser.expression(3).unwrap();
        assert_eq!(parser.checkpoint_private_uses(), [1]);
    }

    #[test]
    fn regexp_rescans_invalidate_both_arrow_and_pattern_memo_tables() {
        let mut parser = Parser::new("[old]; (x)=>x; [y=/[}]/]=source").unwrap();
        parser.expression(1).unwrap();
        parser.expect(";").unwrap();
        assert!(!parser.pattern_covers.is_empty());
        parser.arrow_expression().unwrap().unwrap();
        parser.expect(";").unwrap();
        assert!(!parser.arrow_heads.is_empty());
        let cover = parser.pattern_cover().unwrap().unwrap();
        assert!(parser.arrow_heads.is_empty());
        assert_eq!(parser.pattern_covers.len(), 1);
        parser.consume_pattern_cover(&cover).unwrap();
        assert!(parser.at("="));
        assert!(parser.scan_error.is_none());
    }

    #[test]
    fn nested_pattern_fallbacks_reuse_contextual_probe_results() {
        let n = 16;
        let source = format!("{}1{}", "[x=".repeat(n), "]".repeat(n));
        let mut parser = Parser::new(&source).unwrap();
        assert!(parser.pattern_cover().unwrap().is_some());
        parser.expression(3).unwrap();
        assert!(parser.pattern_covers.len() <= n * n);
        assert!(parser.finish(Ok(())).is_ok());
    }
}
