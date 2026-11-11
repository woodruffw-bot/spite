//! Script and direct-eval parsing entry points.

use crate::ast::Script;
use crate::lexer::Kind;
use crate::{
    ControlContext, Parser, ScopeKind, has_use_strict, reject_legacy_tokens, validate_scope,
};
use crate::{source, using};
use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};
use std::collections::BTreeSet;

/// Parses a Script with an opted-in maximum UTF-8 source length.
pub fn parse_script_with_source_limit(
    source: &str,
    max_bytes: usize,
) -> Result<Script, Diagnostic> {
    if source.len() > max_bytes {
        return Err(Diagnostic::new(
            DiagnosticKind::Limit,
            Span::new(0, source.len()),
            "source size limit exceeded",
        ));
    }
    parse_script(source)
}

/// Parses a Script and validates implemented early errors before returning it.
///
/// Recognized syntax outside the implemented grammar produces
/// [`DiagnosticKind::Unsupported`].
pub fn parse_script(source: &str) -> Result<Script, Diagnostic> {
    let mut parser = Parser::new(source)?;
    let result = parse_script_contents(&mut parser);
    parser.finish(result)
}

/// Parses a Script from UTF-16, preserving lone surrogate source code points.
///
/// Scalar spans use UTF-8 lengths; each lone surrogate occupies three bytes.
/// Strictness comes from the Script's directive prologue. Return and new.target
/// remain invalid outside their own function contexts, including in arrows.
pub fn parse_script_utf16(source: &JsString) -> Result<Script, Diagnostic> {
    let mut parser =
        Parser::from_source(std::rc::Rc::new(source::SourceText::from_utf16(source)?))?;
    let result = parse_script_contents(&mut parser);
    parser.finish(result)
}

/// Parses lossless UTF-16 Script source with an opted-in encoded byte limit.
///
/// Scalars use their UTF-8 lengths; each lone surrogate occupies three bytes.
/// The limit is checked before allocating the parser's source representation.
pub fn parse_script_utf16_with_source_limit(
    source: &JsString,
    max_bytes: usize,
) -> Result<Script, Diagnostic> {
    let source = source::SourceText::from_utf16_with_limit(source, Some(max_bytes))?;
    let mut parser = Parser::from_source(std::rc::Rc::new(source))?;
    let result = parse_script_contents(&mut parser);
    parser.finish(result)
}

/// Caller context used by direct eval's Script early errors (19.2.1.1).
///
/// Arrows inherit the nearest non-arrow function's new.target and super context.
/// Private names are supplied separately to [`parse_eval_utf16_with_private_names`].
#[derive(Clone, Copy, Debug, Default)]
pub struct EvalContext {
    /// Whether the direct caller executes strict code.
    pub strict: bool,
    /// Whether GetThisEnvironment identifies a non-arrow function environment.
    pub in_function: bool,
    /// Whether that function environment has a super binding.
    pub in_method: bool,
    /// Whether that function is a derived class constructor.
    pub in_derived_constructor: bool,
    /// Whether the nearest non-arrow function is a class field initializer.
    pub in_class_field_initializer: bool,
}

/// Parses lossless UTF-16 eval code with inherited strictness and caller context.
///
/// Return remains invalid at the Script level, even when the caller is a function.
pub fn parse_eval_utf16(source: &JsString, context: EvalContext) -> Result<Script, Diagnostic> {
    parse_eval_utf16_with_private_names(source, context, &BTreeSet::new())
}

/// Parses eval code with the private identifier spellings visible to its caller.
///
/// Names include the leading `#`. This validates lexical availability only;
/// private identity and receiver brand checks belong to runtime evaluation.
pub fn parse_eval_utf16_with_private_names(
    source: &JsString,
    context: EvalContext,
    private_names: &BTreeSet<String>,
) -> Result<Script, Diagnostic> {
    let mut parser =
        Parser::from_source(std::rc::Rc::new(source::SourceText::from_utf16(source)?))?;
    parser.allow_new_target = context.in_function;
    parser.allow_super_property = context.in_method;
    parser.allow_super_call = context.in_derived_constructor;
    parser.allow_arguments = !context.in_class_field_initializer;
    parser.inherit_private_names(private_names);
    let result = parse_script_contents_with_strictness(&mut parser, context.strict);
    let script = parser.finish(result)?;
    let scope = parser.private_scopes.pop().expect("eval private scope");
    parser.finish_private_scope(scope)?;
    Ok(script)
}

pub(crate) fn parse_script_contents(parser: &mut Parser) -> Result<Script, Diagnostic> {
    parse_script_contents_with_strictness(parser, false)
}

pub(crate) fn parse_script_contents_with_strictness(
    parser: &mut Parser,
    inherited_strict: bool,
) -> Result<Script, Diagnostic> {
    let mut statements = Vec::new();
    while parser.current().kind != Kind::Eof {
        statements.push(parser.statement(true)?);
    }
    using::reject_direct_using(
        &statements,
        "using declaration is not allowed at Script level",
    )?;
    let strict = inherited_strict || has_use_strict(&statements, parser.source.lexical_text());
    if strict {
        reject_legacy_tokens(&parser.tokens)?;
    }
    validate_scope(
        &statements,
        strict,
        ControlContext::default(),
        &mut Vec::new(),
        ScopeKind::Variable,
    )?;
    Ok(Script { statements, strict })
}
