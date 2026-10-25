//! Owned value-expression continuations with safe paths into retained source AST.

use super::async_execution::AsyncAwaitAction;
use crate::{Value, function::CallTarget};
use spite_core::Span;
use spite_parser::ast::{BinaryOp, UnaryOp};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ExprRoot {
    Arrow,
    Statement,
    IfTest,
    LoopTest,
    Binding { element: usize },
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
    Callee,
    Argument(usize),
    ArrayElement(usize),
    MemberBase,
    MemberKey,
}

pub(crate) type ExprPath = Vec<ExprChild>;

#[derive(Debug)]
pub(crate) enum InvocationTarget {
    Call(CallTarget),
    Construct(Value),
}

impl InvocationTarget {
    fn values(&self) -> impl Iterator<Item = &Value> {
        let (value, call) = match self {
            Self::Call(call) => (None, Some(call)),
            Self::Construct(value) => (Some(value), None),
        };
        value
            .into_iter()
            .chain(call.into_iter().flat_map(CallTarget::values))
    }
}

#[derive(Debug)]
pub(crate) enum MemberAction {
    Value,
    Call { path: ExprPath, span: Span },
    Delete { span: Span },
}

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
    Callee {
        path: ExprPath,
        span: Span,
    },
    ArgumentNext {
        path: ExprPath,
        index: usize,
        target: InvocationTarget,
        arguments: Vec<Value>,
        span: Span,
    },
    ArgumentAppend {
        path: ExprPath,
        index: usize,
        target: InvocationTarget,
        arguments: Vec<Value>,
        span: Span,
    },
    ArrayNext {
        path: ExprPath,
        element: usize,
        index: u64,
        array: Value,
        span: Span,
    },
    ArrayAppend {
        path: ExprPath,
        element: usize,
        index: u64,
        array: Value,
        span: Span,
    },
    MemberBase {
        path: ExprPath,
        action: MemberAction,
        span: Span,
    },
    MemberKey {
        base: Value,
        action: MemberAction,
        span: Span,
    },
    CheckString(Span),
}

impl ExprTask {
    fn values(&self) -> impl Iterator<Item = &Value> {
        let (value, target, arguments): (Option<&Value>, Option<&InvocationTarget>, &[Value]) =
            match self {
                Self::BinaryApply { left, .. } | Self::ChainApply { left, .. } => {
                    (Some(left), None, &[])
                }
                Self::ArrayNext { array, .. } | Self::ArrayAppend { array, .. } => {
                    (Some(array), None, &[])
                }
                Self::MemberKey { base, .. } => (Some(base), None, &[]),
                Self::ArgumentNext {
                    target, arguments, ..
                }
                | Self::ArgumentAppend {
                    target, arguments, ..
                } => (None, Some(target), arguments),
                _ => (None, None, &[]),
            };
        value
            .into_iter()
            .chain(target.into_iter().flat_map(InvocationTarget::values))
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
