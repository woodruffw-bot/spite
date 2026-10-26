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
    WithObject,
    SwitchDiscriminant,
    SwitchSelector,
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
    TemplateSubstitution(usize),
    ObjectKey(usize),
    ObjectValue(usize),
    ObjectSpread(usize),
    SuperKey,
    OptionalBase,
    OptionalKey(usize),
    OptionalArgument { step: usize, index: usize },
}

pub(crate) type ExprPath = Vec<ExprChild>;

#[derive(Debug)]
pub(crate) enum InvocationTarget {
    Call(CallTarget),
    Construct(Value),
    Super(crate::super_property::SuperCallTarget),
}

impl InvocationTarget {
    fn values(&self) -> impl Iterator<Item = &Value> {
        let (value, call, super_target) = match self {
            Self::Call(call) => (None, Some(call), None),
            Self::Construct(value) => (Some(value), None, None),
            Self::Super(target) => (None, None, Some(target)),
        };
        value
            .into_iter()
            .chain(call.into_iter().flat_map(CallTarget::values))
            .chain(
                super_target
                    .into_iter()
                    .flat_map(crate::super_property::SuperCallTarget::values),
            )
    }

    fn environment(&self) -> Option<&EnvironmentHandle> {
        match self {
            Self::Super(target) => Some(target.environment()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum MemberAction {
    Value,
    OptionalBase {
        path: ExprPath,
        action: Box<MemberAction>,
    },
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
pub(crate) struct OptionalCallCursor {
    pub path: ExprPath,
    pub step: usize,
    pub index: usize,
    pub target: InvocationTarget,
    pub arguments: Vec<Value>,
    pub action: MemberAction,
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
    TemplateNext {
        path: ExprPath,
        index: usize,
        units: Vec<u16>,
    },
    TemplateAppend {
        path: ExprPath,
        index: usize,
        units: Vec<u16>,
    },
    ObjectNext {
        path: ExprPath,
        element: usize,
        object: Value,
    },
    ObjectKey {
        path: ExprPath,
        element: usize,
        object: Value,
    },
    ObjectValue {
        path: ExprPath,
        element: usize,
        object: Value,
        key: Value,
    },
    ObjectSpread {
        path: ExprPath,
        element: usize,
        object: Value,
    },
    OptionalBase {
        path: ExprPath,
        action: MemberAction,
    },
    OptionalKey {
        path: ExprPath,
        step: usize,
        base: Value,
        action: MemberAction,
    },
    OptionalArgumentNext(OptionalCallCursor),
    OptionalArgumentAppend(OptionalCallCursor),
    SuperKey {
        this_value: Value,
        action: MemberAction,
        span: Span,
    },
    PrivateIn(ExprPath),
    DestructureApply(ExprPath),
    AssignmentApply {
        reference: Reference<'static>,
        left: Option<Value>,
        op: Option<BinaryOp>,
        span: Span,
    },
    CheckString(Span),
}

impl ExprTask {
    fn invocation_environment(&self) -> Option<&EnvironmentHandle> {
        match self {
            Self::ArgumentNext { target, .. } | Self::ArgumentAppend { target, .. } => {
                target.environment()
            }
            Self::OptionalArgumentNext(cursor) | Self::OptionalArgumentAppend(cursor) => {
                cursor.target.environment()
            }
            _ => None,
        }
    }

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
                Self::ObjectNext { object, .. }
                | Self::ObjectKey { object, .. }
                | Self::ObjectValue { object, .. }
                | Self::ObjectSpread { object, .. } => (Some(object), None, &[]),
                Self::MemberKey { base, .. }
                | Self::OptionalKey { base, .. }
                | Self::SuperKey {
                    this_value: base, ..
                } => (Some(base), None, &[]),
                Self::AssignmentApply { left, .. } => (left.as_ref(), None, &[]),
                Self::OptionalArgumentNext(cursor) | Self::OptionalArgumentAppend(cursor) => {
                    (None, Some(&cursor.target), &cursor.arguments)
                }
                Self::ArgumentNext {
                    target, arguments, ..
                }
                | Self::ArgumentAppend {
                    target, arguments, ..
                } => (None, Some(target), arguments),
                _ => (None, None, &[]),
            };
        let key = match self {
            Self::ObjectValue { key, .. } => Some(key),
            _ => None,
        };
        value
            .into_iter()
            .chain(key)
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
            .chain(
                self.tasks
                    .iter()
                    .filter_map(ExprTask::invocation_environment),
            )
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
