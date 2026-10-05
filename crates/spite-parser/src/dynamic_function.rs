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
    )
}

fn parse_sources(
    parameters: source::SourceText,
    body: source::SourceText,
) -> Result<Rc<Function>, Diagnostic> {
    let parameters = Rc::new(parameters);
    let mut parser = Parser::from_source(parameters.clone())?;
    parser.allow_new_target = true;
    parser.formal_parameter_list("invalid function parameter identifier", false)?;
    if parser.current().kind != Kind::Eof {
        return Err(parser.error("unexpected token after function parameters"));
    }

    // The line feeds are part of CreateDynamicFunction's body parse string.
    let newline = source::SourceText::from_str("\n");
    let body = Rc::new(source::SourceText::join(&[&newline, &body, &newline])?);
    let mut parser = Parser::from_source(body.clone())?;
    parser.allow_new_target = true;
    parser.function_body_contents(false)?;

    let prefix = source::SourceText::from_str("function anonymous(");
    let middle = source::SourceText::from_str("\n) {");
    let suffix = source::SourceText::from_str("}");
    let source = source::SourceText::join(&[&prefix, &parameters, &middle, &body, &suffix])?;
    let mut parser = Parser::from_source(Rc::new(source))?;
    let function = parser.ordinary_function(false)?;
    if parser.current().kind != Kind::Eof {
        return Err(parser.error("unexpected token after dynamic function"));
    }
    function::validate_function(&function, false)?;
    Ok(function)
}
