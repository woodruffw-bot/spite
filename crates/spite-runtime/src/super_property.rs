//! SuperProperty and MakeSuperPropertyReference (13.3.7.1, 13.3.7.3).

use crate::{Error, Realm, Reference, Value, environment::EnvironmentHandle};
use spite_core::Span;
use spite_parser::ast::{Argument, PropertyName};

#[derive(Debug)]
pub(crate) struct SuperCallTarget {
    environment: EnvironmentHandle,
    function: Value,
    superclass: Value,
    new_target: Value,
}

impl SuperCallTarget {
    pub(crate) fn values(&self) -> impl Iterator<Item = &Value> {
        [&self.function, &self.superclass, &self.new_target].into_iter()
    }
    pub(crate) fn environment(&self) -> &EnvironmentHandle {
        &self.environment
    }
}

impl Realm {
    pub(super) fn super_call(
        &mut self,
        arguments: &[Argument],
        span: Span,
    ) -> Result<Value, Error> {
        self.tick(span)?;
        let target = self.prepare_super_call(span)?;
        let values = self.argument_list(arguments)?;
        self.finish_super_call(target, values, span)
    }

    pub(crate) fn prepare_super_call(&mut self, span: Span) -> Result<SuperCallTarget, Error> {
        let mut next = self.scopes.last().cloned();
        let (environment, function, new_target) = loop {
            let environment = next.expect("validated derived constructor context");
            let (function_environment, function, target, outer) =
                self.object_work(span, |objects, budget| {
                    budget.charge(1)?;
                    let record = objects.environment(&environment)?;
                    Ok((
                        record.this.is_some(),
                        record.derived_constructor.clone(),
                        record.new_target.clone(),
                        record.outer.clone(),
                    ))
                })?;
            if function_environment {
                break (
                    environment,
                    function.expect("derived constructor"),
                    target.expect("constructor newTarget"),
                );
            }
            next = outer;
        };
        // GetSuperConstructor precedes arguments. A repeated super call still
        // evaluates arguments and constructs before BindThisValue rejects it.
        let superclass = self
            .get_prototype_of(&function, span)?
            .map_or(Value::Null, Value::Object);
        Ok(SuperCallTarget {
            environment,
            function: Value::Object(function),
            superclass,
            new_target: Value::Object(new_target),
        })
    }

    pub(crate) fn finish_super_call(
        &mut self,
        target: SuperCallTarget,
        values: Vec<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(function) = target.function else {
            unreachable!("super function object")
        };
        let Value::Object(new_target) = target.new_target else {
            unreachable!("super newTarget")
        };
        let environment = target.environment;
        let superclass = target.superclass;
        let instance =
            self.construct_with_new_target(superclass, values, Some(new_target), span)?;
        let initialized = self.object_work(span, |objects, budget| {
            budget.value(&instance)?;
            let record = objects.environment_mut(&environment)?;
            if !matches!(
                record.this,
                Some(crate::environment::ThisBinding::Uninitialized)
            ) {
                return Ok(false);
            }
            record.this = Some(crate::environment::ThisBinding::Initialized(
                instance.clone(),
            ));
            Ok(true)
        })?;
        if !initialized {
            return Err(Self::exception(
                crate::ExceptionKind::ReferenceError,
                span,
                "derived constructor this is already initialized",
            ));
        }
        // BindThisValue precedes instance initialization. A throwing initializer
        // leaves this initialized; a repeated super call never reruns the fields.
        self.initialize_instance_elements(&instance, &function, span)?;
        Ok(instance)
    }

    pub(super) fn super_property_reference<'a>(
        &mut self,
        name: &'a PropertyName,
        span: Span,
    ) -> Result<Reference<'a>, Error> {
        // GetThisBinding precedes computed-name evaluation. GetSuperBase follows
        // it, so the key expression can replace the home object's prototype.
        let this_value = self.this_value(span)?;
        let key = match name {
            PropertyName::Private(_) => {
                return Err(Self::unsupported(
                    span,
                    "private elements are not implemented",
                ));
            }
            PropertyName::Literal(literal) => self.literal_value(literal, span)?,
            PropertyName::Computed(expression) => self.expression(expression)?,
        };
        self.make_super_property_reference(key, this_value, span)
    }
    pub(crate) fn make_super_property_reference(
        &mut self,
        key: Value,
        this_value: Value,
        span: Span,
    ) -> Result<Reference<'static>, Error> {
        let mut next = self.scopes.last().cloned();
        let mut base = Value::Undefined;
        while let Some(environment) = next {
            let (function, home, outer) = self.object_work(span, |objects, budget| {
                budget.charge(1)?;
                let environment = objects.environment(&environment)?;
                Ok((
                    environment.this.is_some(),
                    environment.home_object.clone(),
                    environment.outer.clone(),
                ))
            })?;
            if function {
                if let Some(home) = home {
                    base = self
                        .get_prototype_of(&home, span)?
                        .map_or(Value::Null, Value::Object);
                }
                break;
            }
            next = outer;
        }
        Ok(Reference::SuperProperty {
            base,
            key,
            this_value,
        })
    }
}
