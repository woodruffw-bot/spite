//! Operator precedence and assignment classification.

use crate::ast::{BinaryOp, Expr, ExprKind};
use crate::lexer::Kind;

pub(crate) fn binary(kind: &Kind) -> Option<(BinaryOp, u8)> {
    use BinaryOp::*;
    let Kind::Punct(p) = kind else {
        return None;
    };
    Some(match *p {
        "," => (Comma, 1),
        "??" => (Nullish, 4),
        "||" => (Or, 4),
        "&&" => (And, 5),
        "|" => (BitOr, 6),
        "^" => (BitXor, 7),
        "&" => (BitAnd, 8),
        "==" => (Equal, 9),
        "!=" => (NotEqual, 9),
        "===" => (StrictEqual, 9),
        "!==" => (StrictNotEqual, 9),
        "<" => (Less, 10),
        "<=" => (LessEqual, 10),
        ">" => (Greater, 10),
        ">=" => (GreaterEqual, 10),
        "<<" => (LeftShift, 11),
        ">>" => (RightShift, 11),
        ">>>" => (UnsignedRightShift, 11),
        "+" => (Add, 12),
        "-" => (Subtract, 12),
        "*" => (Multiply, 13),
        "/" => (Divide, 13),
        "%" => (Remainder, 13),
        "**" => (Exponentiate, 14),
        _ => return None,
    })
}

pub(crate) fn compound_assignment(kind: &Kind) -> Option<BinaryOp> {
    use BinaryOp::*;
    let Kind::Punct(p) = kind else {
        return None;
    };
    Some(match *p {
        "+=" => Add,
        "-=" => Subtract,
        "*=" => Multiply,
        "/=" => Divide,
        "%=" => Remainder,
        "**=" => Exponentiate,
        "<<=" => LeftShift,
        ">>=" => RightShift,
        ">>>=" => UnsignedRightShift,
        "&=" => BitAnd,
        "^=" => BitXor,
        "|=" => BitOr,
        "&&=" => And,
        "||=" => Or,
        "??=" => Nullish,
        _ => return None,
    })
}

pub(crate) fn forbidden_nullish_mix(op: BinaryOp, expr: &Expr) -> bool {
    let other = match &expr.kind {
        ExprKind::Binary(other, ..) => *other,
        ExprKind::BinaryChain { steps, .. } => match steps.last() {
            Some(step) => step.op,
            None => return false,
        },
        _ => return false,
    };
    (op == BinaryOp::Nullish && matches!(other, BinaryOp::And | BinaryOp::Or))
        || (other == BinaryOp::Nullish && matches!(op, BinaryOp::And | BinaryOp::Or))
}

// ReservedWord StringValues other than the context-sensitive yield and await.
// https://262.ecma-international.org/17.0/#sec-identifiers-static-semantics-early-errors
