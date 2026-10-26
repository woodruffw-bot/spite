//! Module Contains `await` for ParseModule's HasTLA (16.2.1).

use super::*;

enum Node<'a> {
    Statement(&'a Statement),
    Expression(&'a Expr),
    Name(&'a PropertyName),
    Class(&'a Class),
    Binding(&'a BindingPattern),
    BindingElement(&'a BindingElement),
    Assignment(&'a AssignmentPattern),
    AssignmentElement(&'a AssignmentElement),
    Target(&'a AssignmentTarget),
    ForBinding(&'a ForBinding),
}

impl Module {
    /// Whether Module evaluation contains top-level Await syntax.
    ///
    /// Includes for-await loops and awaits in computed names, bindings and class
    /// heritage. Function bodies/parameters and class initializers are separate
    /// grammar scopes. The query is static, including unreachable statements.
    pub fn has_top_level_await(&self) -> bool {
        let mut pending: Vec<_> = self.statements.iter().map(Node::Statement).collect();
        while let Some(node) = pending.pop() {
            match node {
                Node::Statement(statement) => match &statement.kind {
                    StatementKind::Empty
                    | StatementKind::Debugger
                    | StatementKind::Function(_)
                    | StatementKind::Break(_)
                    | StatementKind::Continue(_) => (),
                    StatementKind::Expression(value) | StatementKind::Throw(value) => {
                        pending.push(Node::Expression(value))
                    }
                    StatementKind::Return(value) => {
                        pending.extend(value.iter().map(Node::Expression))
                    }
                    StatementKind::Var(bindings)
                    | StatementKind::Using(bindings)
                    | StatementKind::Lexical { bindings, .. } => {
                        pending.extend(bindings.iter().map(Node::BindingElement))
                    }
                    StatementKind::Class(class) => pending.push(Node::Class(class)),
                    StatementKind::Block(body) => pending.extend(body.iter().map(Node::Statement)),
                    StatementKind::Labelled { body, .. } => pending.push(Node::Statement(body)),
                    StatementKind::If {
                        test,
                        consequent,
                        alternate,
                    } => {
                        pending.push(Node::Expression(test));
                        pending.push(Node::Statement(consequent));
                        pending.extend(alternate.iter().map(|body| Node::Statement(body)));
                    }
                    StatementKind::While { test, body } | StatementKind::DoWhile { test, body } => {
                        pending.push(Node::Expression(test));
                        pending.push(Node::Statement(body));
                    }
                    StatementKind::With { object, body } => {
                        pending.push(Node::Expression(object));
                        pending.push(Node::Statement(body));
                    }
                    StatementKind::For {
                        initializer,
                        test,
                        update,
                        body,
                    } => {
                        if let Some(initializer) = initializer {
                            match initializer {
                                ForInitializer::Expression(value) => {
                                    pending.push(Node::Expression(value))
                                }
                                ForInitializer::Var(bindings)
                                | ForInitializer::Using(bindings)
                                | ForInitializer::Lexical { bindings, .. } => {
                                    pending.extend(bindings.iter().map(Node::BindingElement))
                                }
                            }
                        }
                        pending.extend(test.iter().chain(update).map(Node::Expression));
                        pending.push(Node::Statement(body));
                    }
                    StatementKind::ForOf {
                        awaited,
                        binding,
                        iterable,
                        body,
                    } => {
                        if *awaited {
                            return true;
                        }
                        pending.push(Node::ForBinding(binding));
                        pending.push(Node::Expression(iterable));
                        pending.push(Node::Statement(body));
                    }
                    StatementKind::ForIn {
                        binding,
                        object,
                        body,
                    } => {
                        pending.push(Node::ForBinding(binding));
                        pending.push(Node::Expression(object));
                        pending.push(Node::Statement(body));
                    }
                    StatementKind::Switch {
                        discriminant,
                        clauses,
                    } => {
                        pending.push(Node::Expression(discriminant));
                        for clause in clauses {
                            pending.extend(clause.test.iter().map(Node::Expression));
                            pending.extend(clause.statements.iter().map(Node::Statement));
                        }
                    }
                    StatementKind::Try {
                        body,
                        handler,
                        finalizer,
                    } => {
                        pending.push(Node::Statement(body));
                        if let Some(handler) = handler {
                            pending.extend(handler.parameter.iter().map(Node::Binding));
                            pending.push(Node::Statement(&handler.body));
                        }
                        pending.extend(finalizer.iter().map(|body| Node::Statement(body)));
                    }
                },
                Node::Expression(expression) => match &expression.kind {
                    ExprKind::Await(_) => return true,
                    ExprKind::Literal(_)
                    | ExprKind::Identifier(_)
                    | ExprKind::This
                    | ExprKind::NewTarget
                    | ExprKind::ImportMeta
                    | ExprKind::Function(_)
                    | ExprKind::Arrow { .. } => (),
                    ExprKind::Array(elements) => pending.extend(
                        elements
                            .iter()
                            .filter_map(ArrayElement::expression)
                            .map(Node::Expression),
                    ),
                    ExprKind::Object(elements) => {
                        for element in elements {
                            match element {
                                ObjectElement::Spread(value) => {
                                    pending.push(Node::Expression(value))
                                }
                                ObjectElement::Property(property) => {
                                    pending.push(Node::Name(&property.name));
                                    pending.push(Node::Expression(&property.value));
                                }
                            }
                        }
                    }
                    ExprKind::Template { substitutions, .. } => {
                        pending.extend(substitutions.iter().map(Node::Expression))
                    }
                    ExprKind::TaggedTemplate {
                        tag, substitutions, ..
                    } => {
                        pending.push(Node::Expression(tag));
                        pending.extend(substitutions.iter().map(Node::Expression));
                    }
                    ExprKind::OptionalChain { base, steps } => {
                        pending.push(Node::Expression(base));
                        for step in steps {
                            match &step.kind {
                                ChainStepKind::Property(name) => pending.push(Node::Name(name)),
                                ChainStepKind::Call(arguments) => pending.extend(
                                    arguments
                                        .iter()
                                        .map(|argument| Node::Expression(argument.expression())),
                                ),
                            }
                        }
                    }
                    ExprKind::Class(class) => pending.push(Node::Class(class)),
                    ExprKind::Parenthesized(value)
                    | ExprKind::PrivateIn { value, .. }
                    | ExprKind::Unary(_, value)
                    | ExprKind::Update {
                        argument: value, ..
                    } => pending.push(Node::Expression(value)),
                    ExprKind::Member(base, name) => {
                        pending.push(Node::Expression(base));
                        pending.push(Node::Name(name));
                    }
                    ExprKind::SuperProperty(name) => pending.push(Node::Name(name)),
                    ExprKind::SuperCall(arguments) => pending.extend(
                        arguments
                            .iter()
                            .map(|argument| Node::Expression(argument.expression())),
                    ),
                    ExprKind::Call { callee, arguments } => {
                        pending.push(Node::Expression(callee));
                        pending.extend(
                            arguments
                                .iter()
                                .map(|argument| Node::Expression(argument.expression())),
                        );
                    }
                    ExprKind::New { callee, arguments } => {
                        pending.push(Node::Expression(callee));
                        pending.extend(
                            arguments
                                .iter()
                                .flatten()
                                .map(|argument| Node::Expression(argument.expression())),
                        );
                    }
                    ExprKind::ImportCall { specifier, options } => {
                        pending.push(Node::Expression(specifier));
                        pending.extend(options.iter().map(|value| Node::Expression(value)));
                    }
                    ExprKind::DestructuringAssign { pattern, value } => {
                        pending.push(Node::Assignment(pattern));
                        pending.push(Node::Expression(value));
                    }
                    ExprKind::Yield { argument, .. } => {
                        pending.extend(argument.iter().map(|value| Node::Expression(value)))
                    }
                    ExprKind::Binary(_, left, right)
                    | ExprKind::Assign(left, right)
                    | ExprKind::CompoundAssign(_, left, right) => {
                        pending.push(Node::Expression(left));
                        pending.push(Node::Expression(right));
                    }
                    ExprKind::BinaryChain { head, steps } => {
                        pending.push(Node::Expression(head));
                        pending.extend(steps.iter().map(|step| Node::Expression(&step.right)));
                    }
                    ExprKind::Conditional(test, yes, no) => {
                        pending.push(Node::Expression(test));
                        pending.push(Node::Expression(yes));
                        pending.push(Node::Expression(no));
                    }
                },
                Node::Name(name) => {
                    if let PropertyName::Computed(value) = name {
                        pending.push(Node::Expression(value));
                    }
                }
                Node::Class(class) => {
                    pending.extend(class.heritage.iter().map(Node::Expression));
                    // Contains excludes method/constructor bodies, field
                    // initializers and static blocks, but includes element names.
                    pending.extend(
                        class
                            .elements
                            .iter()
                            .filter_map(ClassElement::name)
                            .map(Node::Name),
                    );
                }
                Node::Binding(pattern) => match &pattern.kind {
                    BindingPatternKind::Identifier(_) => (),
                    BindingPatternKind::Object { properties, rest } => {
                        for property in properties {
                            pending.push(Node::Name(&property.key));
                            pending.push(Node::BindingElement(&property.element));
                        }
                        pending.extend(rest.iter().map(|pattern| Node::Binding(pattern)));
                    }
                    BindingPatternKind::Array { elements, rest } => {
                        pending.extend(elements.iter().flatten().map(Node::BindingElement));
                        pending.extend(rest.iter().map(|pattern| Node::Binding(pattern)));
                    }
                },
                Node::BindingElement(element) => {
                    pending.push(Node::Binding(&element.pattern));
                    pending.extend(element.initializer.iter().map(Node::Expression));
                }
                Node::Assignment(pattern) => match &pattern.kind {
                    AssignmentPatternKind::Object { properties, rest } => {
                        for property in properties {
                            pending.push(Node::Name(&property.key));
                            pending.push(Node::AssignmentElement(&property.element));
                        }
                        pending.extend(rest.iter().map(|value| Node::Expression(value)));
                    }
                    AssignmentPatternKind::Array { elements, rest } => {
                        pending.extend(elements.iter().flatten().map(Node::AssignmentElement));
                        pending.extend(rest.iter().map(Node::Target));
                    }
                },
                Node::AssignmentElement(element) => {
                    pending.push(Node::Target(&element.target));
                    pending.extend(element.initializer.iter().map(Node::Expression));
                }
                Node::Target(target) => match target {
                    AssignmentTarget::Reference(value) => pending.push(Node::Expression(value)),
                    AssignmentTarget::Pattern(pattern) => pending.push(Node::Assignment(pattern)),
                },
                Node::ForBinding(binding) => match binding {
                    ForBinding::Assignment(value) => pending.push(Node::Expression(value)),
                    ForBinding::Pattern(pattern) => pending.push(Node::Assignment(pattern)),
                    ForBinding::Var(element) => pending.push(Node::BindingElement(element)),
                    ForBinding::Using(pattern)
                    | ForBinding::Lexical {
                        binding: pattern, ..
                    } => pending.push(Node::Binding(pattern)),
                },
            }
        }
        false
    }
}
