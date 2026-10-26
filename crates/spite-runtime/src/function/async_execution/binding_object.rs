//! KeyedBindingInitialization and object rest through the shared pattern cursor.

use super::{AsyncAwaitAction, AsyncExecutionData, Step, binding, binding_pattern, expression};
use crate::{
    Error, Realm, Value,
    object::{
        async_binding::{BindingChild, BindingCursorKind, ObjectBindingPhase},
        async_expression::ExprRoot,
    },
};
use spite_parser::ast::{BindingPatternKind, PropertyName};

impl Realm {
    pub(super) fn advance_async_object_binding(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Option<Step>, Error> {
        let state = data.pattern_binding.as_ref().expect("object binding state");
        let cursor = state.cursors.last().expect("object binding cursor");
        let pattern = binding_pattern::pattern_at(
            &data.body,
            binding::binding_location(data),
            &data.frames,
            state.root,
            &cursor.path,
        );
        let BindingPatternKind::Object { properties, rest } = &pattern.kind else {
            unreachable!("object binding source")
        };
        let BindingCursorKind::Object(object) = &cursor.kind else {
            unreachable!("object binding cursor")
        };
        let span = cursor.span;
        let index = cursor.index;
        let environment = state.environment.clone();
        if matches!(object.phase, ObjectBindingPhase::Finish) {
            let cursor = data
                .pattern_binding
                .as_mut()
                .expect("object binding state")
                .cursors
                .last_mut()
                .expect("object binding cursor");
            let BindingCursorKind::Object(object) = &mut cursor.kind else {
                unreachable!("object binding cursor")
            };
            let key = object.key.take().expect("completed property key");
            if rest.is_some() {
                self.exclude_rest_key(&mut object.excluded, key, span)?;
            }
            object.phase = ObjectBindingPhase::Key;
            cursor.index += 1;
        } else if let Some(property) = properties.get(index) {
            if matches!(object.phase, ObjectBindingPhase::Key) {
                if matches!(&property.key, PropertyName::Computed(key) if expression::has_supported_await(key))
                {
                    return self
                        .start_async_expression(
                            data,
                            ExprRoot::BindingKey,
                            AsyncAwaitAction::BindingKey,
                        )
                        .map(Some);
                }
                let key = self.binding_property_key(property, span)?;
                let cursor = data
                    .pattern_binding
                    .as_mut()
                    .expect("object binding state")
                    .cursors
                    .last_mut()
                    .expect("object binding cursor");
                let BindingCursorKind::Object(object) = &mut cursor.kind else {
                    unreachable!("object binding cursor")
                };
                object.key = Some(key);
                object.phase = ObjectBindingPhase::Bind;
            } else {
                let key = object.key.as_ref().expect("binding property key");
                // SingleNameBinding resolves before GetV, including with hooks.
                let reference = self
                    .binding_reference(&property.element.pattern, environment.as_ref())?
                    .map(|reference| reference.into_owned(property.element.pattern.span))
                    .transpose()?;
                let value = self.get_property_value(&object.value, key, span)?;
                let cursor = data
                    .pattern_binding
                    .as_mut()
                    .expect("object binding state")
                    .cursors
                    .last_mut()
                    .expect("object binding cursor");
                cursor.reference = reference;
                let BindingCursorKind::Object(object) = &mut cursor.kind else {
                    unreachable!("object binding cursor")
                };
                object.phase = ObjectBindingPhase::Finish;
                if matches!(value, Value::Undefined)
                    && property
                        .element
                        .initializer
                        .as_ref()
                        .is_some_and(expression::has_supported_await)
                {
                    data.pattern_binding
                        .as_mut()
                        .expect("object binding state")
                        .pending_index = Some(index);
                    return self
                        .start_async_expression(
                            data,
                            ExprRoot::BindingDefault,
                            AsyncAwaitAction::BindingDefault,
                        )
                        .map(Some);
                }
                let value = self.binding_element_value(&property.element, value)?;
                let path =
                    binding_pattern::child_path(&cursor.path, BindingChild::Property(index), span)?;
                self.initialize_async_pattern_node(data, path, value)?;
            }
        } else if cursor.rest_pending {
            let rest = rest.as_ref().expect("object rest binding");
            let reference = self
                .binding_reference(rest, environment.as_ref())?
                .map(|reference| reference.into_owned(rest.span))
                .transpose()?;
            let value = self.copy_rest_object(object.value.clone(), &object.excluded, span)?;
            let cursor = data
                .pattern_binding
                .as_mut()
                .expect("object binding state")
                .cursors
                .last_mut()
                .expect("object binding cursor");
            cursor.rest_pending = false;
            cursor.reference = reference;
            let path = binding_pattern::child_path(&cursor.path, BindingChild::Rest, span)?;
            self.initialize_async_pattern_node(data, path, value)?;
        } else {
            data.pattern_binding
                .as_mut()
                .expect("object binding state")
                .cursors
                .pop();
        }
        Ok(None)
    }

    pub(super) fn resume_async_binding_key(
        &mut self,
        data: &mut AsyncExecutionData,
        value: Value,
    ) -> Result<Step, Error> {
        let cursor = data
            .pattern_binding
            .as_mut()
            .expect("object binding key")
            .cursors
            .last_mut()
            .expect("object binding cursor");
        let key = self.property_key(value, cursor.span)?;
        let BindingCursorKind::Object(object) = &mut cursor.kind else {
            unreachable!("computed binding key")
        };
        object.key = Some(key);
        object.phase = ObjectBindingPhase::Bind;
        Ok(Step::Continue)
    }
}
