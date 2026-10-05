//! Separate dynamic Function grammar goals and combined early errors (20.2.1.1.1).

use super::*;
use std::rc::Rc;

/// Parses ordinary dynamic Function source from a joined parameter list and body.
///
/// Each argument must form its own grammar goal before the combined function is
/// validated. The returned function retains the standard `anonymous` source text
/// and derives strictness from its body, independently of the caller.
/// Unsupported syntax and the parser's native-stack limit remain distinct from
/// syntax errors. Inputs use the same UTF-8 source representation as Scripts.
pub fn parse_dynamic_function(parameters: &str, body: &str) -> Result<Rc<Function>, Diagnostic> {
    let mut parser = Parser::new(parameters)?;
    parser.allow_new_target = true;
    parser.formal_parameter_list("invalid function parameter identifier", false)?;
    if parser.current().kind != Kind::Eof {
        return Err(parser.error("unexpected token after function parameters"));
    }

    // The line feeds are part of CreateDynamicFunction's body parse string.
    let body = format!("\n{body}\n");
    let mut parser = Parser::new(&body)?;
    parser.allow_new_target = true;
    parser.function_body_contents(false)?;

    let source = format!("function anonymous({parameters}\n) {{{body}}}");
    let mut parser = Parser::new(&source)?;
    let function = parser.ordinary_function(false)?;
    if parser.current().kind != Kind::Eof {
        return Err(parser.error("unexpected token after dynamic function"));
    }
    function::validate_function(&function, false)?;
    Ok(function)
}
