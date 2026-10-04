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
        self.allow_new_target = true;
        let result = self.ordinary_function_inner(require_name);
        self.allow_new_target = previous;
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
    ) -> Result<Rc<[Binding]>, Diagnostic> {
        self.expect("(")?;
        let mut parameters = Vec::new();
        while !self.at(")") {
            parameters.push(self.formal_parameter(invalid_name, true)?);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")")?;
        Ok(parameters.into())
    }

    pub(super) fn formal_parameter(
        &mut self,
        invalid_name: &str,
        allow_default: bool,
    ) -> Result<Binding, Diagnostic> {
        if self.at("...") || self.at("[") || self.at("{") {
            return Err(self.unsupported("rest and binding-pattern parameters are not implemented"));
        }
        let token = self.bump();
        let Kind::Word(name) = token.kind else {
            return Err(early(token.span, "invalid function parameter"));
        };
        if reserved(&name) {
            return Err(early(token.span, invalid_name));
        }
        let initializer = if allow_default && self.eat("=") {
            Some(self.expression_with_in(2, true)?)
        } else {
            None
        };
        Ok(Binding {
            name,
            span: token.span,
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
        let token_start = self.index;
        let previous_return = self.allow_return;
        let previous_in = self.allow_in;
        self.allow_return = true;
        self.allow_in = true;
        let result = (|| {
            let mut statements = Vec::new();
            while !self.at("}") {
                if self.current().kind == Kind::Eof {
                    return Err(self.error("unterminated function body"));
                }
                statements.push(self.statement(true)?);
            }
            let strict = has_use_strict(&statements, &self.source);
            if strict {
                reject_legacy_tokens(&self.tokens[token_start..self.index])?;
            }
            self.expect("}")?;
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
    parameters: &[Binding],
    inherited_strict: bool,
    own_strict: bool,
    unique: bool,
    span: Span,
) -> Result<BTreeSet<&str>, Diagnostic> {
    let non_simple = parameters.iter().any(|p| p.initializer.is_some());
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
        let duplicate = !names.insert(parameter.name.as_str());
        if duplicate && (unique || strict || non_simple) {
            return Err(early(parameter.span, "duplicate lexical binding"));
        }
        validate_binding(parameter, strict)?;
        if let Some(initializer) = &parameter.initializer {
            validate_expr(initializer, strict)?;
        }
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
                if parameters.contains(binding.name.as_str()) {
                    return Err(early(
                        binding.span,
                        "lexical declaration conflicts with parameter",
                    ));
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
