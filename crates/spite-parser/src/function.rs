//! Ordinary function expressions and shared formal-parameter/body parsing.

use super::*;
use std::rc::Rc;

impl Parser {
    pub(super) fn function_expression(&mut self) -> Result<Expr, Diagnostic> {
        let function = self.ordinary_function(false)?;
        let span = function.source.span;
        self.make_expr(ExprKind::Function(function), span)
    }

    pub(super) fn ordinary_function(
        &mut self,
        require_name: bool,
    ) -> Result<Rc<Function>, Diagnostic> {
        self.enter()?;
        let previous = self.allow_new_target;
        let previous_super = self.allow_super_property;
        self.allow_new_target = true;
        self.allow_super_property = false;
        let result = self.ordinary_function_inner(require_name);
        self.allow_new_target = previous;
        self.allow_super_property = previous_super;
        self.depth -= 1;
        result
    }

    fn ordinary_function_inner(&mut self, require_name: bool) -> Result<Rc<Function>, Diagnostic> {
        let start = self.current().span.start;
        self.expect("function")?;
        if self.at("*") {
            return Err(self.unsupported("generator functions are not implemented"));
        }
        let name = if self.at("(") {
            if require_name {
                return Err(self.error("function declaration requires a name"));
            }
            None
        } else {
            let token = self.bump();
            let Kind::Word(name) = token.kind else {
                return Err(early(token.span, "expected a function binding identifier"));
            };
            if reserved(&name) {
                return Err(early(token.span, "invalid function binding identifier"));
            }
            Some(FunctionName {
                name,
                span: token.span,
            })
        };
        let parameters = self.formal_parameters("invalid function parameter identifier")?;
        let body = self.function_body()?;
        let span = Span::new(start, self.tokens[self.index - 1].span.end);
        let source = FunctionSource {
            text: self.source.clone(),
            span,
        };
        Ok(Rc::new(Function {
            name,
            parameters,
            body,
            source,
        }))
    }

    pub(super) fn formal_parameters(
        &mut self,
        invalid_name: &str,
    ) -> Result<Rc<[Parameter]>, Diagnostic> {
        self.expect("(")?;
        let parameters = self.formal_parameter_list(invalid_name, true)?;
        self.expect(")")?;
        Ok(parameters)
    }

    pub(super) fn formal_parameter_list(
        &mut self,
        invalid_name: &str,
        parenthesized: bool,
    ) -> Result<Rc<[Parameter]>, Diagnostic> {
        let mut parameters = Vec::new();
        while if parenthesized {
            !self.at(")")
        } else {
            self.current().kind != Kind::Eof
        } {
            if self.eat("...") {
                let binding = self.formal_parameter(invalid_name, false)?;
                parameters.push(Parameter::Rest(binding));
                // BindingRestElement has no initializer and no trailing comma.
                break;
            }
            parameters.push(Parameter::Ordinary(
                self.formal_parameter(invalid_name, true)?,
            ));
            if !self.eat(",") {
                break;
            }
        }
        Ok(parameters.into())
    }

    pub(super) fn formal_parameter(
        &mut self,
        invalid_name: &str,
        allow_default: bool,
    ) -> Result<BindingElement, Diagnostic> {
        let pattern = if self.at("[") || self.at("{") {
            self.binding_pattern()?
        } else {
            let token = self.bump();
            let Kind::Word(name) = token.kind else {
                return Err(early(token.span, "invalid function parameter"));
            };
            if reserved(&name) {
                return Err(early(token.span, invalid_name));
            }
            BindingPattern {
                kind: BindingPatternKind::Identifier(name),
                span: token.span,
            }
        };
        let initializer = if allow_default && self.eat("=") {
            Some(self.expression_with_in(2, true)?)
        } else {
            None
        };
        Ok(BindingElement {
            pattern,
            initializer,
        })
    }

    pub(super) fn function_body(&mut self) -> Result<FunctionBody, Diagnostic> {
        // Declarations can nest without any expression calls. Charge the body
        // boundary as well as its statements before recursing into another head.
        self.enter()?;
        let result = self.function_body_inner();
        self.depth -= 1;
        result
    }

    fn function_body_inner(&mut self) -> Result<FunctionBody, Diagnostic> {
        self.expect("{")?;
        let body = self.function_body_contents(true)?;
        self.expect("}")?;
        Ok(body)
    }

    pub(super) fn function_body_contents(
        &mut self,
        braced: bool,
    ) -> Result<FunctionBody, Diagnostic> {
        let token_start = self.index;
        let previous_return = self.allow_return;
        let previous_in = self.allow_in;
        self.allow_return = true;
        self.allow_in = true;
        let result = (|| {
            let mut statements = Vec::new();
            while if braced {
                !self.at("}")
            } else {
                self.current().kind != Kind::Eof
            } {
                if self.current().kind == Kind::Eof {
                    return Err(self.error("unterminated function body"));
                }
                statements.push(self.statement(true)?);
            }
            let strict = has_use_strict(&statements, self.source.lexical_text());
            if strict {
                reject_legacy_tokens(&self.tokens[token_start..self.index])?;
            }
            Ok(FunctionBody {
                statements: statements.into(),
                strict,
            })
        })();
        self.allow_return = previous_return;
        self.allow_in = previous_in;
        result
    }
}

pub(super) fn validate_parameters(
    parameters: &[Parameter],
    inherited_strict: bool,
    own_strict: bool,
    unique: bool,
    span: Span,
) -> Result<BTreeSet<&str>, Diagnostic> {
    let non_simple = parameters.iter().any(|p| !p.is_simple());
    // ECMA-262 15.2.1 and 15.3.1.
    if own_strict && non_simple {
        return Err(early(
            span,
            "use strict directive with non-simple parameters",
        ));
    }
    let strict = inherited_strict || own_strict;
    let mut names = BTreeSet::new();
    for parameter in parameters {
        let parameter = parameter.binding();
        for (name, span) in parameter.pattern.bound_names() {
            let duplicate = !names.insert(name);
            if duplicate && (unique || strict || non_simple) {
                return Err(early(span, "duplicate lexical binding"));
            }
        }
        binding::validate_element(parameter, strict)?;
    }
    Ok(names)
}

pub(super) fn validate_body(
    body: &FunctionBody,
    strict: bool,
    parameters: &BTreeSet<&str>,
) -> Result<(), Diagnostic> {
    for statement in body.statements() {
        if let StatementKind::Lexical { bindings, .. } = &statement.kind {
            for binding in bindings {
                for (name, span) in binding.pattern.bound_names() {
                    if parameters.contains(name) {
                        return Err(early(span, "lexical declaration conflicts with parameter"));
                    }
                }
            }
        }
    }
    // ECMA-262 15.2.1: functions reset labels and break/continue targets.
    validate_scope(
        body.statements(),
        strict,
        ControlContext::default(),
        &mut Vec::new(),
        ScopeKind::Variable,
    )
}

pub(super) fn validate_function(
    function: &Function,
    inherited_strict: bool,
) -> Result<(), Diagnostic> {
    let own_strict = function.body.is_strict();
    let names = validate_parameters(
        &function.parameters,
        inherited_strict,
        own_strict,
        false,
        function.source.span,
    )?;
    let strict = inherited_strict || own_strict;
    if let Some(name) = &function.name {
        validate_binding_name(&name.name, name.span, strict)?;
    }
    validate_body(&function.body, strict, &names)
}

pub(super) fn validate_method(
    function: &Function,
    inherited_strict: bool,
) -> Result<(), Diagnostic> {
    let own_strict = function.body.is_strict();
    // MethodDefinition uses UniqueFormalParameters even in non-strict code (15.4).
    let names = validate_parameters(
        &function.parameters,
        inherited_strict,
        own_strict,
        true,
        function.source.span,
    )?;
    validate_body(&function.body, inherited_strict || own_strict, &names)
}
