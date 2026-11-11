//! Statement evaluation, loop environments and catch completions.

use crate::environment::{BindingState, EnvironmentHandle};
use crate::{Completion, CompletionKind, Error, Realm, Value};
use spite_parser::ast::{
    Argument, BinaryOp, BindingElement, BindingPatternKind, CatchClause, Expr, ExprKind,
    ForInitializer, Literal, PropertyName, Statement, StatementKind, SwitchClause,
};
use std::collections::BTreeMap;

impl Realm {
    pub(crate) fn statements(&mut self, statements: &[Statement]) -> Result<Completion, Error> {
        let mut completion = Completion::normal(None);
        for statement in statements {
            // ECMA-262 14.2.2: even an abrupt statement inherits the previous
            // value when its own value is empty, then stops the statement list.
            completion = self.statement(statement)?.update_empty(completion.value);
            if completion.kind != CompletionKind::Normal {
                break;
            }
        }
        Ok(completion)
    }
    pub(crate) fn statement(&mut self, statement: &Statement) -> Result<Completion, Error> {
        self.labelled_statement(statement, &[])
    }

    pub(crate) fn labelled_statement(
        &mut self,
        statement: &Statement,
        labels: &[&str],
    ) -> Result<Completion, Error> {
        if let StatementKind::Expression(expression) = &statement.kind {
            // The expression owns the recursion guard. Its transparent statement
            // wrapper still pays work before evaluation, without a second slot.
            self.check_evaluation_depth(statement.span)?;
            self.tick(statement.span)?;
            return self
                .expression(expression)
                .map(|value| Completion::normal(Some(value)));
        }
        if let StatementKind::Return(expression) = &statement.kind {
            // As with expression statements, the value expression owns the
            // recursion guard; returning it adds work, not another AST level.
            self.check_evaluation_depth(statement.span)?;
            self.tick(statement.span)?;
            return self.return_completion(expression.as_ref());
        }
        self.enter_evaluation(statement.span)?;
        // Keep recursive returns out of the large general statement frame.
        let result = match &statement.kind {
            StatementKind::Expression(_) => unreachable!("expression statement dispatched above"),
            StatementKind::Return(_) => unreachable!("return statement dispatched above"),
            _ => self.statement_inner(statement, labels),
        };
        self.evaluation_depth -= 1;
        result
    }

    pub(crate) fn statement_inner(
        &mut self,
        statement: &Statement,
        labels: &[&str],
    ) -> Result<Completion, Error> {
        self.tick(statement.span)?;
        match &statement.kind {
            // ECMA-262 14.16.1: no debugging facility is active in this host.
            StatementKind::Empty | StatementKind::Debugger => Ok(Completion::normal(None)),
            // 15.2.6: declaration evaluation is empty; instantiation made the value.
            StatementKind::Function(_) => Ok(Completion::normal(None)),
            StatementKind::Class(class) => {
                let value = self.class_definition(class, None)?;
                let name = class.name.as_ref().expect("class declaration name");
                let environment = self.scopes.last().expect("declaration environment");
                self.objects
                    .environment_mut(environment)
                    .expect("declaration environment")
                    .bindings
                    .get_mut(&name.name)
                    .expect("instantiated class binding")
                    .value = Some(value);
                Ok(Completion::normal(None))
            }
            StatementKind::Expression(_) => {
                unreachable!("expression statement dispatched separately")
            }
            StatementKind::Break(target) | StatementKind::Continue(target) => Ok(Completion {
                tail_call: None,
                kind: if matches!(statement.kind, StatementKind::Break(_)) {
                    CompletionKind::Break
                } else {
                    CompletionKind::Continue
                },
                value: None,
                target: target.as_ref().map(|label| label.name.clone()),
            }),
            StatementKind::Labelled { label, body } => {
                // Only a chain of labels passes its label set into a loop.
                // Blocks and conditional branches call statement with a fresh set.
                let mut nested = labels.to_vec();
                nested.push(&label.name);
                let mut result = self.labelled_statement(body, &nested)?;
                if result.kind == CompletionKind::Break
                    && result.target.as_deref() == Some(&label.name)
                {
                    result.kind = CompletionKind::Normal;
                    result.target = None;
                }
                Ok(result)
            }
            StatementKind::Return(_) => unreachable!("return dispatched separately"),
            StatementKind::Throw(expr) => Err(Error::Thrown(self.expression(expr)?)),
            StatementKind::Using(bindings) => {
                self.initialize_using_bindings(bindings)?;
                Ok(Completion::normal(None))
            }
            StatementKind::Lexical { bindings, .. } => {
                self.initialize_bindings(bindings)?;
                Ok(Completion::normal(None))
            }
            StatementKind::Var(bindings) => {
                self.evaluate_var_bindings(bindings)?;
                Ok(Completion::normal(None))
            }
            // 14.2.2: an empty Block returns an empty completion directly.
            StatementKind::Block(body) if body.is_empty() => Ok(Completion::normal(None)),
            StatementKind::Block(body) => {
                self.push_scope(BTreeMap::new(), statement.span)?;
                let environment = self.scopes.last().expect("block environment").clone();
                let result = self
                    .instantiate(body.iter(), false, true)
                    .and_then(|()| self.statements(body));
                let result =
                    self.dispose_environment_resources(&environment, result, statement.span);
                self.scopes.pop();
                result
            }
            StatementKind::With { object, body } => {
                self.with_statement(object, body, statement.span)
            }
            StatementKind::Try {
                body,
                handler,
                finalizer,
            } => {
                // Calls in the protected body must still reach its handler or
                // finalizer. A catch without finally and the finalizer itself
                // can contain tail positions (15.10.2).
                let tail_scope = self.tail_scope.take();
                let body_result = self.statement(body);
                self.tail_scope = if finalizer.is_none() {
                    tail_scope
                } else {
                    None
                };
                let mut result = match body_result {
                    Err(error) if error.is_language_exception() => match handler {
                        Some(handler) => self.catch_clause(handler, error),
                        None => Err(error),
                    },
                    other => other,
                };
                self.tail_scope = tail_scope;
                // Host failures cannot enter or be suppressed by language control.
                if result.as_ref().is_err_and(|e| !e.is_language_exception()) {
                    return result;
                }
                // ECMA-262 14.15.3: a normal finalizer preserves the protected
                // completion, ignoring its own value. An abrupt one replaces it.
                if let Some(finalizer) = finalizer {
                    let finalizer_result = self.statement(finalizer)?;
                    if finalizer_result.kind != CompletionKind::Normal {
                        result = Ok(finalizer_result);
                    }
                }
                result.map(|c| c.update_empty(Some(Value::Undefined)))
            }
            StatementKind::Switch {
                discriminant,
                clauses,
            } => {
                // ECMA-262 14.12.4 evaluates the discriminant before creating
                // the shared case-block environment. Selectors use that new scope.
                let input = self.expression(discriminant)?;
                self.push_scope(BTreeMap::new(), statement.span)?;
                let result = self
                    .instantiate(
                        clauses.iter().flat_map(|clause| clause.statements.iter()),
                        false,
                        true,
                    )
                    .and_then(|()| self.case_block(clauses, &input));
                self.scopes.pop();
                result.map(Completion::consume_unlabelled_break)
            }
            // ECMA-262 14.7.3.2: the result is the last non-empty body value,
            // initially undefined. Condition values never replace it.
            StatementKind::While { test, body } => {
                let mut value = Value::Undefined;
                while self.expression(test)?.to_boolean() {
                    let result = self.statement(body)?.update_empty(Some(value));
                    if !result.loop_continues(labels) {
                        return Ok(result.consume_unlabelled_break());
                    }
                    value = result.value.expect("loop UpdateEmpty supplies a value");
                }
                Ok(Completion::normal(Some(value)))
            }
            // ECMA-262 14.7.2.2: the first body precedes the first condition.
            StatementKind::DoWhile { body, test } => {
                let mut value = Value::Undefined;
                loop {
                    let result = self.statement(body)?.update_empty(Some(value));
                    if !result.loop_continues(labels) {
                        return Ok(result.consume_unlabelled_break());
                    }
                    value = result.value.expect("loop UpdateEmpty supplies a value");
                    if !self.expression(test)?.to_boolean() {
                        return Ok(Completion::normal(Some(value)));
                    }
                }
            }
            StatementKind::For {
                initializer,
                test,
                update,
                body,
            } => {
                // ECMA-262 14.7.4.2: all header bindings exist before any
                // initializer runs, and the outer scope is restored on every exit.
                match initializer {
                    Some(ForInitializer::Using(bindings)) => {
                        let scope = self.pattern_bindings(
                            bindings.iter().map(|binding| &binding.pattern),
                            false,
                        )?;
                        self.push_scope(scope, statement.span)?;
                        let environment =
                            self.scopes.last().expect("using loop environment").clone();
                        let result = self.initialize_using_bindings(bindings).and_then(|()| {
                            self.for_body(test.as_ref(), update.as_ref(), body, &[], labels)
                        });
                        let result = self.dispose_environment_resources(
                            &environment,
                            result,
                            statement.span,
                        );
                        self.scopes.pop();
                        result
                    }
                    Some(ForInitializer::Var(bindings)) => {
                        self.evaluate_var_bindings(bindings)?;
                        self.for_body(test.as_ref(), update.as_ref(), body, &[], labels)
                    }
                    Some(ForInitializer::Lexical { mutable, bindings }) => {
                        let scope = self.pattern_bindings(
                            bindings.iter().map(|binding| &binding.pattern),
                            *mutable,
                        )?;
                        self.push_scope(scope, statement.span)?;
                        let per_iteration = if *mutable { bindings.as_slice() } else { &[] };
                        let result = self.initialize_bindings(bindings).and_then(|()| {
                            self.for_body(
                                test.as_ref(),
                                update.as_ref(),
                                body,
                                per_iteration,
                                labels,
                            )
                        });
                        self.scopes.pop();
                        result
                    }
                    _ => {
                        if let Some(ForInitializer::Expression(expr)) = initializer {
                            self.expression(expr)?;
                        }
                        self.for_body(test.as_ref(), update.as_ref(), body, &[], labels)
                    }
                }
            }
            StatementKind::ForOf {
                binding,
                iterable,
                body,
                awaited,
            } => {
                assert!(
                    !*awaited,
                    "for-await-of is handled by the async execution driver"
                );
                // IteratorClose must observe the actual call completion.
                let tail_scope = self.tail_scope.take();
                let result = self.for_of(binding, iterable, body, labels, statement.span);
                self.tail_scope = tail_scope;
                result
            }
            StatementKind::ForIn {
                binding,
                object,
                body,
            } => self.for_in(binding, object, body, labels, statement.span),
            StatementKind::If {
                test,
                consequent,
                alternate,
            } => {
                let result = if self.expression(test)?.to_boolean() {
                    self.statement(consequent)?
                } else if let Some(alternate) = alternate {
                    self.statement(alternate)?
                } else {
                    Completion::normal(None)
                };
                // ECMA-262 14.6.2 applies UpdateEmpty even to break/continue.
                Ok(result.update_empty(Some(Value::Undefined)))
            }
        }
    }

    // ECMA-262 14.12.2/14.12.3: locate the first strictly equal selector,
    // skipping default during the search. After a match, fall through in source
    // order without evaluating any further selectors. Only when no selector
    // matches does execution begin at default (if present).
    pub(crate) fn case_block(
        &mut self,
        clauses: &[SwitchClause],
        input: &Value,
    ) -> Result<Completion, Error> {
        let mut selected = None;
        let mut default = None;
        for (index, clause) in clauses.iter().enumerate() {
            self.tick(clause.span)?;
            if let Some(test) = &clause.test {
                let candidate = self.expression(test)?;
                if self.strictly_equal(input, &candidate, test.span)? {
                    selected = Some(index);
                    break;
                }
            } else {
                default = Some(index);
            }
        }
        let mut value = Value::Undefined;
        if let Some(start) = selected.or(default) {
            for clause in &clauses[start..] {
                // Empty fall-through clauses still consume the host work budget.
                self.tick(clause.span)?;
                let result = self
                    .statements(&clause.statements)?
                    .update_empty(Some(value));
                if result.kind != CompletionKind::Normal {
                    return Ok(result);
                }
                value = result
                    .value
                    .expect("case-block UpdateEmpty supplies a value");
            }
        }
        Ok(Completion::normal(Some(value)))
    }

    // ECMA-262 14.7.4.3 ForBodyEvaluation. The body's completion determines
    // whether the update runs, and only body values contribute to the result.
    pub(crate) fn for_body(
        &mut self,
        test: Option<&Expr>,
        update: Option<&Expr>,
        body: &Statement,
        per_iteration: &[BindingElement],
        labels: &[&str],
    ) -> Result<Completion, Error> {
        let reuse_environment = self.can_reuse_for_environment(per_iteration, test, update, body);
        let mut value = Value::Undefined;
        self.create_per_iteration_environment(per_iteration)?;
        loop {
            if let Some(test) = test
                && !self.expression(test)?.to_boolean()
            {
                return Ok(Completion::normal(Some(value)));
            }
            // Statement evaluation consumes host budget even for `for (;;) ;`.
            let result = self.statement(body)?.update_empty(Some(value));
            if !result.loop_continues(labels) {
                return Ok(result.consume_unlabelled_break());
            }
            value = result.value.expect("loop UpdateEmpty supplies a value");
            if !reuse_environment {
                self.create_per_iteration_environment(per_iteration)?;
            }
            if let Some(update) = update {
                self.expression(update)?;
            }
        }
    }

    // Only Number-valued, noncapturing loop records can omit later copies.
    // Keep the first copy and the complete original path for either host quota.
    pub(crate) fn can_reuse_for_environment(
        &self,
        bindings: &[BindingElement],
        test: Option<&Expr>,
        update: Option<&Expr>,
        body: &Statement,
    ) -> bool {
        if self.limits.max_steps.is_some()
            || self.limits.max_heap_entries.is_some()
            || bindings.is_empty()
        {
            return false;
        }
        let Some(current) = self.scopes.last() else {
            return false;
        };
        let Ok(environment) = self.objects.environment(current) else {
            return false;
        };
        for binding in bindings {
            let BindingPatternKind::Identifier(name) = &binding.pattern.kind else {
                return false;
            };
            if name == "eval"
                || !environment
                    .bindings
                    .get(name)
                    .is_some_and(|binding| matches!(&binding.value, Some(Value::Number(_))))
            {
                return false;
            }
        }
        // The visitor excludes captures and keeps every header write numeric,
        // so omitted records cannot retain historical object or Symbol values.
        let header = |name: &str| {
            bindings.iter().any(|binding| {
                matches!(&binding.pattern.kind, BindingPatternKind::Identifier(value) if value == name)
            })
        };
        enum Node<'a> {
            Expression(&'a Expr),
            Statement(&'a Statement),
        }
        let mut pending = Vec::new();
        macro_rules! push {
            ($node:expr) => {
                if pending.try_reserve(1).is_err() {
                    return false;
                }
                pending.push($node);
            };
        }
        for binding in bindings {
            if let Some(initializer) = &binding.initializer {
                push!(Node::Expression(initializer));
            }
        }
        for expression in [test, update].into_iter().flatten() {
            push!(Node::Expression(expression));
        }
        push!(Node::Statement(body));
        while let Some(node) = pending.pop() {
            match node {
                Node::Statement(statement) => match &statement.kind {
                    StatementKind::Empty => {}
                    StatementKind::Expression(expression) => {
                        push!(Node::Expression(expression));
                    }
                    StatementKind::Block(statements) => {
                        for statement in statements {
                            push!(Node::Statement(statement));
                        }
                    }
                    StatementKind::If {
                        test,
                        consequent,
                        alternate,
                    } => {
                        push!(Node::Expression(test));
                        push!(Node::Statement(consequent));
                        if let Some(alternate) = alternate {
                            push!(Node::Statement(alternate));
                        }
                    }
                    _ => return false,
                },
                Node::Expression(expression) => match &expression.kind {
                    ExprKind::Literal(Literal::RegExp { .. }) => return false,
                    ExprKind::Literal(_) => {}
                    ExprKind::Identifier(name) if name != "eval" => {}
                    ExprKind::Parenthesized(inner) => {
                        push!(Node::Expression(inner));
                    }
                    ExprKind::Member(object, property) => {
                        push!(Node::Expression(object));
                        match property {
                            PropertyName::Literal(_) => {}
                            PropertyName::Computed(key) => {
                                push!(Node::Expression(key));
                            }
                            PropertyName::Private(_) => return false,
                        }
                    }
                    ExprKind::Binary(
                        BinaryOp::StrictEqual
                        | BinaryOp::StrictNotEqual
                        | BinaryOp::Less
                        | BinaryOp::LessEqual
                        | BinaryOp::Greater
                        | BinaryOp::GreaterEqual,
                        left,
                        right,
                    ) => {
                        push!(Node::Expression(left));
                        push!(Node::Expression(right));
                    }
                    ExprKind::Assign(target, value) => {
                        match &target.kind {
                            ExprKind::Identifier(name) if header(name) => {
                                if !matches!(&value.kind, ExprKind::Literal(Literal::Number(_))) {
                                    return false;
                                }
                            }
                            ExprKind::Identifier(_) | ExprKind::Member(_, _) => {}
                            _ => return false,
                        }
                        push!(Node::Expression(target));
                        push!(Node::Expression(value));
                    }
                    ExprKind::CompoundAssign(BinaryOp::Add, target, value) => {
                        match &target.kind {
                            ExprKind::Identifier(name) if !header(name) => {}
                            ExprKind::Member(_, _) => {}
                            _ => return false,
                        }
                        push!(Node::Expression(target));
                        push!(Node::Expression(value));
                    }
                    ExprKind::Update { argument, .. } => {
                        if !matches!(
                            &argument.kind,
                            ExprKind::Identifier(_) | ExprKind::Member(_, _)
                        ) {
                            return false;
                        }
                        push!(Node::Expression(argument));
                    }
                    ExprKind::Call { callee, arguments } => {
                        if !matches!(&callee.kind, ExprKind::Member(_, _)) {
                            return false;
                        }
                        push!(Node::Expression(callee));
                        for argument in arguments {
                            let Argument::Expression(expression) = argument else {
                                return false;
                            };
                            push!(Node::Expression(expression));
                        }
                    }
                    _ => return false,
                },
            }
        }
        true
    }

    // ECMA-262 14.7.4.4: copy let values into a fresh environment with the
    // same outer environment. Const declarations do not request this operation.
    pub(crate) fn create_per_iteration_environment(
        &mut self,
        bindings: &[BindingElement],
    ) -> Result<(), Error> {
        if bindings.is_empty() {
            return Ok(());
        }
        let mut next = BTreeMap::new();
        for (name, span) in bindings
            .iter()
            .flat_map(|binding| binding.pattern.bound_names())
        {
            self.tick(span)?;
            let handle = self.scopes.last().expect("loop environment exists");
            let scope = &self
                .objects
                .environment(handle)
                .expect("active environment")
                .bindings;
            let value = scope[name]
                .value
                .clone()
                .expect("loop binding is initialized");
            next.insert(
                name.to_owned(),
                BindingState {
                    value: Some(value),
                    mutable: true,
                    deletable: false,
                    strict: true,
                },
            );
        }
        if next.is_empty() {
            return Ok(());
        }
        let current = self.scopes.last().expect("loop environment exists");
        let outer = self
            .objects
            .environment(current)
            .expect("active environment")
            .outer
            .clone();
        let next = self.object_work(bindings[0].pattern.span, |objects, budget| {
            objects.create_environment(outer, next, budget)
        })?;
        *self.scopes.last_mut().expect("loop environment exists") = next;
        Ok(())
    }

    pub(crate) fn catch_clause(
        &mut self,
        handler: &CatchClause,
        error: Error,
    ) -> Result<Completion, Error> {
        let Some((environment, value)) = self.prepare_catch_clause(handler, error)? else {
            return self.statement(&handler.body);
        };
        let parameter = handler.parameter.as_ref().expect("catch parameter");
        let result = self
            .initialize_pattern(parameter, value, &environment)
            .and_then(|()| self.statement(&handler.body));
        self.scopes.pop();
        result
    }

    pub(crate) fn prepare_catch_clause(
        &mut self,
        handler: &CatchClause,
        error: Error,
    ) -> Result<Option<(EnvironmentHandle, Value)>, Error> {
        let Some(parameter) = &handler.parameter else {
            return Ok(None);
        };
        let value = match error {
            Error::Thrown(value) => value,
            Error::Exception {
                kind,
                span,
                message,
            } => self.materialize_exception(kind, message, span)?,
            _ => unreachable!("only language throws enter a catch clause"),
        };
        let mut bindings = BTreeMap::new();
        for (name, span) in parameter.bound_names() {
            self.object_work(span, |_, budget| budget.charge(name.len() + 1))?;
            bindings.insert(
                name.to_owned(),
                BindingState {
                    value: None,
                    mutable: true,
                    deletable: false,
                    strict: true,
                },
            );
        }
        self.push_scope(bindings, handler.span)?;
        let environment = self.scopes.last().expect("catch environment").clone();
        Ok(Some((environment, value)))
    }
}
