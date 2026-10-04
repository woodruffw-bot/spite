//! InternalizeJSONProperty uses explicit frames, preserving observable step order.

use crate::{Error, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, PropertyKey, Span};
use spite_parser::json::{JsonDocument, JsonKind};

struct Snapshot<'a> {
    source: &'a JsString,
    document: &'a JsonDocument,
    values: &'a [Value],
}

struct Frame {
    holder: ObjectHandle,
    name: JsString,
    value: Value,
    context: ObjectHandle,
    record: Option<usize>,
    children: Children,
}

enum Children {
    None,
    Array { index: u64, length: u64 },
    Object { index: usize, keys: Vec<JsString> },
}

impl Realm {
    pub(super) fn json_revive(
        &mut self,
        source: &JsString,
        document: &JsonDocument,
        values: &[Value],
        reviver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self.ensure_object_intrinsics(span)?;
        let root = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        self.json_create_data(
            &root,
            JsString::from(""),
            values[document.root].clone(),
            span,
        )?;
        // The flat document plus original values is the immutable parse snapshot.
        // Duplicate names resolve to their last source occurrence when descended.
        let snapshot = Snapshot {
            source,
            document,
            values,
        };
        let frame = self.json_reviver_frame(
            root,
            JsString::from(""),
            Some(document.root),
            &snapshot,
            span,
        )?;
        let mut frames = Vec::new();
        push_frame(&mut frames, frame, span)?;
        loop {
            self.tick(span)?;
            let frame = frames.last_mut().expect("active JSON reviver frame");
            let child = match &mut frame.children {
                Children::None => None,
                Children::Array { index, length } => {
                    if *index == *length {
                        None
                    } else {
                        let name = JsString::from(index.to_string().as_str());
                        let record = frame.record.and_then(|record| {
                            let JsonKind::Array(elements) = &document.nodes[record].kind else {
                                unreachable!("array parse snapshot");
                            };
                            usize::try_from(*index)
                                .ok()
                                .and_then(|i| elements.get(i).copied())
                        });
                        *index += 1;
                        Some((name, record))
                    }
                }
                Children::Object { index, keys } => {
                    if *index == keys.len() {
                        None
                    } else {
                        let name = keys[*index].clone();
                        let mut child_record = None;
                        if let Some(record) = frame.record {
                            let JsonKind::Object(entries) = &document.nodes[record].kind else {
                                unreachable!("object parse snapshot");
                            };
                            for (key, child) in entries.iter().rev() {
                                self.tick(span)?;
                                if key == &name {
                                    child_record = Some(*child);
                                    break;
                                }
                            }
                        }
                        *index += 1;
                        Some((name, child_record))
                    }
                }
            };
            if let Some((name, record)) = child {
                let Value::Object(holder) = &frame.value else {
                    unreachable!("only objects have reviver children");
                };
                let child =
                    self.json_reviver_frame(holder.clone(), name, record, &snapshot, span)?;
                push_frame(&mut frames, child, span)?;
                continue;
            }
            let frame = frames.pop().expect("completed JSON reviver frame");
            let value = self.call(
                reviver.clone(),
                Value::Object(frame.holder.clone()),
                vec![
                    Value::String(frame.name.clone()),
                    frame.value,
                    Value::Object(frame.context),
                ],
                span,
            )?;
            if frames.is_empty() {
                return Ok(value);
            }
            // InternalizeJSONProperty ignores false from Delete/CreateDataProperty,
            // including nonconfigurable properties changed by earlier callbacks.
            // Abrupt completions still propagate, and setters are never invoked.
            if matches!(value, Value::Undefined) {
                self.delete_property_value(&Value::Object(frame.holder), &frame.name, span)?;
            } else {
                self.json_create_data(&frame.holder, frame.name, value, span)?;
            }
        }
    }

    fn json_reviver_frame(
        &mut self,
        holder: ObjectHandle,
        name: JsString,
        record: Option<usize>,
        snapshot: &Snapshot<'_>,
        span: Span,
    ) -> Result<Frame, Error> {
        let Snapshot {
            source,
            document,
            values,
        } = snapshot;
        let value = self.get_property(&holder, &name, span)?;
        let prototype = self.ensure_object_intrinsics(span)?;
        let context = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let record = record.filter(|index| values[*index].same_value(&value));
        if !matches!(value, Value::Object(_)) {
            if let Some(record) = record {
                let units = &source.code_units()[document.nodes[record].source.clone()];
                let mut text = Vec::new();
                text.try_reserve(units.len())
                    .map_err(|_| capacity_error(span))?;
                text.extend_from_slice(units);
                let text = JsString::from_code_units(text);
                self.check_json_string(&text, span)?;
                self.json_create_data(
                    &context,
                    JsString::from("source"),
                    Value::String(text),
                    span,
                )?;
            }
        }
        let children = if let Value::Object(object) = &value {
            let array =
                self.object_work(span, |objects, _| Ok(objects.inspect(object)?.is_array()))?;
            if array {
                Children::Array {
                    index: 0,
                    length: self.length_of_array_like(object, span)?,
                }
            } else {
                // EnumerableOwnProperties(key) snapshots all enumerable string
                // keys now, before any child getter or reviver runs.
                let mut keys = Vec::new();
                for key in self.own_property_keys(object, span)? {
                    self.tick(span)?;
                    let PropertyKey::String(key) = key else {
                        continue;
                    };
                    if self
                        .own_property_descriptor(object, &key, span)?
                        .is_some_and(|p| p.enumerable())
                    {
                        keys.try_reserve(1).map_err(|_| capacity_error(span))?;
                        keys.push(key);
                    }
                }
                Children::Object { index: 0, keys }
            }
        } else {
            Children::None
        };
        Ok(Frame {
            holder,
            name,
            value,
            context,
            record,
            children,
        })
    }

    fn json_create_data(
        &mut self,
        object: &ObjectHandle,
        key: JsString,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        self.object_work(span, |objects, budget| {
            objects.define(
                object,
                key,
                DataDescriptor {
                    value: Some(value),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                },
                budget,
            )
        })?;
        Ok(())
    }
}

fn push_frame(frames: &mut Vec<Frame>, frame: Frame, span: Span) -> Result<(), Error> {
    frames.try_reserve(1).map_err(|_| capacity_error(span))?;
    frames.push(frame);
    Ok(())
}

fn capacity_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "JSON reviver tree exceeds platform capacity".into(),
    }
}
