//! SharedArrayBuffer construction, branded lengths and grow (25.2).

use super::global_metadata::Member;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{SharedArrayBufferData, SharedBufferError},
};
use spite_core::{JsString, Span};

impl Realm {
    pub(super) fn shared_array_buffer_constructor(
        &mut self,
        new_target: ObjectHandle,
        length: Value,
        options: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let byte_length = self.array_buffer_index(length, span)?;
        let maximum = self.array_buffer_max_byte_length_option(options, span)?;
        // AllocateSharedArrayBuffer checks this before OrdinaryCreateFromConstructor.
        if maximum.is_some_and(|maximum| byte_length > maximum) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "SharedArrayBuffer length exceeds maxByteLength",
            ));
        }
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.check_function_realm(&new_target, span)?;
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .global_metadata
                .shared_array_buffer_prototype
                .clone()
        };
        self.array_buffer_work(span, |objects, budget| {
            objects.create_shared_array_buffer(&prototype, byte_length, maximum, budget)
        })
        .map(Value::Object)
    }

    fn shared_buffer_brand_error(span: Span) -> Error {
        Self::exception(
            ExceptionKind::TypeError,
            span,
            "operation requires a SharedArrayBuffer",
        )
    }

    fn shared_buffer_error(error: SharedBufferError, span: Span) -> Error {
        match error {
            SharedBufferError::Poisoned => Error::Host {
                span,
                message: "shared backing storage lock is poisoned".into(),
            },
            SharedBufferError::NotGrowable => Self::exception(
                ExceptionKind::TypeError,
                span,
                "SharedArrayBuffer is not growable",
            ),
            SharedBufferError::OutOfBounds => Self::exception(
                ExceptionKind::RangeError,
                span,
                "SharedArrayBuffer growth is outside its current/maximum lengths",
            ),
        }
    }

    fn shared_buffer_data(
        &mut self,
        value: Value,
        span: Span,
    ) -> Result<SharedArrayBufferData, Error> {
        let Value::Object(object) = value else {
            return Err(Self::shared_buffer_brand_error(span));
        };
        self.object_work(span, |objects, _| {
            Ok(objects
                .inspect(&object)?
                .shared_array_buffer_data()
                .cloned())
        })?
        .ok_or_else(|| Self::shared_buffer_brand_error(span))
    }

    pub(super) fn shared_array_buffer_method(
        &mut self,
        member: Member,
        receiver: Value,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let data = self.shared_buffer_data(receiver, span)?;
        let (length, maximum) = data
            .metadata()
            .map_err(|error| Self::shared_buffer_error(error, span))?;
        match member {
            Member::SharedArrayBufferByteLength => Ok(Value::Number(length as f64)),
            Member::SharedArrayBufferGrowable => Ok(Value::Boolean(maximum.is_some())),
            Member::SharedArrayBufferMaxByteLength => {
                Ok(Value::Number(maximum.unwrap_or(length) as f64))
            }
            Member::SharedArrayBufferGrow => {
                if maximum.is_none() {
                    return Err(Self::shared_buffer_error(
                        SharedBufferError::NotGrowable,
                        span,
                    ));
                }
                let length = self.array_buffer_index(value, span)?;
                self.object_work(span, |_, budget| budget.charge(1))?;
                data.grow(length)
                    .map_err(|error| Self::shared_buffer_error(error, span))?;
                Ok(Value::Undefined)
            }
            _ => unreachable!("SharedArrayBuffer member"),
        }
    }
}
