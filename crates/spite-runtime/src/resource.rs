//! Synchronous lexical resource registration and DisposeResources completions.

use crate::{
    Completion, Error, Realm, Value,
    environment::EnvironmentHandle,
    object::{DisposableResource, DisposableStackKind, DisposalHint},
};
use spite_core::{JsString, Span};
use spite_parser::ast::{BindingElement, BindingPattern, BindingPatternKind};

impl Realm {
    pub(super) fn initialize_using_bindings(
        &mut self,
        bindings: &[BindingElement],
    ) -> Result<(), Error> {
        let environment = self.scopes.last().expect("using environment").clone();
        for binding in bindings {
            self.tick(binding.pattern.span)?;
            let BindingPatternKind::Identifier(name) = &binding.pattern.kind else {
                unreachable!("using declarations only bind identifiers");
            };
            let initializer = binding.initializer.as_ref().expect("using initializer");
            let value = self.named_expression(initializer, JsString::from(name.as_str()))?;
            self.initialize_using_value(&binding.pattern, value, &environment)?;
        }
        Ok(())
    }

    pub(super) fn initialize_using_value(
        &mut self,
        binding: &BindingPattern,
        value: Value,
        environment: &EnvironmentHandle,
    ) -> Result<(), Error> {
        if let Some(resource) =
            self.create_disposable_resource(&value, DisposableStackKind::Sync, binding.span)?
        {
            self.object_work(binding.span, |objects, budget| {
                objects.push_environment_disposable_resource(environment, resource, budget)
            })?;
        }
        // AddDisposableResource precedes InitializeReferencedBinding (14.3.1).
        self.initialize_pattern(binding, value, environment)
    }

    pub(super) fn dispose_environment_resources(
        &mut self,
        environment: &EnvironmentHandle,
        completion: Result<Completion, Error>,
        span: Span,
    ) -> Result<Completion, Error> {
        let resources = self
            .objects
            .take_environment_disposable_resources(environment)
            .expect("active resource environment");
        self.dispose_resource_list(resources, completion, span)
    }

    pub(super) fn dispose_resource_list(
        &mut self,
        resources: Vec<DisposableResource>,
        mut completion: Result<Completion, Error>,
        span: Span,
    ) -> Result<Completion, Error> {
        if completion
            .as_ref()
            .is_err_and(|error| !error.is_language_exception())
        {
            return completion;
        }
        for resource in resources.into_iter().rev() {
            debug_assert_eq!(resource.hint, DisposalHint::Sync);
            let arguments = resource.argument.into_iter().collect();
            if let Err(error) = self.call(resource.method, resource.value, arguments, span) {
                if !error.is_language_exception() {
                    return Err(error);
                }
                completion = Err(if let Err(previous) = completion {
                    let error = self.language_exception_value(error)?;
                    let suppressed = self.language_exception_value(previous)?;
                    Error::Thrown(self.create_intrinsic_suppressed_error(error, suppressed, span)?)
                } else {
                    error
                });
            }
        }
        completion
    }
}
