//! Set-like records and ordered combination/predicate algorithms (24.2.1, 24.2.4).

use super::*;

struct SetRecord {
    object: ObjectHandle,
    size: f64,
    has: Value,
    keys: Value,
}

impl Realm {
    fn get_set_record(&mut self, other: Value, span: Span) -> Result<SetRecord, Error> {
        let Value::Object(object) = other else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "set-like value is not an object",
            ));
        };
        let size = self.get_property(&object, &JsString::from("size"), span)?;
        let size = self.number(size, span)?;
        if size.is_nan() {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "set-like size is NaN",
            ));
        }
        let size = size.trunc();
        if size < 0.0 {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "set-like size is negative",
            ));
        }
        let has = self.get_property(&object, &JsString::from("has"), span)?;
        if !self.is_callable(&has, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "set-like has is not callable",
            ));
        }
        let keys = self.get_property(&object, &JsString::from("keys"), span)?;
        if !self.is_callable(&keys, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "set-like keys is not callable",
            ));
        }
        Ok(SetRecord {
            object,
            size,
            has,
            keys,
        })
    }

    fn set_result(&mut self, data: SetData, span: Span) -> Result<Value, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .set
            .prototype
            .clone();
        self.object_work(span, |objects, budget| {
            objects.create_set(&prototype, data, budget)
        })
        .map(Value::Object)
    }

    fn set_copy(&mut self, set: &ObjectHandle, span: Span) -> Result<SetData, Error> {
        self.object_work(span, |objects, budget| objects.set_data(set)?.copy(budget))
    }

    fn set_has_internal(
        &mut self,
        set: &ObjectHandle,
        value: &Value,
        span: Span,
    ) -> Result<bool, Error> {
        self.object_work(span, |objects, budget| {
            objects.set_data(set)?.has(value, budget)
        })
    }

    pub(crate) fn set_combine(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        other: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let set = self.set_receiver(receiver, span)?;
        let other = self.get_set_record(other, span)?;
        let size = self.object_work(span, |objects, budget| {
            budget.charge(1)?;
            Ok(objects.set_data(&set)?.size())
        })? as f64;
        match builtin {
            Builtin::SetDifference | Builtin::SetIntersection => {
                let mut result = if matches!(builtin, Builtin::SetDifference) {
                    self.set_copy(&set, span)?
                } else {
                    SetData::default()
                };
                if size <= other.size {
                    let mut index = 0;
                    loop {
                        self.tick(span)?;
                        // Difference traverses its fixed snapshot; intersection
                        // resumes the live receiver after every observable call.
                        let next = if matches!(builtin, Builtin::SetDifference) {
                            self.object_work(span, |_, budget| result.next(index, budget))?
                        } else {
                            self.object_work(span, |objects, budget| {
                                objects.set_data(&set)?.next(index, budget)
                            })?
                        };
                        let Some((next, value)) = next else {
                            break;
                        };
                        index = next;
                        let in_other = self
                            .call(
                                other.has.clone(),
                                Value::Object(other.object.clone()),
                                vec![value.clone()],
                                span,
                            )?
                            .to_boolean();
                        if in_other {
                            self.object_work(span, |_, budget| {
                                if matches!(builtin, Builtin::SetDifference) {
                                    result.delete(&value, budget).map(|_| ())
                                } else {
                                    result.insert(value, budget)
                                }
                            })?;
                        }
                    }
                } else {
                    let mut iterator = self.get_iterator_from_method(
                        Value::Object(other.object),
                        other.keys,
                        span,
                    )?;
                    loop {
                        self.tick(span)?;
                        let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                            break;
                        };
                        if matches!(builtin, Builtin::SetDifference) {
                            self.object_work(span, |_, budget| result.delete(&value, budget))?;
                        } else if self.set_has_internal(&set, &value, span)? {
                            self.object_work(span, |_, budget| result.insert(value, budget))?;
                        }
                    }
                }
                self.set_result(result, span)
            }
            Builtin::SetUnion | Builtin::SetSymmetricDifference => {
                // keys and cached next acquisition may mutate the receiver.
                // The specification takes this snapshot after acquisition.
                let mut iterator =
                    self.get_iterator_from_method(Value::Object(other.object), other.keys, span)?;
                let mut result = self.set_copy(&set, span)?;
                loop {
                    self.tick(span)?;
                    let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                        break;
                    };
                    if matches!(builtin, Builtin::SetSymmetricDifference)
                        && self.set_has_internal(&set, &value, span)?
                    {
                        self.object_work(span, |_, budget| result.delete(&value, budget))?;
                    } else {
                        self.object_work(span, |_, budget| result.insert(value, budget))?;
                    }
                }
                self.set_result(result, span)
            }
            Builtin::SetIsSubsetOf | Builtin::SetIsSupersetOf | Builtin::SetIsDisjointFrom => {
                if matches!(builtin, Builtin::SetIsSubsetOf) && size > other.size
                    || matches!(builtin, Builtin::SetIsSupersetOf) && size < other.size
                {
                    return Ok(Value::Boolean(false));
                }
                if matches!(builtin, Builtin::SetIsSubsetOf)
                    || matches!(builtin, Builtin::SetIsDisjointFrom) && size <= other.size
                {
                    let mut index = 0;
                    loop {
                        self.tick(span)?;
                        let Some((next, value)) = self.object_work(span, |objects, budget| {
                            objects.set_data(&set)?.next(index, budget)
                        })?
                        else {
                            return Ok(Value::Boolean(true));
                        };
                        index = next;
                        let in_other = self
                            .call(
                                other.has.clone(),
                                Value::Object(other.object.clone()),
                                vec![value],
                                span,
                            )?
                            .to_boolean();
                        if matches!(builtin, Builtin::SetIsSubsetOf) && !in_other
                            || matches!(builtin, Builtin::SetIsDisjointFrom) && in_other
                        {
                            return Ok(Value::Boolean(false));
                        }
                    }
                }
                let mut iterator =
                    self.get_iterator_from_method(Value::Object(other.object), other.keys, span)?;
                loop {
                    self.tick(span)?;
                    let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                        return Ok(Value::Boolean(true));
                    };
                    let in_this = self.set_has_internal(&set, &value, span)?;
                    if matches!(builtin, Builtin::SetIsSupersetOf) && !in_this
                        || matches!(builtin, Builtin::SetIsDisjointFrom) && in_this
                    {
                        self.iterator_close(&iterator, span)?;
                        return Ok(Value::Boolean(false));
                    }
                }
            }
            _ => unreachable!("Set combination method"),
        }
    }
}
