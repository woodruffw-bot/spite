//! Array length conversion outside storage borrows (10.4.2.4).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DescriptorKind, PropertyDescriptor},
    value::to_uint32,
};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn convert_array_length(
        &mut self,
        object: &ObjectHandle,
        key: &JsString,
        descriptor: &mut PropertyDescriptor,
        span: Span,
    ) -> Result<(), Error> {
        if key.code_units() != [0x6c, 0x65, 0x6e, 0x67, 0x74, 0x68] {
            return Ok(());
        }
        let DescriptorKind::Data {
            value: Some(value), ..
        } = &mut descriptor.kind
        else {
            return Ok(());
        };
        if !self.object_work(span, |objects, _| Ok(objects.inspect(object)?.is_array()))? {
            return Ok(());
        }
        self.object_work(span, |_, budget| budget.value(value))?;
        // Both conversions use the original value. They can call user code
        // twice and mutate the array; storage reads its current length only
        // after both complete. ToUint32 may produce a different first Number.
        let new_length = to_uint32(self.number(value.clone(), span)?);
        let number_length = self.number(std::mem::replace(value, Value::Undefined), span)?;
        if f64::from(new_length) != number_length {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "invalid Array length",
            ));
        }
        *value = Value::Number(f64::from(new_length));
        Ok(())
    }
}

#[cfg(test)]
mod tests;
