//! Separate dynamic Function grammar goals and combined early errors (20.2.1.1.1).

use super::*;
use std::rc::Rc;

/// Parses ordinary dynamic Function source from a joined parameter list and body.
///
/// The parameter list and body must each form their own grammar goal before
/// validating the combined function. It retains the standard `anonymous` source text
/// and derives strictness from its body, independently of the caller.
/// Unsupported syntax and the parser's native-stack limit remain distinct from
/// syntax errors. Inputs use the same UTF-8 source representation as Scripts.
pub fn parse_dynamic_function(parameters: &str, body: &str) -> Result<Rc<Function>, Diagnostic> {
    parse_sources(
        source::SourceText::from_str(parameters),
        source::SourceText::from_str(body),
        FunctionKind::Ordinary,
    )
}

/// Parses dynamic Function inputs as UTF-16, retaining every source code point.
///
/// Valid surrogate pairs become supplementary code points. Lone surrogates are
/// accepted in comments and literal contents, with exact cooked/raw values and
/// retained function source. Their occurrence in identifiers remains a syntax
/// error. Spans use UTF-8 lengths for scalars and three bytes per lone surrogate.
pub fn parse_dynamic_function_utf16(
    parameters: &JsString,
    body: &JsString,
) -> Result<Rc<Function>, Diagnostic> {
    parse_sources(
        source::SourceText::from_utf16(parameters)?,
        source::SourceText::from_utf16(body)?,
        FunctionKind::Ordinary,
    )
}

/// Parses dynamic AsyncFunction source with independent async grammar goals.
/// Await bindings/parameter expressions are syntax errors; body syntax retains
/// AwaitExpression nodes.
pub fn parse_dynamic_async_function(
    parameters: &str,
    body: &str,
) -> Result<Rc<Function>, Diagnostic> {
    parse_sources(
        source::SourceText::from_str(parameters),
        source::SourceText::from_str(body),
        FunctionKind::Async,
    )
}

/// Parses dynamic AsyncFunction inputs as lossless UTF-16, retaining standard
/// `async function anonymous` source and the body's independent strictness.
pub fn parse_dynamic_async_function_utf16(
    parameters: &JsString,
    body: &JsString,
) -> Result<Rc<Function>, Diagnostic> {
    parse_sources(
        source::SourceText::from_utf16(parameters)?,
        source::SourceText::from_utf16(body)?,
        FunctionKind::Async,
    )
}

/// Parses dynamic GeneratorFunction source with separate parameter/body goals.
/// Generator parameters forbid yield bindings and expressions; the body retains
/// bare/delegating YieldExpression nodes and its independent strictness.
pub fn parse_dynamic_generator_function(
    parameters: &str,
    body: &str,
) -> Result<Rc<Function>, Diagnostic> {
    parse_sources(
        source::SourceText::from_str(parameters),
        source::SourceText::from_str(body),
        FunctionKind::Generator,
    )
}

/// Parses dynamic GeneratorFunction inputs as lossless UTF-16, retaining the
/// standard `function* anonymous` source and all original source code points.
pub fn parse_dynamic_generator_function_utf16(
    parameters: &JsString,
    body: &JsString,
) -> Result<Rc<Function>, Diagnostic> {
    parse_sources(
        source::SourceText::from_utf16(parameters)?,
        source::SourceText::from_utf16(body)?,
        FunctionKind::Generator,
    )
}

fn parse_sources(
    parameters: source::SourceText,
    body: source::SourceText,
    kind: FunctionKind,
) -> Result<Rc<Function>, Diagnostic> {
    let parameters = Rc::new(parameters);
    let mut parser = Parser::from_source(parameters.clone())?;
    parser.allow_new_target = true;
    parser.allow_await_identifier = kind != FunctionKind::Async;
    parser.await_context = if kind == FunctionKind::Async {
        AwaitContext::Parameters
    } else {
        AwaitContext::None
    };
    parser.yield_context = if kind == FunctionKind::Generator {
        YieldContext::Parameters
    } else {
        YieldContext::None
    };
    let result = parser.formal_parameter_list("invalid function parameter identifier", false);
    parser.finish(result)?;
    if parser.current().kind != Kind::Eof {
        return Err(parser.error("unexpected token after function parameters"));
    }

    // The line feeds are part of CreateDynamicFunction's body parse string.
    let newline = source::SourceText::from_str("\n");
    let body = Rc::new(source::SourceText::join(&[&newline, &body, &newline])?);
    let mut parser = Parser::from_source(body.clone())?;
    parser.allow_new_target = true;
    parser.allow_await_identifier = kind != FunctionKind::Async;
    parser.await_context = if kind == FunctionKind::Async {
        AwaitContext::Body
    } else {
        AwaitContext::None
    };
    parser.yield_context = if kind == FunctionKind::Generator {
        YieldContext::Body
    } else {
        YieldContext::None
    };
    let result = parser.function_body_contents(false);
    parser.finish(result)?;

    let prefix = source::SourceText::from_str(match kind {
        FunctionKind::Ordinary => "function anonymous(",
        FunctionKind::Async => "async function anonymous(",
        FunctionKind::Generator => "function* anonymous(",
    });
    let middle = source::SourceText::from_str("\n) {");
    let suffix = source::SourceText::from_str("}");
    let source = source::SourceText::join(&[&prefix, &parameters, &middle, &body, &suffix])?;
    let mut parser = Parser::from_source(Rc::new(source))?;
    let result = parser.ordinary_function(false);
    let function = parser.finish(result)?;
    if parser.current().kind != Kind::Eof {
        return Err(parser.error("unexpected token after dynamic function"));
    }
    function::validate_function(&function, false)?;
    Ok(function)
}
