//! JSON.stringify and its serialization record (25.5.4).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span};
use std::collections::HashSet;

mod output;
mod property;

struct State {
    replacer: Option<Value>,
    properties: Option<Vec<JsString>>,
    gap: Vec<u16>,
    indent: Vec<u16>,
    frames: Vec<Frame>,
    ancestors: HashSet<ObjectHandle>,
    output: Vec<u16>,
}

struct Frame {
    object: ObjectHandle,
    children: Children,
    emitted: bool,
}

enum Children {
    Array { index: u64, length: u64 },
    Object { index: usize, keys: Vec<JsString> },
}

impl Children {
    fn is_array(&self) -> bool {
        matches!(self, Self::Array { .. })
    }
    fn next(&mut self) -> Option<JsString> {
        match self {
            Self::Array { index, length } if *index < *length => {
                let name = JsString::from(index.to_string().as_str());
                *index += 1;
                Some(name)
            }
            Self::Object { index, keys } if *index < keys.len() => {
                let name = keys[*index].clone();
                *index += 1;
                Some(name)
            }
            _ => None,
        }
    }
}

enum Serialized {
    Omit,
    String(JsString),
    Raw(JsString),
    Container(ObjectHandle, bool),
}

impl Realm {
    pub(crate) fn json_stringify(
        &mut self,
        value: Value,
        replacer: Value,
        space: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let mut state = State {
            replacer: None,
            properties: None,
            gap: Vec::new(),
            indent: Vec::new(),
            frames: Vec::new(),
            ancestors: HashSet::new(),
            output: Vec::new(),
        };
        if let Value::Object(object) = &replacer {
            if self.is_callable(&replacer, span)? {
                state.replacer = Some(replacer.clone());
            } else if self
                .object_work(span, |objects, _| Ok(objects.inspect(object)?.is_array()))?
            {
                let length = self.length_of_array_like(object, span)?;
                let mut properties = Vec::new();
                let mut seen = HashSet::new();
                for index in 0..length {
                    self.tick(span)?;
                    let item = self.get_property(
                        object,
                        &JsString::from(index.to_string().as_str()),
                        span,
                    )?;
                    let usable = match &item {
                        Value::String(_) | Value::Number(_) => true,
                        Value::Object(object) => self.object_work(span, |objects, _| {
                            let object = objects.inspect(object)?;
                            Ok(object.string_data().is_some() || object.number_data().is_some())
                        })?,
                        _ => false,
                    };
                    if usable {
                        let key = self.string(item, span)?;
                        seen.try_reserve(1).map_err(|_| capacity_error(span))?;
                        if seen.insert(key.clone()) {
                            properties
                                .try_reserve(1)
                                .map_err(|_| capacity_error(span))?;
                            properties.push(key);
                        }
                    }
                }
                state.properties = Some(properties);
            }
        }
        let mut space = space;
        if let Value::Object(object) = &space {
            let (number, string) = self.object_work(span, |objects, _| {
                let object = objects.inspect(object)?;
                Ok((
                    object.number_data().is_some(),
                    object.string_data().is_some(),
                ))
            })?;
            if number {
                space = Value::Number(self.number(space, span)?);
            } else if string {
                space = Value::String(self.string(space, span)?);
            }
        }
        match space {
            Value::Number(number) if !number.is_nan() && number >= 1.0 => {
                let count = number.trunc().min(10.0) as usize;
                state
                    .gap
                    .try_reserve(count)
                    .map_err(|_| capacity_error(span))?;
                state.gap.resize(count, 0x20);
            }
            Value::String(string) => {
                let units = &string.code_units()[..string.len().min(10)];
                state
                    .gap
                    .try_reserve(units.len())
                    .map_err(|_| capacity_error(span))?;
                state.gap.extend_from_slice(units);
            }
            _ => {}
        }
        let prototype = self.ensure_object_intrinsics(span)?;
        let wrapper = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        self.define_property_or_throw(
            &wrapper,
            JsString::from(""),
            DataDescriptor {
                value: Some(value),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            }
            .into(),
            span,
        )?;
        let root = self.json_serialize_property(&state, &wrapper, &JsString::from(""), span)?;
        if matches!(root, Serialized::Omit) {
            return Ok(Value::Undefined);
        }
        self.json_emit_value(&mut state, root, span)?;
        while let Some(frame) = state.frames.last_mut() {
            self.tick(span)?;
            if let Some(key) = frame.children.next() {
                let holder = frame.object.clone();
                let array = frame.children.is_array();
                let mut value = self.json_serialize_property(&state, &holder, &key, span)?;
                if matches!(value, Serialized::Omit) {
                    if !array {
                        continue;
                    }
                    value = Serialized::Raw(JsString::from("null"));
                }
                self.json_emit_prefix(&mut state, &key, span)?;
                self.json_emit_value(&mut state, value, span)?;
            } else {
                let frame = state
                    .frames
                    .pop()
                    .expect("completed JSON serialization frame");
                let length = state.indent.len() - state.gap.len();
                state.indent.truncate(length);
                if frame.emitted && !state.gap.is_empty() {
                    self.json_append(&mut state.output, &[0x0a], span)?;
                    self.json_append(&mut state.output, &state.indent, span)?;
                }
                self.json_append(
                    &mut state.output,
                    &[if frame.children.is_array() {
                        0x5d
                    } else {
                        0x7d
                    }],
                    span,
                )?;
                state.ancestors.remove(&frame.object);
            }
        }
        Ok(Value::String(JsString::from_code_units(state.output)))
    }

    fn json_emit_value(
        &mut self,
        state: &mut State,
        value: Serialized,
        span: Span,
    ) -> Result<(), Error> {
        match value {
            Serialized::Omit => unreachable!("omitted values are handled by the parent"),
            Serialized::String(string) => self.json_quote(&mut state.output, &string, span)?,
            Serialized::Raw(string) => {
                self.json_append(&mut state.output, string.code_units(), span)?
            }
            Serialized::Container(object, array) => {
                if state.ancestors.contains(&object) {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "JSON value contains a cycle",
                    ));
                }
                state
                    .ancestors
                    .try_reserve(1)
                    .map_err(|_| capacity_error(span))?;
                state.ancestors.insert(object.clone());
                state
                    .indent
                    .try_reserve(state.gap.len())
                    .map_err(|_| capacity_error(span))?;
                state.indent.extend_from_slice(&state.gap);
                let children = if array {
                    Children::Array {
                        index: 0,
                        length: self.length_of_array_like(&object, span)?,
                    }
                } else {
                    let keys = if let Some(properties) = &state.properties {
                        let mut keys = Vec::new();
                        keys.try_reserve(properties.len())
                            .map_err(|_| capacity_error(span))?;
                        keys.extend(properties.iter().cloned());
                        keys
                    } else {
                        self.json_enumerable_keys(&object, span)?
                    };
                    Children::Object { index: 0, keys }
                };
                self.json_append(&mut state.output, &[if array { 0x5b } else { 0x7b }], span)?;
                state
                    .frames
                    .try_reserve(1)
                    .map_err(|_| capacity_error(span))?;
                state.frames.push(Frame {
                    object,
                    children,
                    emitted: false,
                });
            }
        }
        Ok(())
    }

    fn json_emit_prefix(
        &mut self,
        state: &mut State,
        key: &JsString,
        span: Span,
    ) -> Result<(), Error> {
        let frame = state.frames.last_mut().expect("JSON child parent");
        if frame.emitted {
            self.json_append(&mut state.output, &[0x2c], span)?;
        }
        frame.emitted = true;
        if !state.gap.is_empty() {
            self.json_append(&mut state.output, &[0x0a], span)?;
            self.json_append(&mut state.output, &state.indent, span)?;
        }
        if !frame.children.is_array() {
            self.json_quote(&mut state.output, key, span)?;
            self.json_append(&mut state.output, &[0x3a], span)?;
            if !state.gap.is_empty() {
                self.json_append(&mut state.output, &[0x20], span)?;
            }
        }
        Ok(())
    }

    fn json_enumerable_keys(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Vec<JsString>, Error> {
        let mut keys = Vec::new();
        for key in self.own_property_keys(object, span)? {
            self.tick(span)?;
            let spite_core::PropertyKey::String(key) = key else {
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
        Ok(keys)
    }
}

fn capacity_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "JSON serialization exceeds platform capacity".into(),
    }
}
