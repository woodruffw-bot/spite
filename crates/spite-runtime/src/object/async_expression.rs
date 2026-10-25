//! Owned value-expression continuations with safe paths into retained source AST.

use super::async_execution::AsyncAwaitAction;
use crate::{Value, function::CallTarget};
use spite_core::Span;
use spite_parser::ast::{BinaryOp, UnaryOp};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ExprRoot {
    Arrow,
    Statement(usize),
    Binding { statement: usize, element: usize },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ExprChild {
    Inner,
    Left,
    Right,
    ChainHead,
    ChainRight(usize),
    Test,
    Yes,
    No,
    CallCallee,
    CallArgument(usize),
}

pub(crate) type ExprPath = Vec<ExprChild>;

#[derive(Debug)]
pub(crate) enum ExprTask {
    Evaluate(ExprPath),
    Unary {
        op: UnaryOp,
        span: Span,
    },
    Await(Span),
    BinaryLeft {
        op: BinaryOp,
        right: ExprPath,
        span: Span,
    },
    BinaryApply {
        op: BinaryOp,
        left: Value,
        span: Span,
    },
    ChainNext {
        path: ExprPath,
        index: usize,
    },
    ChainApply {
        op: BinaryOp,
        left: Value,
        span: Span,
        path: ExprPath,
        next: usize,
    },
    Conditional {
        yes: ExprPath,
        no: ExprPath,
        span: Span,
    },
    CallCallee {
        path: ExprPath,
        span: Span,
    },
    CallNext {
        path: ExprPath,
        index: usize,
        target: CallTarget,
        arguments: Vec<Value>,
        span: Span,
    },
    CallAppend {
        path: ExprPath,
        index: usize,
        target: CallTarget,
        arguments: Vec<Value>,
        span: Span,
    },
    CheckString(Span),
}

impl ExprTask {
    fn values(&self) -> impl Iterator<Item = &Value> {
        let (left, target, arguments): (Option<&Value>, Option<&CallTarget>, &[Value]) = match self
        {
            Self::BinaryApply { left, .. } | Self::ChainApply { left, .. } => {
                (Some(left), None, &[])
            }
            Self::CallNext {
                target, arguments, ..
            }
            | Self::CallAppend {
                target, arguments, ..
            } => (None, Some(target), arguments),
            _ => (None, None, &[]),
        };
        left.into_iter()
            .chain(target.into_iter().flat_map(CallTarget::values))
            .chain(arguments)
    }
}

#[derive(Debug)]
pub(crate) struct AsyncExpressionData {
    pub root: ExprRoot,
    pub action: AsyncAwaitAction,
    pub tasks: Vec<ExprTask>,
    pub current: Option<Value>,
}

impl AsyncExpressionData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.current
            .iter()
            .chain(self.tasks.iter().flat_map(ExprTask::values))
    }
}
