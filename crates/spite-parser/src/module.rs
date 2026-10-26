//! Module grammar goal, strict code and retained ordinary items (16.2).

use super::*;

mod exports;
mod imports;
mod records;

/// Parses the supported Module body with strict code and top-level Await.
///
/// Static imports and exports retain module records for runtime linking and
/// evaluation. The AST is separate from Script execution input.
pub fn parse_module(source: &str) -> Result<Module, Diagnostic> {
    let mut parser = Parser::new(source)?;
    let result = contents(&mut parser);
    parser.finish(result)
}

/// Parses a Module with an opted-in UTF-8 source length limit.
pub fn parse_module_with_source_limit(
    source: &str,
    max_bytes: usize,
) -> Result<Module, Diagnostic> {
    if source.len() > max_bytes {
        return Err(Diagnostic::new(
            DiagnosticKind::Limit,
            Span::new(0, source.len()),
            "source size limit exceeded",
        ));
    }
    parse_module(source)
}

/// Parses lossless UTF-16 Module source using the same byte-span convention as Script.
pub fn parse_module_utf16(source: &JsString) -> Result<Module, Diagnostic> {
    let mut parser =
        Parser::from_source(std::rc::Rc::new(source::SourceText::from_utf16(source)?))?;
    let result = contents(&mut parser);
    parser.finish(result)
}

fn contents(parser: &mut Parser) -> Result<Module, Diagnostic> {
    parser.module_goal = true;
    parser.allow_await_identifier = false;
    parser.await_context = AwaitContext::Body;
    let mut statements = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    while parser.current().kind != Kind::Eof {
        // Declarations are ModuleItems only at this level. Nested declarations
        // remain Syntax errors; import() and import.meta are expressions.
        if parser.at("export") && !parser.current().escaped {
            let (export, statement) = parser.export_declaration()?;
            exports.push(export);
            if let Some(statement) = statement {
                statements.push(statement);
            }
            continue;
        }
        if parser.at("import")
            && !parser.current().escaped
            && parser
                .token_at(parser.index + 1)
                .is_some_and(|token| !matches!(token.kind, Kind::Punct("(" | ".")))
        {
            imports.push(parser.import_declaration()?);
            continue;
        }
        statements.push(parser.statement(true)?);
    }
    reject_legacy_tokens(&parser.tokens)?;
    validate_scope_with_bindings(
        &statements,
        true,
        ControlContext::default(),
        &mut Vec::new(),
        ScopeKind::Module,
        imports.iter().flat_map(|declaration| {
            declaration
                .bindings
                .iter()
                .map(|binding| (binding.local.as_str(), binding.span))
        }),
    )?;
    exports::validate(&statements, &imports, &exports)?;
    Ok(Module {
        statements,
        imports,
        exports,
    })
}
