//! Sparse Array exotic storage (10.4.2.1/2/4), after Realm length conversion.

use super::{
    Budget, DataProperty, DescriptorKind, Error, OrdinaryObject, Property, PropertyDescriptor,
    PropertyLimit, array_index,
};
use crate::Value;
use spite_core::{PropertyKey, PropertyKeyRef};

impl OrdinaryObject {
    pub(super) fn initialize_array_elements(
        &mut self,
        values: Vec<Value>,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        // Only an unexposed, extensible Array with its writable final length and
        // no indexed properties may use this path. Existing named metadata is
        // retained. Numeric keys are unique and below length by construction.
        if !self.array || !self.extensible {
            return Err(Error::WrongKind);
        }
        let (length, writable) = self.array_length();
        if !writable || usize::try_from(length).ok() != Some(values.len()) {
            return Err(Error::WrongKind);
        }
        budget.charge(self.properties.len())?;
        if self
            .properties
            .iter()
            .any(|(key, _)| array_index(key).is_some())
        {
            return Err(Error::WrongKind);
        }
        let count = self
            .properties
            .len()
            .checked_add(values.len())
            .ok_or(Error::PropertyLimit)?;
        if self.max_properties.is_some_and(|limit| count > limit) {
            return Err(Error::PropertyLimit);
        }
        // Complete every fallible charge/reservation before changing the record.
        // No ordinary property lookup or ArraySetLength is required for these
        // fresh own elements (RegExpBuiltinExec, 22.2.7.2).
        for (index, value) in values.iter().enumerate() {
            budget.charge(
                index
                    .checked_ilog10()
                    .map_or(1, |digits| digits as usize + 1)
                    + 1,
            )?;
            budget.value(value)?;
        }
        self.properties
            .try_reserve(values.len())
            .map_err(|_| Error::PropertyLimit)?;
        for (index, value) in values.into_iter().enumerate() {
            self.properties.push((
                PropertyKey::from(spite_core::JsString::from(index.to_string().as_str())),
                Property::Data(DataProperty {
                    value,
                    writable: true,
                    enumerable: true,
                    configurable: true,
                }),
            ));
        }
        Ok(())
    }

    pub(super) fn prepare_array_definition(
        &self,
        key: &PropertyKey,
        descriptor: &PropertyDescriptor,
        budget: &mut Budget,
    ) -> Result<Option<u32>, Error> {
        if !self.array {
            return Ok(None);
        }
        // Charge length lookups and updates before touching the descriptor.
        let count = self.properties.len();
        budget.charge(count.checked_mul(16).ok_or(Error::WorkLimit)?)?;
        if !is_length(key) {
            return Ok(None);
        }
        let DescriptorKind::Data {
            value: Some(value), ..
        } = &descriptor.kind
        else {
            return Ok(None);
        };
        let Value::Number(number) = value else {
            return Err(Error::UnnormalizedArrayLength);
        };
        if !number.is_finite()
            || number.fract() != 0.0
            || !(0.0..=f64::from(u32::MAX)).contains(number)
        {
            return Err(Error::UnnormalizedArrayLength);
        }
        let new_length = *number as u32;
        let (old_length, writable) = self.array_length();
        if new_length < old_length && writable {
            // Each deletion finds the largest remaining numeric index and may
            // shift the property vector. Charge a conservative quadratic bound
            // before reducing length, so a work abort cannot break invariants.
            let work = count
                .checked_mul(count)
                .and_then(|n| n.checked_mul(16))
                .ok_or(Error::WorkLimit)?;
            budget.charge(work)?;
        }
        Ok(Some(new_length))
    }

    pub(super) fn define_array_property(
        &mut self,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        new_length: Option<u32>,
    ) -> Result<bool, PropertyLimit> {
        if is_length(&key) {
            return match new_length {
                Some(length) => self.set_array_length(key, descriptor, length),
                None => self.define_own_property(key, descriptor),
            };
        }
        let Some(index) = array_index(&key) else {
            return self.define_own_property(key, descriptor);
        };
        let (length, writable) = self.array_length();
        if index >= length && !writable {
            return Ok(false);
        }
        let allowed = self.define_own_property(key, descriptor)?;
        if allowed && index >= length {
            self.array_length_mut().value = Value::Number(f64::from(index + 1));
        }
        Ok(allowed)
    }

    fn set_array_length(
        &mut self,
        key: PropertyKey,
        mut descriptor: PropertyDescriptor,
        new_length: u32,
    ) -> Result<bool, PropertyLimit> {
        let DescriptorKind::Data { value, writable } = &mut descriptor.kind else {
            unreachable!("validated length value");
        };
        *value = Some(Value::Number(f64::from(new_length)));
        let (old_length, old_writable) = self.array_length();
        if new_length >= old_length {
            return self.define_own_property(key, descriptor);
        }
        if !old_writable {
            return Ok(false);
        }
        let make_read_only = *writable == Some(false);
        if make_read_only {
            *writable = Some(true);
        }
        if !self.define_own_property(key, descriptor)? {
            return Ok(false);
        }
        loop {
            let largest = self
                .properties
                .iter()
                .enumerate()
                .filter_map(|(slot, (key, _))| {
                    array_index(key)
                        .filter(|&index| index >= new_length)
                        .map(|index| (index, slot))
                })
                .max_by_key(|&(index, _)| index);
            let Some((index, slot)) = largest else {
                break;
            };
            if !self.properties[slot].1.configurable() {
                let length = self.array_length_mut();
                length.value = Value::Number(f64::from(index + 1));
                if make_read_only {
                    length.writable = false;
                }
                return Ok(false);
            }
            self.properties.remove(slot);
        }
        if make_read_only {
            self.array_length_mut().writable = false;
        }
        Ok(true)
    }

    fn array_length(&self) -> (u32, bool) {
        let (_, Property::Data(data)) = self
            .properties
            .iter()
            .find(|(key, _)| is_length(key))
            .expect("Array length")
        else {
            unreachable!("Array length is a data property");
        };
        let Value::Number(length) = &data.value else {
            unreachable!("Array length is an integral Number");
        };
        (*length as u32, data.writable)
    }

    fn array_length_mut(&mut self) -> &mut super::DataProperty {
        let (_, Property::Data(data)) = self
            .properties
            .iter_mut()
            .find(|(key, _)| is_length(key))
            .expect("Array length")
        else {
            unreachable!("Array length is a data property");
        };
        data
    }
}

pub(super) fn is_length<'key>(key: impl Into<PropertyKeyRef<'key>>) -> bool {
    key.into()
        .as_string()
        .is_some_and(|key| key.code_units() == [0x6c, 0x65, 0x6e, 0x67, 0x74, 0x68])
}

#[cfg(test)]
mod tests;
