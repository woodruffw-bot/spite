//! Statement completion records and control transfers.

use crate::Value;
use crate::function;

// Implemented statement completions. Throws already carry a non-empty value in
// Error::Thrown; host failures remain separate from these control transfers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompletionKind {
    Normal,
    Break,
    Continue,
    Return,
}

#[derive(Debug)]
pub(crate) struct Completion {
    pub(crate) kind: CompletionKind,
    // None is empty, distinct from Some(Value::Undefined).
    pub(crate) value: Option<Value>,
    pub(crate) target: Option<String>,
    // Synchronous source returns only. Suspended drivers never create transfers.
    pub(crate) tail_call: Option<Box<function::TailCall>>,
}

impl Completion {
    pub(crate) fn normal(value: Option<Value>) -> Self {
        Self {
            kind: CompletionKind::Normal,
            value,
            target: None,
            tail_call: None,
        }
    }

    // ECMA-262 14.7.1.1 LoopContinues.
    pub(crate) fn loop_continues(&self, labels: &[&str]) -> bool {
        self.kind == CompletionKind::Normal
            || (self.kind == CompletionKind::Continue
                && self
                    .target
                    .as_deref()
                    .is_none_or(|target| labels.contains(&target)))
    }

    // ECMA-262 14.13.4: loops and switches consume only unlabelled breaks.
    pub(crate) fn consume_unlabelled_break(mut self) -> Self {
        if self.kind == CompletionKind::Break && self.target.is_none() {
            self.kind = CompletionKind::Normal;
        }
        self
    }

    // ECMA-262 6.2.4.4 UpdateEmpty preserves the completion's kind.
    pub(crate) fn update_empty(mut self, value: Option<Value>) -> Self {
        self.value = self.value.or(value);
        self
    }
}
