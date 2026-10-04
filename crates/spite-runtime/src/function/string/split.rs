//! String.prototype.split: Symbol.split delegation and UTF-16 splitting (22.1.3.23).

use crate::{Error, Realm, Value, value::to_uint32};
use spite_core::{Span, WellKnownSymbol};

impl Realm {
    pub(crate) fn string_split(
        &mut self,
        receiver: Value,
        separator: Value,
        limit: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        // Edition 17 only looks up Symbol.split on object separators. Primitive
        // prototype hooks are ignored, including for Strings and BigInts.
        if matches!(separator, Value::Object(_)) {
            if let Some(method) =
                self.get_method(&separator, &WellKnownSymbol::Split.symbol(), span)?
            {
                // Delegate with the original receiver and limit, before any
                // string or numeric conversion. Return the hook's value as-is.
                return self.call(method, separator, vec![receiver, limit], span);
            }
        }
        let string = self.string(receiver, span)?;
        let result = self.create_intrinsic_array(0, span)?;
        let limit = if matches!(limit, Value::Undefined) {
            u32::MAX
        } else {
            to_uint32(self.number(limit, span)?)
        };
        let undefined_separator = matches!(separator, Value::Undefined);
        // ToString(separator) follows ToUint32(limit), even for a zero limit.
        let separator = self.string(separator, span)?;
        if limit == 0 {
            return Ok(Value::Object(result));
        }
        if undefined_separator {
            self.create_array_element(&result, 0, Value::String(string), span)?;
            return Ok(Value::Object(result));
        }
        if separator.is_empty() {
            // Empty separators split code units, including halves of pairs.
            // Empty strings produce no elements and no leading/trailing empty.
            for index in 0..string.len().min(limit as usize) {
                self.tick(span)?;
                let value = self.copy_string_units(&string.code_units()[index..index + 1], span)?;
                self.create_array_element(&result, index as u64, value, span)?;
            }
            return Ok(Value::Object(result));
        }
        let mut start = 0;
        let mut count = 0;
        while let Some(position) = self.find_string(&string, &separator, start, false, span)? {
            self.tick(span)?;
            let value = self.copy_string_units(&string.code_units()[start..position], span)?;
            self.create_array_element(&result, u64::from(count), value, span)?;
            count += 1;
            if count == limit {
                return Ok(Value::Object(result));
            }
            start = position + separator.len();
        }
        let value = self.copy_string_units(&string.code_units()[start..], span)?;
        self.create_array_element(&result, u64::from(count), value, span)?;
        Ok(Value::Object(result))
    }
}
