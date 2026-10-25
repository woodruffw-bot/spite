//! Object prototype and extensibility APIs (20.1.2.12/16/20/23).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::Span;

impl Realm {
    pub(crate) fn object_get_prototype_of(
        &mut self,
        target: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(target, span)? else {
            unreachable!("ToObject");
        };
        self.get_prototype_of(&object, span)
            .map(|prototype| prototype.map_or(Value::Null, Value::Object))
    }

    pub(crate) fn object_set_prototype_of(
        &mut self,
        target: Value,
        prototype: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // Validate proto even for a primitive target; never coerce or box it.
        Self::require_object_coercible(&target, span)?;
        let prototype = match prototype {
            Value::Object(prototype) => Some(prototype),
            Value::Null => None,
            _ => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "prototype must be an object or null",
                ));
            }
        };
        let Value::Object(object) = &target else {
            return Ok(target);
        };
        let changed = self.set_prototype_of(object, prototype, span)?;
        if !changed {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "prototype change was rejected",
            ));
        }
        Ok(target)
    }

    pub(crate) fn object_extensibility(
        &mut self,
        target: Value,
        prevent: bool,
        span: Span,
    ) -> Result<Value, Error> {
        if prevent {
            if let Value::Object(object) = &target
                && !self.prevent_extensions(object, span)?
            {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "preventExtensions was rejected",
                ));
            }
            return Ok(target);
        }
        let Value::Object(object) = target else {
            return Ok(Value::Boolean(false));
        };
        self.is_extensible(&object, span).map(Value::Boolean)
    }
}
