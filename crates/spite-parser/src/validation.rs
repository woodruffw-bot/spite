//! Binding, control-flow, scope and expression early errors.

use crate::ast::{
    ArrayElement, ArrowBody, BindingElement, ChainStepKind, Expr, ExprKind, ForBinding,
    ForInitializer, Label, ObjectElement, PropertyKind, PropertyName, Statement, StatementKind,
    UnaryOp,
};
use crate::{assignment_pattern, binding, class, function, private, using};
use spite_core::{Diagnostic, DiagnosticKind, Span};
use std::collections::BTreeSet;

pub(crate) fn assignment_name(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name),
        ExprKind::Parenthesized(inner) => assignment_name(inner),
        _ => None,
    }
}

pub(crate) fn assignment_target(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Identifier(_) | ExprKind::Member(..) | ExprKind::SuperProperty(_) => true,
        ExprKind::Parenthesized(inner) => assignment_target(inner),
        _ => false,
    }
}

pub(crate) fn member_base(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Identifier(_)
            | ExprKind::This
            | ExprKind::NewTarget
            | ExprKind::ImportMeta
            | ExprKind::Literal(_)
            | ExprKind::Object(_)
            | ExprKind::Array(_)
            | ExprKind::Template { .. }
            | ExprKind::TaggedTemplate { .. }
            | ExprKind::OptionalChain { .. }
            | ExprKind::Parenthesized(_)
            | ExprKind::Member(..)
            | ExprKind::SuperProperty(_)
            | ExprKind::Call { .. }
            | ExprKind::ImportCall { .. }
            | ExprKind::New {
                arguments: Some(_),
                ..
            }
            | ExprKind::Function(_)
            | ExprKind::Class(_)
            | ExprKind::SuperCall(_)
    )
}

pub(crate) fn reserved(name: &str) -> bool {
    matches!(
        name,
        "null"
            | "true"
            | "false"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
    )
}
pub(crate) fn strict_reserved(name: &str) -> bool {
    matches!(
        name,
        "implements"
            | "interface"
            | "let"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "static"
            | "yield"
    )
}
pub(crate) fn early(span: Span, message: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticKind::Syntax, span, message)
}

pub(crate) fn validate_binding_names<'a>(
    bindings: &'a [BindingElement],
    strict: bool,
    names: &mut BTreeSet<&'a str>,
) -> Result<(), Diagnostic> {
    for binding in bindings {
        for (name, span) in binding.pattern.bound_names() {
            if !names.insert(name) {
                return Err(early(span, "duplicate lexical binding"));
            }
        }
        binding::validate_pattern(&binding.pattern, strict)?;
    }
    Ok(())
}

pub(crate) fn validate_binding_name(
    name: &str,
    span: Span,
    strict: bool,
) -> Result<(), Diagnostic> {
    if strict && (strict_reserved(name) || matches!(name, "eval" | "arguments")) {
        return Err(early(span, "invalid binding in strict mode"));
    }
    Ok(())
}

pub(crate) fn validate_var_bindings(
    bindings: &[BindingElement],
    strict: bool,
) -> Result<(), Diagnostic> {
    for binding in bindings {
        binding::validate_element(binding, strict)?;
    }
    Ok(())
}

// Function bodies start with a fresh context; class static blocks will too.
#[derive(Clone, Copy, Default)]
pub(crate) struct ControlContext {
    pub(crate) in_iteration: bool,
    pub(crate) in_breakable: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ScopeKind {
    Variable,
    Block,
    Module,
}

// LexicallyDeclaredNames of a block includes its function declarations (8.2.6).
pub(crate) fn block_lexical_names(statement: &Statement) -> impl Iterator<Item = (&str, Span)> {
    let bindings = match &statement.kind {
        StatementKind::Lexical { bindings, .. } | StatementKind::Using(bindings) => {
            bindings.as_slice()
        }
        _ => &[],
    };
    let function_name = match &statement.kind {
        StatementKind::Function(function) => function.name.as_ref(),
        StatementKind::Class(class) => class.name.as_ref(),
        _ => None,
    };
    bindings
        .iter()
        .flat_map(|b| b.pattern.bound_names())
        .chain(function_name.map(|name| (name.name.as_str(), name.span)))
}

pub(crate) fn validate_scope<'a>(
    statements: impl IntoIterator<Item = &'a Statement>,
    strict: bool,
    control: ControlContext,
    labels: &mut Vec<(&'a str, bool)>,
    scope_kind: ScopeKind,
) -> Result<(), Diagnostic> {
    validate_scope_with_bindings(
        statements,
        strict,
        control,
        labels,
        scope_kind,
        std::iter::empty(),
    )
}

pub(crate) fn validate_scope_with_bindings<'a>(
    statements: impl IntoIterator<Item = &'a Statement>,
    strict: bool,
    control: ControlContext,
    labels: &mut Vec<(&'a str, bool)>,
    scope_kind: ScopeKind,
    initial_bindings: impl IntoIterator<Item = (&'a str, Span)>,
) -> Result<(), Diagnostic> {
    let mut names = BTreeSet::new();
    for (name, span) in initial_bindings {
        validate_binding_name(name, span, strict)?;
        if !names.insert(name) {
            return Err(early(span, "duplicate lexical binding"));
        }
    }
    let mut var_names = BTreeSet::new();
    let mut declarations = Vec::new();
    for statement in statements {
        if matches!(scope_kind, ScopeKind::Block | ScopeKind::Module)
            || !matches!(statement.kind, StatementKind::Function(_))
        {
            for (name, span) in block_lexical_names(statement) {
                if !names.insert(name) {
                    return Err(early(span, "duplicate lexical binding"));
                }
                validate_binding_name(name, span, strict)?;
                if var_names.contains(name) {
                    return Err(early(span, "lexical declaration conflicts with var"));
                }
            }
        } else if let StatementKind::Function(function) = &statement.kind {
            // 8.2.8: direct functions are var-scoped only in Scripts/function bodies.
            let name = function.name.as_ref().expect("named declaration");
            if names.contains(name.name.as_str()) {
                return Err(early(
                    name.span,
                    "function declaration conflicts with lexical binding",
                ));
            }
            var_names.insert(name.name.as_str());
        }
        // ECMA-262 14.2.1, 14.12.1, 16.1.1: vars in nested statements
        // cannot conflict with lexical names of an enclosing statement list.
        declarations.clear();
        statement.collect_var_declarations(&mut declarations);
        for binding in &declarations {
            if names.contains(binding.name) {
                return Err(early(
                    binding.span,
                    "var declaration conflicts with lexical binding",
                ));
            }
            var_names.insert(binding.name);
        }
        validate_statement(statement, strict, control, labels)?;
    }
    Ok(())
}
pub(crate) fn validate_statement<'a>(
    statement: &'a Statement,
    strict: bool,
    control: ControlContext,
    labels: &mut Vec<(&'a str, bool)>,
) -> Result<(), Diagnostic> {
    match &statement.kind {
        StatementKind::Function(function) => function::validate_function(function, strict)?,
        StatementKind::Class(class) => class::validate_class(class)?,
        StatementKind::Var(bindings) => validate_var_bindings(bindings, strict)?,
        StatementKind::Expression(expr)
        | StatementKind::Throw(expr)
        | StatementKind::Return(Some(expr)) => validate_expr(expr, strict)?,
        StatementKind::Lexical { bindings, .. } | StatementKind::Using(bindings) => {
            for binding in bindings {
                binding::validate_element(binding, strict)?;
            }
        }
        StatementKind::Block(body) => {
            validate_scope(body, strict, control, labels, ScopeKind::Block)?;
        }
        StatementKind::With { object, body } => {
            // ECMA-262 14.11.1: this grammar is available only in non-strict code.
            if strict {
                return Err(early(
                    statement.span,
                    "with statements are not allowed in strict mode",
                ));
            }
            validate_expr(object, strict)?;
            validate_statement(body, strict, control, labels)?;
        }
        StatementKind::Try {
            body,
            handler,
            finalizer,
        } => {
            validate_statement(body, strict, control, labels)?;
            if let Some(handler) = handler {
                if let Some(parameter) = &handler.parameter {
                    binding::validate_pattern(parameter, strict)?;
                    let names = parameter.bound_names();
                    let mut seen = BTreeSet::new();
                    for &(name, span) in &names {
                        if !seen.insert(name) {
                            return Err(early(span, "duplicate catch binding"));
                        }
                    }
                    let StatementKind::Block(statements) = &handler.body.kind else {
                        unreachable!("catch requires a block");
                    };
                    // ECMA-262 14.15.1. The Annex B exception allowing var to
                    // redeclare a simple catch parameter is not enabled by this host.
                    for statement in statements {
                        for (name, span) in block_lexical_names(statement) {
                            if seen.contains(name) {
                                return Err(early(
                                    span,
                                    "catch parameter conflicts with lexical declaration",
                                ));
                            }
                        }
                    }
                    let mut declarations = Vec::new();
                    handler.body.collect_var_declarations(&mut declarations);
                    for binding in declarations {
                        if seen.contains(binding.name) {
                            return Err(early(
                                binding.span,
                                "catch parameter conflicts with var declaration",
                            ));
                        }
                    }
                }
                validate_statement(&handler.body, strict, control, labels)?;
            }
            if let Some(finalizer) = finalizer {
                validate_statement(finalizer, strict, control, labels)?;
            }
        }
        StatementKind::While { test, body } | StatementKind::DoWhile { test, body } => {
            validate_expr(test, strict)?;
            validate_statement(
                body,
                strict,
                ControlContext {
                    in_iteration: true,
                    in_breakable: true,
                },
                labels,
            )?;
        }
        StatementKind::For {
            initializer,
            test,
            update,
            body,
        } => {
            if let Some(initializer) = initializer {
                match initializer {
                    ForInitializer::Expression(expr) => validate_expr(expr, strict)?,
                    ForInitializer::Var(bindings) => validate_var_bindings(bindings, strict)?,
                    ForInitializer::Lexical { bindings, .. } | ForInitializer::Using(bindings) => {
                        let mut names = BTreeSet::new();
                        validate_binding_names(bindings, strict, &mut names)?;
                        let mut declarations = Vec::new();
                        body.collect_var_declarations(&mut declarations);
                        // ECMA-262 14.7.4.1: lexical header names cannot be vars in the body.
                        for binding in declarations {
                            if names.contains(binding.name) {
                                return Err(early(
                                    binding.span,
                                    "var declaration conflicts with lexical for binding",
                                ));
                            }
                        }
                        for binding in bindings {
                            if let Some(expr) = &binding.initializer {
                                validate_expr(expr, strict)?;
                            }
                        }
                    }
                }
            }
            for expression in [test, update].into_iter().flatten() {
                validate_expr(expression, strict)?;
            }
            validate_statement(
                body,
                strict,
                ControlContext {
                    in_iteration: true,
                    in_breakable: true,
                },
                labels,
            )?;
        }
        StatementKind::ForOf {
            binding,
            iterable,
            body,
            ..
        }
        | StatementKind::ForIn {
            binding,
            object: iterable,
            body,
        } => {
            match binding {
                ForBinding::Assignment(target) => {
                    if strict
                        && assignment_name(target).is_some_and(|name| {
                            strict_reserved(name) || matches!(name, "eval" | "arguments")
                        })
                    {
                        return Err(early(target.span, "invalid assignment in strict mode"));
                    }
                    validate_expr(target, strict)?;
                }
                ForBinding::Pattern(pattern) => {
                    assignment_pattern::validate_pattern(pattern, strict)?;
                }
                ForBinding::Var(binding) => {
                    binding::validate_pattern(&binding.pattern, strict)?;
                    if let Some(initializer) = &binding.initializer {
                        // Core ForBinding has no Initializer (14.7.5); the
                        // optional Annex B grammar is disabled for this host.
                        return Err(early(
                            initializer.span,
                            if strict {
                                "for-in var initializer is forbidden in strict mode"
                            } else {
                                "initialized for-in var declarations require Annex B"
                            },
                        ));
                    }
                }
                ForBinding::Lexical { binding, .. } | ForBinding::Using(binding) => {
                    binding::validate_pattern(binding, strict)?;
                    let mut names = BTreeSet::new();
                    for (name, span) in binding.bound_names() {
                        if !names.insert(name) {
                            return Err(early(span, "duplicate lexical binding"));
                        }
                    }
                    let mut declarations = Vec::new();
                    body.collect_var_declarations(&mut declarations);
                    for declaration in declarations {
                        if names.contains(declaration.name) {
                            return Err(early(
                                declaration.span,
                                "var declaration conflicts with lexical for binding",
                            ));
                        }
                    }
                }
            }
            validate_expr(iterable, strict)?;
            validate_statement(
                body,
                strict,
                ControlContext {
                    in_iteration: true,
                    in_breakable: true,
                },
                labels,
            )?;
        }
        StatementKind::Switch {
            discriminant,
            clauses,
        } => {
            validate_expr(discriminant, strict)?;
            for clause in clauses {
                using::reject_direct_using(
                    &clause.statements,
                    "using declaration requires a block inside switch clauses",
                )?;
                if let Some(test) = &clause.test {
                    validate_expr(test, strict)?;
                }
            }
            // ECMA-262 14.12.1: all clauses form one scope for duplicate names.
            // Switch permits break while preserving the enclosing loop context.
            validate_scope(
                clauses.iter().flat_map(|clause| &clause.statements),
                strict,
                ControlContext {
                    in_breakable: true,
                    ..control
                },
                labels,
                ScopeKind::Block,
            )?;
        }
        // ECMA-262 14.8.1/14.9.1 distinguish iterations from breakable statements.
        StatementKind::Break(None) if !control.in_breakable => {
            return Err(early(
                statement.span,
                "break requires an enclosing loop or switch",
            ));
        }
        StatementKind::Continue(None) if !control.in_iteration => {
            return Err(early(statement.span, "continue requires an enclosing loop"));
        }
        StatementKind::If {
            test,
            consequent,
            alternate,
        } => {
            validate_expr(test, strict)?;
            validate_statement(consequent, strict, control, labels)?;
            if let Some(alternate) = alternate {
                validate_statement(alternate, strict, control, labels)?;
            }
        }
        StatementKind::Labelled { label, body } => {
            validate_label(label, strict)?;
            // ECMA-262 8.3.1: duplicate labels are forbidden only while active.
            if labels.iter().any(|(name, _)| *name == label.name) {
                return Err(early(label.span, "duplicate label"));
            }
            labels.push((&label.name, labels_iteration(body)));
            let result = validate_statement(body, strict, control, labels);
            labels.pop();
            result?;
        }
        StatementKind::Break(Some(target)) | StatementKind::Continue(Some(target)) => {
            validate_label(target, strict)?;
            // ECMA-262 8.3.2/8.3.3: continue needs an iteration target, not just
            // a label enclosing an iteration somewhere in its subtree.
            let Some((_, is_iteration)) = labels.iter().find(|(name, _)| *name == target.name)
            else {
                return Err(early(target.span, "undefined label"));
            };
            if matches!(statement.kind, StatementKind::Continue(_)) && !is_iteration {
                return Err(early(
                    target.span,
                    "continue target is not an iteration label",
                ));
            }
        }
        StatementKind::Empty
        | StatementKind::Debugger
        | StatementKind::Break(None)
        | StatementKind::Continue(None)
        | StatementKind::Return(None) => {}
    }
    Ok(())
}
pub(crate) fn validate_label(label: &Label, strict: bool) -> Result<(), Diagnostic> {
    if strict && strict_reserved(&label.name) {
        return Err(early(label.span, "reserved label in strict mode"));
    }
    Ok(())
}

pub(crate) fn labels_iteration(mut statement: &Statement) -> bool {
    while let StatementKind::Labelled { body, .. } = &statement.kind {
        statement = body;
    }
    matches!(
        statement.kind,
        StatementKind::While { .. }
            | StatementKind::DoWhile { .. }
            | StatementKind::For { .. }
            | StatementKind::ForOf { .. }
            | StatementKind::ForIn { .. }
    )
}

pub(crate) fn validate_expr(expr: &Expr, strict: bool) -> Result<(), Diagnostic> {
    match &expr.kind {
        ExprKind::Function(function) => function::validate_function(function, strict)?,
        ExprKind::Class(class) => class::validate_class(class)?,
        ExprKind::Arrow {
            parameters, body, ..
        } => {
            let own_strict = matches!(body, ArrowBody::Block(body) if body.is_strict());
            let names =
                function::validate_parameters(parameters, strict, own_strict, true, expr.span)?;
            let strict = strict || own_strict;
            match body {
                ArrowBody::Expression(body) => validate_expr(body, strict)?,
                ArrowBody::Block(body) => function::validate_body(body, strict, &names)?,
            }
        }

        ExprKind::Array(elements) => {
            for element in elements.iter().filter_map(ArrayElement::expression) {
                validate_expr(element, strict)?;
            }
        }
        ExprKind::Object(properties) => {
            for element in properties {
                let property = match element {
                    ObjectElement::Spread(expression) => {
                        validate_expr(expression, strict)?;
                        continue;
                    }
                    ObjectElement::Property(property) => property,
                };
                if let PropertyName::Computed(key) = &property.name {
                    validate_expr(key, strict)?;
                }
                if matches!(
                    property.kind,
                    PropertyKind::Method | PropertyKind::Getter | PropertyKind::Setter
                ) {
                    let ExprKind::Function(function) = &property.value.kind else {
                        unreachable!("method syntax");
                    };
                    function::validate_method(function, strict)?;
                } else {
                    validate_expr(&property.value, strict)?;
                }
            }
        }
        ExprKind::OptionalChain { base, steps } => {
            validate_expr(base, strict)?;
            for step in steps {
                match &step.kind {
                    ChainStepKind::Property(PropertyName::Computed(expression)) => {
                        validate_expr(expression, strict)?;
                    }
                    ChainStepKind::Property(
                        PropertyName::Literal(_) | PropertyName::Private(_),
                    ) => {}
                    ChainStepKind::Call(arguments) => {
                        for argument in arguments {
                            validate_expr(argument.expression(), strict)?;
                        }
                    }
                }
            }
        }
        ExprKind::TaggedTemplate {
            tag, substitutions, ..
        } => {
            validate_expr(tag, strict)?;
            for expression in substitutions {
                validate_expr(expression, strict)?;
            }
        }
        ExprKind::Template { substitutions, .. } => {
            for expression in substitutions {
                validate_expr(expression, strict)?;
            }
        }
        ExprKind::Identifier(name) if strict && strict_reserved(name) => {
            return Err(early(expr.span, "reserved identifier in strict mode"));
        }
        ExprKind::Assign(target, value) | ExprKind::CompoundAssign(_, target, value) => {
            if strict
                && assignment_name(target).is_some_and(|name| {
                    strict_reserved(name) || matches!(name, "eval" | "arguments")
                })
            {
                return Err(early(expr.span, "invalid assignment in strict mode"));
            }
            validate_expr(target, strict)?;
            validate_expr(value, strict)?;
        }
        ExprKind::DestructuringAssign { pattern, value } => {
            assignment_pattern::validate_pattern(pattern, strict)?;
            validate_expr(value, strict)?;
        }
        ExprKind::Member(base, name) => {
            validate_expr(base, strict)?;
            if let PropertyName::Computed(key) = name {
                validate_expr(key, strict)?;
            }
        }
        ExprKind::PrivateIn { value, .. } => validate_expr(value, strict)?,
        ExprKind::SuperProperty(PropertyName::Computed(key)) => validate_expr(key, strict)?,
        ExprKind::SuperCall(arguments) => {
            for argument in arguments {
                validate_expr(argument.expression(), strict)?;
            }
        }
        ExprKind::ImportCall { specifier, options } => {
            validate_expr(specifier, strict)?;
            if let Some(options) = options {
                validate_expr(options, strict)?;
            }
        }
        ExprKind::Call { callee, arguments } => {
            validate_expr(callee, strict)?;
            for argument in arguments {
                validate_expr(argument.expression(), strict)?;
            }
        }
        ExprKind::New { callee, arguments } => {
            validate_expr(callee, strict)?;
            for argument in arguments.iter().flatten() {
                validate_expr(argument.expression(), strict)?;
            }
        }
        ExprKind::Update { argument, .. } => {
            if strict
                && assignment_name(argument).is_some_and(|name| {
                    strict_reserved(name) || matches!(name, "eval" | "arguments")
                })
            {
                return Err(early(argument.span, "invalid update target in strict mode"));
            }
            validate_expr(argument, strict)?;
        }
        ExprKind::Unary(UnaryOp::Delete, e) if strict && assignment_name(e).is_some() => {
            // ECMA-262 13.5.1.1 also rejects parenthesized identifier references.
            return Err(early(e.span, "cannot delete an identifier in strict mode"));
        }
        ExprKind::Unary(UnaryOp::Delete, e) if private::is_private_reference(e) => {
            return Err(early(e.span, "cannot delete a private element"));
        }
        ExprKind::Unary(_, e) | ExprKind::Await(e) | ExprKind::Parenthesized(e) => {
            validate_expr(e, strict)?;
        }
        ExprKind::Yield {
            argument: Some(argument),
            ..
        } => validate_expr(argument, strict)?,
        ExprKind::Binary(_, a, b) => {
            validate_expr(a, strict)?;
            validate_expr(b, strict)?;
        }
        ExprKind::BinaryChain { head, steps } => {
            validate_expr(head, strict)?;
            for step in steps {
                validate_expr(&step.right, strict)?;
            }
        }
        ExprKind::Conditional(a, b, c) => {
            validate_expr(a, strict)?;
            validate_expr(b, strict)?;
            validate_expr(c, strict)?;
        }
        _ => {}
    }
    Ok(())
}
