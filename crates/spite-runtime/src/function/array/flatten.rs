//! FlattenIntoArray, flat, and flatMap (23.1.3.13–14).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

struct Frame {
    source: ObjectHandle,
    length: u64,
    index: u64,
    // None denotes +Infinity. A finite depth can saturate at usize::MAX:
    // exhausting that depth would first require an unaddressable frame vector.
    depth: Option<usize>,
}

impl Realm {
    pub(crate) fn array_flat(
        &mut self,
        receiver: Value,
        depth: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(source) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&source, span)?;
        let depth = if matches!(depth, Value::Undefined) {
            Some(1)
        } else {
            let number = self.number(depth, span)?;
            if number == f64::INFINITY {
                None
            } else {
                Some(number.trunc().max(0.0) as usize)
            }
        };
        let target = self.array_species_create(&source, 0, span)?;
        self.flatten_into_array(
            &target,
            Frame {
                source,
                length,
                index: 0,
                depth,
            },
            0,
            None,
            span,
        )?;
        Ok(Value::Object(target))
    }

    pub(crate) fn array_flat_map(
        &mut self,
        receiver: Value,
        mapper: Value,
        this_arg: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(source) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&source, span)?;
        if !self.is_callable(&mapper, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Array mapper is not callable",
            ));
        }
        let Value::Object(mapper) = mapper else {
            unreachable!("callable object");
        };
        let target = self.array_species_create(&source, 0, span)?;
        self.flatten_into_array(
            &target,
            Frame {
                source,
                length,
                index: 0,
                depth: Some(1),
            },
            0,
            Some((mapper, this_arg)),
            span,
        )?;
        Ok(Value::Object(target))
    }

    fn flatten_into_array(
        &mut self,
        target: &ObjectHandle,
        frame: Frame,
        mut target_index: u64,
        mapper: Option<(ObjectHandle, Value)>,
        span: Span,
    ) -> Result<u64, Error> {
        // 23.1.3.13.1: keep recursive source visits in an explicit stack. Nested
        // arrays snapshot their own length on entry; presence and Get stay live.
        let mut stack = Vec::new();
        Self::push_flatten_frame(&mut stack, frame, span)?;
        while let Some(frame) = stack.last_mut() {
            self.tick(span)?;
            if frame.index == frame.length {
                stack.pop();
                continue;
            }
            let source = frame.source.clone();
            let source_index = frame.index;
            let depth = frame.depth;
            frame.index += 1;
            let key = JsString::from(source_index.to_string().as_str());
            if !self.has_property(&source, &key, span)? {
                continue;
            }
            let mut element = self.get_property(&source, &key, span)?;
            // The optional mapper applies only to the original source, not to
            // elements of the arrays it returns (flatMap flattens one level).
            if let Some((mapper, this_arg)) = mapper.as_ref().filter(|_| stack.len() == 1) {
                element = self.call(
                    Value::Object(mapper.clone()),
                    this_arg.clone(),
                    vec![
                        element,
                        Value::Number(source_index as f64),
                        Value::Object(source),
                    ],
                    span,
                )?;
            }
            let child = match &element {
                Value::Object(child) if depth != Some(0) => self
                    .object_work(span, |objects, _| Ok(objects.inspect(child)?.is_array()))?
                    .then(|| child.clone()),
                _ => None,
            };
            if let Some(child) = child {
                let length = self.length_of_array_like(&child, span)?;
                Self::push_flatten_frame(
                    &mut stack,
                    Frame {
                        source: child,
                        length,
                        index: 0,
                        depth: depth.map(|depth| depth - 1),
                    },
                    span,
                )?;
                continue;
            }
            if target_index >= 9_007_199_254_740_991 {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Array flattening exceeds the maximum safe integer index",
                ));
            }
            self.create_array_element(target, target_index, element, span)?;
            target_index += 1;
        }
        // No final length Set: Array definitions grow length automatically,
        // while custom species results retain their ordinary length property.
        Ok(target_index)
    }

    fn push_flatten_frame(stack: &mut Vec<Frame>, frame: Frame, span: Span) -> Result<(), Error> {
        stack.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "Array flattening traversal allocation failed".into(),
        })?;
        stack.push(frame);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_integer_overflow_follows_the_element_read_and_keeps_prior_definitions() {
        let mut realm = Realm::default();
        let Value::Object(source) = realm
            .eval("var reads='',target={},source={length:2,get 0(){reads+='0';return 7;},get 1(){reads+='1';return 8;}};source")
            .unwrap()
        else {
            panic!("source object");
        };
        let Value::Object(target) = realm.eval("target").unwrap() else {
            panic!("target object");
        };
        assert!(matches!(
            realm.flatten_into_array(
                &target,
                Frame {
                    source,
                    length: 2,
                    index: 0,
                    depth: Some(0),
                },
                9_007_199_254_740_990,
                None,
                Span::new(0, 0),
            ),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(
            realm.eval("reads==='01' && target[9007199254740990]===7 && !Object.hasOwn(target,'9007199254740991')"),
            Ok(Value::Boolean(true))
        );
    }
}
