//! Owned value-expression continuations with safe paths into retained source AST.

use super::async_execution::AsyncAwaitAction;
use crate::{Reference, Value, environment::EnvironmentHandle, function::CallTarget};
use spite_core::{JsSymbol, Span};
use spite_heap::Handle;
use spite_parser::ast::{BinaryOp, UnaryOp, UpdateOp};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ExprRoot {
    Arrow,
    Statement,
    IfTest,
    LoopTest,
    ForHeader,
    IterationSource,
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
    Update {
        op: UpdateOp,
        prefix: bool,
        span: Span,
        argument_span: Span,
    },
    Assignment {
        path: ExprPath,
        op: Option<BinaryOp>,
        span: Span,
    },
    Call {
        path: ExprPath,
        span: Span,
    },
    Delete {
        span: Span,
    },
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
    AssignmentApply {
        reference: Reference<'static>,
        left: Option<Value>,
        op: Option<BinaryOp>,
        span: Span,
    },
    CheckString(Span),
}

impl ExprTask {
    fn reference(&self) -> Option<&Reference<'static>> {
        match self {
            Self::AssignmentApply { reference, .. } => Some(reference),
            _ => None,
        }
    }

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
                Self::AssignmentApply { left, .. } => (left.as_ref(), None, &[]),
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
            .chain(self.reference().into_iter().flat_map(Reference::values))
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
    pub(super) fn symbols(&self) -> impl Iterator<Item = &JsSymbol> {
        self.tasks
            .iter()
            .filter_map(ExprTask::reference)
            .flat_map(Reference::symbols)
    }

    pub(super) fn environments(&self) -> impl Iterator<Item = &EnvironmentHandle> {
        self.tasks
            .iter()
            .filter_map(ExprTask::reference)
            .filter_map(Reference::environment)
    }

    pub(super) fn objects(&self) -> impl Iterator<Item = &Handle> {
        self.tasks
            .iter()
            .filter_map(ExprTask::reference)
            .filter_map(Reference::object)
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.current
            .iter()
            .chain(self.tasks.iter().flat_map(ExprTask::values))
    }
}
