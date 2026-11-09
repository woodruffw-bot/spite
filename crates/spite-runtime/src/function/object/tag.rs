//! Object.prototype.toString and observable tag lookup (20.1.3.6).

use crate::{Error, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    pub(crate) fn object_to_string(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        match value {
            Value::Undefined => return self.object_builtin_tag_string("[object Undefined]", span),
            Value::Null => return self.object_builtin_tag_string("[object Null]", span),
            _ => {}
        }
        let Value::Object(object) = self.box_primitive(value, span)? else {
            unreachable!("ToObject");
        };
        // IsArray precedes even the callable/direct-slot fallback and the
        // observable tag getter, so a revoked Proxy throws at this point.
        let array = self.is_array(&Value::Object(object.clone()), span)?;
        // IsArray validated this handle without any user call or collection.
        // The following physical slot read adds no second ordinary work tick;
        // only the static fallback survives into the observable tag Get.
        let builtin_tag = {
            let record = self
                .objects
                .inspect(&object)
                .expect("IsArray validated the local handle");
            if array {
                "[object Array]"
            } else if record.is_arguments() {
                "[object Arguments]"
            } else if record.is_callable() {
                "[object Function]"
            } else if record.is_error() {
                "[object Error]"
            } else if record.date_value().is_some() {
                "[object Date]"
            } else if record.regexp_data().is_some() {
                "[object RegExp]"
            } else if record.boolean_data().is_some() {
                "[object Boolean]"
            } else if record.number_data().is_some() {
                "[object Number]"
            } else if record.string_data().is_some() {
                "[object String]"
            } else {
                "[object Object]"
            }
        };
        let tag = self.get_property(&object, &WellKnownSymbol::ToStringTag.symbol(), span)?;
        // No ToPrimitive or ToString is performed on non-string tags.
        match tag {
            Value::String(tag) => self.object_tag_string(&tag, span),
            _ => self.object_builtin_tag_string(builtin_tag, span),
        }
    }

    fn check_object_tag_length(&mut self, length: usize, span: Span) -> Result<(), Error> {
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(object_tag_limit(span));
        }
        self.object_work(span, |_, budget| budget.charge(length))
    }

    fn object_builtin_tag_string(
        &mut self,
        literal: &'static str,
        span: Span,
    ) -> Result<Value, Error> {
        // Builtin spellings are ASCII; construct their complete UTF-16 String
        // in one Arc allocation, after the ordinary output check and work.
        self.check_object_tag_length(literal.len(), span)?;
        Ok(Value::String(JsString::from(literal)))
    }

    fn object_tag_string(&mut self, tag: &JsString, span: Span) -> Result<Value, Error> {
        let length = tag
            .len()
            .checked_add(9)
            .ok_or_else(|| object_tag_limit(span))?;
        self.check_object_tag_length(length, span)?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(length)
            .map_err(|_| object_tag_limit(span))?;
        units.extend("[object ".encode_utf16());
        units.extend_from_slice(tag.code_units());
        units.push(u16::from(b']'));
        Ok(Value::String(JsString::from_code_units(units)))
    }
}

fn object_tag_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "Object tag string output limit exceeded".into(),
    }
}
