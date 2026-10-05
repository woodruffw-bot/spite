//! Literal flag early errors before the still-unimplemented Pattern grammar.

use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};

pub(super) fn literal_diagnostic(flags: &JsString, span: Span) -> Diagnostic {
    // https://262.ecma-international.org/17.0/#sec-isvalidregularexpressionliteral
    // checks flags and duplicates before #sec-parsepattern rejects simultaneous
    // Unicode modes.
    const FLAGS: &[u8] = b"dgimsuvy";
    const UNICODE_MODES: u8 = (1 << 5) | (1 << 6);
    let mut seen = 0u8;
    for unit in flags.code_units() {
        let Some(index) = FLAGS.iter().position(|flag| u16::from(*flag) == *unit) else {
            return Diagnostic::new(
                DiagnosticKind::Syntax,
                span,
                "invalid regular expression flag",
            );
        };
        let bit = 1u8 << index;
        if seen & bit != 0 {
            return Diagnostic::new(
                DiagnosticKind::Syntax,
                span,
                "duplicate regular expression flag",
            );
        }
        seen |= bit;
    }
    if seen & UNICODE_MODES == UNICODE_MODES {
        return Diagnostic::new(
            DiagnosticKind::Syntax,
            span,
            "regular expression flags u and v are mutually exclusive",
        );
    }
    Diagnostic::new(
        DiagnosticKind::Unsupported,
        span,
        "regular expression pattern validation and matching are not implemented",
    )
}
