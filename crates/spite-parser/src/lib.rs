//! ECMAScript parsing and early-error validation.

mod array;
mod arrow;
mod assignment_pattern;
pub mod ast;
mod binding;
mod checkpoint;
mod class;
mod construction;
mod cover;
mod directive;
mod dynamic_function;
mod expression;
mod function;
mod generator;
mod import_call;
mod iteration;
pub mod json;
mod lexer;
mod module;
mod object;
mod operator;
mod optional_chain;
mod parser;
mod prefix;
mod private;
mod script;
mod source;
mod statement;
mod template;
mod using;
mod validation;

pub use dynamic_function::{
    parse_dynamic_async_function, parse_dynamic_async_function_utf16,
    parse_dynamic_async_generator_function, parse_dynamic_async_generator_function_utf16,
    parse_dynamic_function, parse_dynamic_function_utf16, parse_dynamic_generator_function,
    parse_dynamic_generator_function_utf16,
};
pub use module::{
    parse_module, parse_module_utf16, parse_module_utf16_with_source_limit,
    parse_module_with_source_limit,
};
pub use script::{
    EvalContext, parse_eval_utf16, parse_eval_utf16_with_private_names, parse_script,
    parse_script_utf16, parse_script_utf16_with_source_limit, parse_script_with_source_limit,
};
pub use spite_regexp::{
    RegExpNamedCapture, RegExpNamedReference, RegExpPatternMetadata, parse_regexp_pattern,
    validate_regexp_pattern,
};

use ast::{
    Argument, ArrayElement, ArrowBody, AssignmentElement, AssignmentPattern, AssignmentPatternKind,
    AssignmentProperty, AssignmentTarget, BinaryOp, BindingElement, BindingPattern,
    BindingPatternKind, BindingProperty, CatchClause, ChainStep, ChainStepKind, Class,
    ClassElement, ExportDeclaration, ExportKind, ExportSpecifier, Expr, ExprKind, ForBinding,
    ForInitializer, Function, FunctionBody, FunctionKind, FunctionName, FunctionSource,
    ImportAttribute, ImportBinding, ImportDeclaration, ImportEntry, ImportName,
    IndirectExportEntry, Literal, LocalExportEntry, Module, ModuleEntries, ModuleExportName,
    ModuleRequest, ObjectElement, ObjectProperty, Parameter, PrivateIdentifier, PropertyKind,
    PropertyName, Statement, StatementKind, SwitchClause, TemplateElement, UnaryOp, UpdateOp,
};
use directive::{has_use_strict, reject_legacy_tokens};
use lexer::{Kind, Lexer, Token};
use operator::{binary, compound_assignment, forbidden_nullish_mix};
use parser::{AwaitContext, Parser, YieldContext};
use spite_core::{Diagnostic, DiagnosticKind, JsString, Span};
use std::collections::BTreeSet;
use validation::{
    ControlContext, ScopeKind, assignment_name, assignment_target, block_lexical_names, early,
    member_base, reserved, strict_reserved, validate_binding_name, validate_expr, validate_scope,
    validate_scope_with_bindings,
};

/// Maximum recursive grammar-call depth accepted by this parser.
pub const MAX_DEPTH: usize = 192;

// Grammar calls and expression-tree nesting consume native frames differently.
// Bound grammar calls separately from expression and template nesting.
const MAX_EXPRESSION_DEPTH: usize = 64;
