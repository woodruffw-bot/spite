//! Strict directives and legacy-token early errors.

use crate::ast::{Expr, ExprKind, Literal, Statement, StatementKind};
use crate::early;
use crate::lexer::Token;
use spite_core::Diagnostic;

// ECMA-262 14.1: only unescaped, unparenthesized string directives count.
pub(crate) fn has_use_strict(statements: &[Statement], source: &str) -> bool {
    statements
        .iter()
        .take_while(|s| {
            matches!(
                s.kind,
                StatementKind::Expression(Expr {
                    kind: ExprKind::Literal(Literal::String(_)),
                    ..
                })
            )
        })
        .any(|s| {
            let StatementKind::Expression(expr) = &s.kind else {
                return false;
            };
            matches!(
                &source[expr.span.start..expr.span.end],
                "\"use strict\"" | "'use strict'"
            )
        })
}

pub(crate) fn reject_legacy_tokens(tokens: &[Token]) -> Result<(), Diagnostic> {
    // Strictness also applies to earlier directives and nested function code.
    if let Some(token) = tokens.iter().find(|token| token.legacy) {
        return Err(early(
            token.span,
            "legacy numeric literals and escapes are forbidden in strict code",
        ));
    }
    Ok(())
}
