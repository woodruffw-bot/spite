//! Iterative object internal methods and Proxy invariant validation (10.5).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{
        self, DataDescriptor, DescriptorKind, Property, PropertyDescriptor, ProxySnapshot,
        ProxyState,
    },
};
use spite_core::{JsString, PropertyKey, PropertyKeyRef, Span};

struct ObjectRequest {
    object: ObjectHandle,
    operation: ObjectOperation,
}

enum ObjectOperation {
    GetPrototypeOf,
    SetPrototypeOf(Option<ObjectHandle>),
    IsExtensible,
    PreventExtensions,
    GetOwnProperty(PropertyKey),
    DefineOwnProperty(PropertyKey, PropertyDescriptor),
    HasProperty(PropertyKey),
    Get(PropertyKey, Value),
    Set(PropertyKey, Value, Value),
    Delete(PropertyKey),
    OwnPropertyKeys,
}

impl ObjectOperation {
    fn trap_name(&self) -> &'static str {
        match self {
            Self::GetPrototypeOf => "getPrototypeOf",
            Self::SetPrototypeOf(_) => "setPrototypeOf",
            Self::IsExtensible => "isExtensible",
            Self::PreventExtensions => "preventExtensions",
            Self::GetOwnProperty(_) => "getOwnPropertyDescriptor",
            Self::DefineOwnProperty(..) => "defineProperty",
            Self::HasProperty(_) => "has",
            Self::Get(..) => "get",
            Self::Set(..) => "set",
            Self::Delete(_) => "deleteProperty",
            Self::OwnPropertyKeys => "ownKeys",
        }
    }
}

enum ObjectTask {
    Internal(ObjectRequest),
    GetMethod(ObjectHandle, PropertyKey),
    ToPropertyDescriptor(Value),
    PropertyKeyList(Value),
}

enum ObjectReply {
    Prototype(Option<ObjectHandle>),
    Boolean(bool),
    OwnProperty(Option<Property>),
    Value(Value),
    Keys(Vec<PropertyKey>),
    Descriptor(PropertyDescriptor),
    Method(Option<ObjectHandle>),
    Length(u64),
}

enum ObjectStep {
    Run(ObjectTask),
    Reply(ObjectReply),
    Call(ObjectHandle, Value, Vec<Value>),
    CoerceLength(Value),
    SetTypedArrayElement(ObjectHandle, f64, Value),
    NormalizeArrayLength(ObjectHandle, PropertyKey, PropertyDescriptor),
}

enum ObjectContinuation {
    ProxyMethod(ObjectRequest, ProxySnapshot),
    ProxyTrap(ObjectRequest, ProxySnapshot),
    Invariant(ProxyInvariant),
    MethodValue,
    DescriptorHas(DescriptorWalk),
    DescriptorGet(DescriptorWalk),
    KeyListLengthGet(KeyListWalk),
    KeyListLengthCoerced(KeyListWalk),
    KeyListElement(KeyListWalk),
    SetReceiverOwn(ObjectHandle, PropertyKey, Value),
    SetterReturned,
    OrdinaryArrayLength(ObjectHandle, PropertyKey),
}

enum ProxyInvariant {
    GetPrototypeExtensible(ObjectHandle, Option<ObjectHandle>),
    GetPrototypeFixed(Option<ObjectHandle>),
    SetPrototypeExtensible(ObjectHandle, Option<ObjectHandle>),
    SetPrototypeFixed(Option<ObjectHandle>),
    IsExtensibleResult(bool),
    PreventExtensionsResult,
    GetOwnTarget(ObjectHandle, Option<ObjectHandle>),
    GetOwnAbsentExtensible,
    GetOwnPresentExtensible(Option<Property>, ObjectHandle),
    GetOwnDescriptor(Option<Property>, bool),
    DefineTarget(ObjectHandle, PropertyDescriptor),
    DefineExtensible(PropertyDescriptor, Option<Property>),
    HasTarget(ObjectHandle),
    HasExtensible,
    GetTarget(Value),
    SetTarget(Value),
    DeleteTarget(ObjectHandle),
    DeleteExtensible,
    KeysConverted(ObjectHandle),
    KeysExtensible(ObjectHandle, Vec<PropertyKey>),
    KeysTargetList(ObjectHandle, Vec<PropertyKey>, bool),
    KeysTargetDescriptor(KeyInvariantWalk),
}

struct KeyInvariantWalk {
    target: ObjectHandle,
    trap_keys: Vec<PropertyKey>,
    target_keys: Vec<PropertyKey>,
    next: usize,
    extensible: bool,
    nonconfigurable: Vec<PropertyKey>,
    configurable: Vec<PropertyKey>,
}

#[derive(Clone, Copy)]
enum DescriptorField {
    Enumerable,
    Configurable,
    Value,
    Writable,
    Get,
    Set,
}

impl DescriptorField {
    fn name(self) -> &'static str {
        match self {
            Self::Enumerable => "enumerable",
            Self::Configurable => "configurable",
            Self::Value => "value",
            Self::Writable => "writable",
            Self::Get => "get",
            Self::Set => "set",
        }
    }
    fn next(self) -> Option<Self> {
        match self {
            Self::Enumerable => Some(Self::Configurable),
            Self::Configurable => Some(Self::Value),
            Self::Value => Some(Self::Writable),
            Self::Writable => Some(Self::Get),
            Self::Get => Some(Self::Set),
            Self::Set => None,
        }
    }
}

struct DescriptorWalk {
    object: ObjectHandle,
    field: DescriptorField,
    enumerable: Option<bool>,
    configurable: Option<bool>,
    value: Option<Value>,
    writable: Option<bool>,
    get: Option<Option<ObjectHandle>>,
    set: Option<Option<ObjectHandle>>,
}

struct KeyListWalk {
    object: ObjectHandle,
    length: u64,
    next: u64,
    keys: Vec<PropertyKey>,
}

fn type_error(span: Span, message: &'static str) -> Error {
    Realm::exception(ExceptionKind::TypeError, span, message)
}

fn allocation_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "object internal-method allocation failed".into(),
    }
}

fn internal(object: ObjectHandle, operation: ObjectOperation) -> ObjectTask {
    ObjectTask::Internal(ObjectRequest { object, operation })
}

fn key_value(key: &PropertyKey) -> Value {
    match key {
        PropertyKey::String(key) => Value::String(key.clone()),
        PropertyKey::Symbol(key) => Value::Symbol(key.clone()),
    }
}

pub(super) fn owned_key(key: PropertyKeyRef<'_>) -> PropertyKey {
    match key {
        PropertyKeyRef::String(key) => PropertyKey::String(key.clone()),
        PropertyKeyRef::Symbol(key) => PropertyKey::Symbol(key.clone()),
    }
}

fn boolean(reply: ObjectReply) -> bool {
    let ObjectReply::Boolean(value) = reply else {
        unreachable!("Boolean object reply")
    };
    value
}
fn prototype(reply: ObjectReply) -> Option<ObjectHandle> {
    let ObjectReply::Prototype(value) = reply else {
        unreachable!("prototype object reply")
    };
    value
}
fn own(reply: ObjectReply) -> Option<Property> {
    let ObjectReply::OwnProperty(value) = reply else {
        unreachable!("descriptor object reply")
    };
    value
}
fn value(reply: ObjectReply) -> Value {
    let ObjectReply::Value(value) = reply else {
        unreachable!("value object reply")
    };
    value
}
fn keys(reply: ObjectReply) -> Vec<PropertyKey> {
    let ObjectReply::Keys(value) = reply else {
        unreachable!("keys object reply")
    };
    value
}

impl Realm {
    fn object_task(&mut self, task: ObjectTask, span: Span) -> Result<ObjectReply, Error> {
        let mut pending = Vec::new();
        let mut step = ObjectStep::Run(task);
        loop {
            step = match step {
                ObjectStep::Run(task) => self.object_task_step(task, &mut pending, span)?,
                ObjectStep::Reply(reply) => match pending.pop() {
                    Some(frame) => self.object_resume(frame, reply, &mut pending, span)?,
                    None => return Ok(reply),
                },
                ObjectStep::Call(function, this, arguments) => {
                    ObjectStep::Reply(ObjectReply::Value(self.call(
                        Value::Object(function),
                        this,
                        arguments,
                        span,
                    )?))
                }
                ObjectStep::CoerceLength(value) => {
                    ObjectStep::Reply(ObjectReply::Length(self.length_from_value(value, span)?))
                }
                ObjectStep::SetTypedArrayElement(object, index, value) => {
                    self.set_typed_array_element(&object, index, value, span)?;
                    ObjectStep::Reply(ObjectReply::Boolean(true))
                }
                ObjectStep::NormalizeArrayLength(object, key, mut descriptor) => {
                    self.convert_array_length(&object, &key, &mut descriptor, span)?;
                    ObjectStep::Reply(ObjectReply::Descriptor(descriptor))
                }
            };
        }
    }

    fn object_then(
        &mut self,
        pending: &mut Vec<ObjectContinuation>,
        frame: ObjectContinuation,
        task: ObjectTask,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        self.object_push(pending, frame, span)?;
        Ok(ObjectStep::Run(task))
    }

    fn object_push(
        &mut self,
        pending: &mut Vec<ObjectContinuation>,
        frame: ObjectContinuation,
        span: Span,
    ) -> Result<(), Error> {
        self.object_work(span, |_, budget| budget.charge(1))?;
        pending.try_reserve(1).map_err(|_| allocation_error(span))?;
        pending.push(frame);
        Ok(())
    }

    fn object_arguments<const N: usize>(
        &mut self,
        arguments: [Value; N],
        span: Span,
    ) -> Result<Vec<Value>, Error> {
        let mut result = Vec::new();
        result
            .try_reserve_exact(N)
            .map_err(|_| allocation_error(span))?;
        result.extend(arguments);
        Ok(result)
    }

    fn object_copy_value(&mut self, value: &Value, span: Span) -> Result<Value, Error> {
        self.object_work(span, |_, budget| {
            budget.value(value)?;
            Ok(value.clone())
        })
    }

    pub(crate) fn get_prototype_of(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Option<ObjectHandle>, Error> {
        let direct = self.object_work(span, |objects, budget| {
            if objects.proxy_state(object, budget)?.is_some() {
                return Ok(None);
            }
            Ok(Some(objects.inspect(object)?.prototype().cloned()))
        })?;
        match direct {
            Some(result) => Ok(result),
            None => self
                .object_task(
                    internal(object.clone(), ObjectOperation::GetPrototypeOf),
                    span,
                )
                .map(prototype),
        }
    }

    pub(crate) fn set_prototype_of(
        &mut self,
        object: &ObjectHandle,
        next: Option<ObjectHandle>,
        span: Span,
    ) -> Result<bool, Error> {
        let direct = self.object_work(span, |objects, budget| {
            if objects.proxy_state(object, budget)?.is_some() {
                return Ok(None);
            }
            objects
                .set_prototype(object, next.as_ref(), budget)
                .map(Some)
        })?;
        match direct {
            Some(result) => Ok(result),
            None => self
                .object_task(
                    internal(object.clone(), ObjectOperation::SetPrototypeOf(next)),
                    span,
                )
                .map(boolean),
        }
    }

    pub(crate) fn is_extensible(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<bool, Error> {
        let direct = self.object_work(span, |objects, budget| {
            if objects.proxy_state(object, budget)?.is_some() {
                return Ok(None);
            }
            Ok(Some(objects.inspect(object)?.is_extensible()))
        })?;
        match direct {
            Some(result) => Ok(result),
            None => self
                .object_task(
                    internal(object.clone(), ObjectOperation::IsExtensible),
                    span,
                )
                .map(boolean),
        }
    }

    pub(crate) fn prevent_extensions(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<bool, Error> {
        let direct = self.object_work(span, |objects, budget| {
            if objects.proxy_state(object, budget)?.is_some() {
                return Ok(None);
            }
            objects.prevent_extensions(object).map(Some)
        })?;
        match direct {
            Some(result) => Ok(result),
            None => self
                .object_task(
                    internal(object.clone(), ObjectOperation::PreventExtensions),
                    span,
                )
                .map(boolean),
        }
    }

    pub(crate) fn is_array(&mut self, value: &Value, span: Span) -> Result<bool, Error> {
        let Value::Object(object) = value else {
            return Ok(false);
        };
        let mut object = object.clone();
        loop {
            let (proxy, array) = self.object_work(span, |objects, budget| {
                Ok((
                    objects.proxy_state(&object, budget)?,
                    objects.inspect(&object)?.is_array(),
                ))
            })?;
            match proxy {
                None => return Ok(array),
                Some(ProxyState::Revoked) => {
                    return Err(type_error(span, "proxy has been revoked"));
                }
                Some(ProxyState::Live(snapshot)) => object = snapshot.target,
            }
        }
    }

    pub(crate) fn proxy_get_method(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKey,
        span: Span,
    ) -> Result<Option<ObjectHandle>, Error> {
        let ObjectReply::Method(method) =
            self.object_task(ObjectTask::GetMethod(object.clone(), key), span)?
        else {
            unreachable!("GetMethod reply")
        };
        Ok(method)
    }

    pub(super) fn proxy_get_property(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKey,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        self.object_task(
            internal(object.clone(), ObjectOperation::Get(key, receiver)),
            span,
        )
        .map(value)
    }
    pub(super) fn proxy_has_property(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKey,
        span: Span,
    ) -> Result<bool, Error> {
        self.object_task(
            internal(object.clone(), ObjectOperation::HasProperty(key)),
            span,
        )
        .map(boolean)
    }
    pub(super) fn proxy_set_property(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKey,
        assigned: Value,
        receiver: Value,
        span: Span,
    ) -> Result<bool, Error> {
        self.object_task(
            internal(
                object.clone(),
                ObjectOperation::Set(key, assigned, receiver),
            ),
            span,
        )
        .map(boolean)
    }
    pub(super) fn proxy_delete_property(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKey,
        span: Span,
    ) -> Result<bool, Error> {
        self.object_task(internal(object.clone(), ObjectOperation::Delete(key)), span)
            .map(boolean)
    }
    pub(super) fn proxy_own_property_keys(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Vec<PropertyKey>, Error> {
        self.object_task(
            internal(object.clone(), ObjectOperation::OwnPropertyKeys),
            span,
        )
        .map(keys)
    }
    pub(super) fn proxy_own_property_descriptor(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKey,
        span: Span,
    ) -> Result<Option<Property>, Error> {
        self.object_task(
            internal(object.clone(), ObjectOperation::GetOwnProperty(key)),
            span,
        )
        .map(own)
    }
    pub(crate) fn proxy_define_property(
        &mut self,
        object: &ObjectHandle,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        span: Span,
    ) -> Result<bool, Error> {
        self.object_task(
            internal(
                object.clone(),
                ObjectOperation::DefineOwnProperty(key, descriptor),
            ),
            span,
        )
        .map(boolean)
    }

    fn object_task_step(
        &mut self,
        task: ObjectTask,
        pending: &mut Vec<ObjectContinuation>,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        match task {
            ObjectTask::GetMethod(object, key) => self.object_then(
                pending,
                ObjectContinuation::MethodValue,
                internal(
                    object.clone(),
                    ObjectOperation::Get(key, Value::Object(object)),
                ),
                span,
            ),
            ObjectTask::ToPropertyDescriptor(attributes) => {
                let Value::Object(object) = attributes else {
                    return Err(type_error(span, "property attributes must be an object"));
                };
                self.object_descriptor_field(
                    DescriptorWalk {
                        object,
                        field: DescriptorField::Enumerable,
                        enumerable: None,
                        configurable: None,
                        value: None,
                        writable: None,
                        get: None,
                        set: None,
                    },
                    pending,
                    span,
                )
            }
            ObjectTask::PropertyKeyList(array_like) => {
                let Value::Object(object) = array_like else {
                    return Err(type_error(span, "ownKeys trap must return an object"));
                };
                let task = internal(
                    object.clone(),
                    ObjectOperation::Get(
                        JsString::from("length").into(),
                        Value::Object(object.clone()),
                    ),
                );
                self.object_then(
                    pending,
                    ObjectContinuation::KeyListLengthGet(KeyListWalk {
                        object,
                        length: 0,
                        next: 0,
                        keys: Vec::new(),
                    }),
                    task,
                    span,
                )
            }
            ObjectTask::Internal(request) => {
                let state = self.object_work(span, |objects, budget| {
                    objects.proxy_state(&request.object, budget)
                })?;
                match state {
                    Some(ProxyState::Revoked) => Err(type_error(span, "proxy has been revoked")),
                    Some(ProxyState::Live(snapshot)) => {
                        let task = ObjectTask::GetMethod(
                            snapshot.handler.clone(),
                            JsString::from(request.operation.trap_name()).into(),
                        );
                        self.object_then(
                            pending,
                            ObjectContinuation::ProxyMethod(request, snapshot),
                            task,
                            span,
                        )
                    }
                    None => self.object_ordinary_step(request, pending, span),
                }
            }
        }
    }

    fn object_ordinary_step(
        &mut self,
        request: ObjectRequest,
        pending: &mut Vec<ObjectContinuation>,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        let object = request.object;
        let reply = match request.operation {
            ObjectOperation::GetPrototypeOf => {
                ObjectReply::Prototype(self.object_work(span, |objects, _| {
                    Ok(objects.inspect(&object)?.prototype().cloned())
                })?)
            }
            ObjectOperation::SetPrototypeOf(next) => {
                ObjectReply::Boolean(self.object_work(span, |objects, budget| {
                    objects.set_prototype(&object, next.as_ref(), budget)
                })?)
            }
            ObjectOperation::IsExtensible => {
                ObjectReply::Boolean(self.object_work(span, |objects, _| {
                    Ok(objects.inspect(&object)?.is_extensible())
                })?)
            }
            ObjectOperation::PreventExtensions => ObjectReply::Boolean(
                self.object_work(span, |objects, _| objects.prevent_extensions(&object))?,
            ),
            ObjectOperation::GetOwnProperty(key) => {
                if let Some(property) =
                    self.namespace_export_property(&object, (&key).into(), span)?
                {
                    return Ok(ObjectStep::Reply(ObjectReply::OwnProperty(Some(property))));
                };
                let property = self.object_work(span, |objects, budget| {
                    objects.get_own(&object, &key, budget)
                })?;
                if property.is_none() && self.missing_intrinsic_property(&object, &key) {
                    return Err(Self::unsupported(
                        span,
                        "intrinsic property descriptor is not implemented",
                    ));
                }
                ObjectReply::OwnProperty(property)
            }
            ObjectOperation::DefineOwnProperty(key, descriptor) => {
                if let Some(allowed) =
                    self.namespace_define_property(&object, (&key).into(), &descriptor, span)?
                {
                    return Ok(ObjectStep::Reply(ObjectReply::Boolean(allowed)));
                };
                self.check_missing_intrinsic_mutation(&object, &key, span)?;
                let action = self.object_work(span, |objects, budget| {
                    objects.typed_array_define_action(&object, &key, &descriptor, budget)
                })?;
                match action {
                    object::TypedArrayDefineAction::Done(result) => {
                        return Ok(ObjectStep::Reply(ObjectReply::Boolean(result)));
                    }
                    object::TypedArrayDefineAction::Element { index } => {
                        let DescriptorKind::Data {
                            value: Some(value), ..
                        } = descriptor.kind
                        else {
                            unreachable!("typed element value")
                        };
                        return Ok(ObjectStep::SetTypedArrayElement(object, index, value));
                    }
                    object::TypedArrayDefineAction::Ordinary => {}
                }
                self.object_push(
                    pending,
                    ObjectContinuation::OrdinaryArrayLength(object.clone(), key.clone()),
                    span,
                )?;
                return Ok(ObjectStep::NormalizeArrayLength(object, key, descriptor));
            }
            ObjectOperation::HasProperty(key) => {
                let (has, terminal) = self.object_work(span, |objects, budget| {
                    let has = objects.has_own(&object, &key, budget)?;
                    let terminal = !has
                        && objects
                            .typed_array_numeric_index(&object, &key, budget)?
                            .is_some();
                    Ok((has, terminal))
                })?;
                if has || self.missing_intrinsic_property(&object, &key) {
                    ObjectReply::Boolean(true)
                } else if terminal {
                    ObjectReply::Boolean(false)
                } else if let Some(parent) = self.object_work(span, |objects, _| {
                    Ok(objects.inspect(&object)?.prototype().cloned())
                })? {
                    return Ok(ObjectStep::Run(internal(
                        parent,
                        ObjectOperation::HasProperty(key),
                    )));
                } else {
                    ObjectReply::Boolean(false)
                }
            }
            ObjectOperation::Get(key, receiver) => {
                if let Some(Property::Data(property)) =
                    self.namespace_export_property(&object, (&key).into(), span)?
                {
                    return Ok(ObjectStep::Reply(ObjectReply::Value(property.value)));
                };
                let (property, terminal) = self.object_work(span, |objects, budget| {
                    let property = objects.get_own(&object, &key, budget)?;
                    let terminal = property.is_none()
                        && objects
                            .typed_array_numeric_index(&object, &key, budget)?
                            .is_some();
                    Ok((property, terminal))
                })?;
                match property {
                    Some(Property::Data(data)) => ObjectReply::Value(data.value),
                    Some(Property::Accessor(data)) => match data.get {
                        Some(getter) => return Ok(ObjectStep::Call(getter, receiver, Vec::new())),
                        None => ObjectReply::Value(Value::Undefined),
                    },
                    None if terminal => ObjectReply::Value(Value::Undefined),
                    None => {
                        if self.missing_intrinsic_property(&object, &key) {
                            return Err(Self::unsupported(
                                span,
                                "intrinsic prototype property is not implemented",
                            ));
                        }
                        match self.object_work(span, |objects, _| {
                            Ok(objects.inspect(&object)?.prototype().cloned())
                        })? {
                            Some(parent) => {
                                return Ok(ObjectStep::Run(internal(
                                    parent,
                                    ObjectOperation::Get(key, receiver),
                                )));
                            }
                            None => ObjectReply::Value(Value::Undefined),
                        }
                    }
                }
            }
            ObjectOperation::Set(key, assigned, receiver) => {
                if self
                    .objects
                    .inspect(&object)
                    .map_err(Error::InvalidObject)?
                    .is_module_namespace()
                {
                    return Ok(ObjectStep::Reply(ObjectReply::Boolean(false)));
                };
                let action = self.object_work(span, |objects, budget| {
                    objects.typed_array_set_action(
                        &object,
                        &key,
                        if let Value::Object(receiver) = &receiver {
                            Some(receiver)
                        } else {
                            None
                        },
                        budget,
                    )
                })?;
                match action {
                    object::TypedArraySetAction::Done(result) => {
                        return Ok(ObjectStep::Reply(ObjectReply::Boolean(result)));
                    }
                    object::TypedArraySetAction::Element { index } => {
                        return Ok(ObjectStep::SetTypedArrayElement(object, index, assigned));
                    }
                    object::TypedArraySetAction::Ordinary => {}
                }
                let property = self.object_work(span, |objects, budget| {
                    objects.get_own(&object, &key, budget)
                })?;
                if property.is_none() {
                    if self.missing_intrinsic_property(&object, &key) {
                        return Err(Self::unsupported(
                            span,
                            "intrinsic property descriptor is not implemented",
                        ));
                    }
                    if let Some(parent) = self.object_work(span, |objects, _| {
                        Ok(objects.inspect(&object)?.prototype().cloned())
                    })? {
                        return Ok(ObjectStep::Run(internal(
                            parent,
                            ObjectOperation::Set(key, assigned, receiver),
                        )));
                    }
                }
                match property {
                    Some(Property::Accessor(data)) => {
                        let Some(setter) = data.set else {
                            return Ok(ObjectStep::Reply(ObjectReply::Boolean(false)));
                        };
                        let args = self.object_arguments([assigned], span)?;
                        self.object_push(pending, ObjectContinuation::SetterReturned, span)?;
                        return Ok(ObjectStep::Call(setter, receiver, args));
                    }
                    Some(Property::Data(data)) if !data.writable => {
                        return Ok(ObjectStep::Reply(ObjectReply::Boolean(false)));
                    }
                    _ => {}
                }
                let Value::Object(receiver) = receiver else {
                    return Ok(ObjectStep::Reply(ObjectReply::Boolean(false)));
                };
                return self.object_then(
                    pending,
                    ObjectContinuation::SetReceiverOwn(receiver.clone(), key.clone(), assigned),
                    internal(receiver, ObjectOperation::GetOwnProperty(key)),
                    span,
                );
            }
            ObjectOperation::Delete(key) => {
                self.check_global_property_operation(&object, &key, span)?;
                self.check_missing_intrinsic_mutation(&object, &key, span)?;
                ObjectReply::Boolean(self.object_work(span, |objects, budget| {
                    objects.delete(&object, &key, budget)
                })?)
            }
            ObjectOperation::OwnPropertyKeys => {
                if object
                    == self
                        .intrinsics
                        .as_ref()
                        .expect("initialized")
                        .array
                        .constructor
                {
                    return Err(Self::unsupported(
                        span,
                        "own keys of this incomplete intrinsic are not implemented",
                    ));
                }
                ObjectReply::Keys(
                    self.object_work(span, |objects, budget| objects.own_keys(&object, budget))?,
                )
            }
        };
        Ok(ObjectStep::Reply(reply))
    }

    fn object_resume(
        &mut self,
        frame: ObjectContinuation,
        reply: ObjectReply,
        pending: &mut Vec<ObjectContinuation>,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        match frame {
            ObjectContinuation::MethodValue => {
                let method = value(reply);
                let method = match method {
                    Value::Undefined | Value::Null => None,
                    method if self.is_callable(&method, span)? => {
                        let Value::Object(method) = method else {
                            unreachable!("callable object")
                        };
                        Some(method)
                    }
                    _ => return Err(type_error(span, "proxy trap must be callable or undefined")),
                };
                Ok(ObjectStep::Reply(ObjectReply::Method(method)))
            }
            ObjectContinuation::ProxyMethod(request, snapshot) => {
                let ObjectReply::Method(method) = reply else {
                    unreachable!("trap method reply")
                };
                let Some(method) = method else {
                    return Ok(ObjectStep::Run(internal(
                        snapshot.target,
                        request.operation,
                    )));
                };
                let target = Value::Object(snapshot.target.clone());
                let args = match &request.operation {
                    ObjectOperation::GetPrototypeOf
                    | ObjectOperation::IsExtensible
                    | ObjectOperation::PreventExtensions
                    | ObjectOperation::OwnPropertyKeys => self.object_arguments([target], span)?,
                    ObjectOperation::SetPrototypeOf(proto) => self.object_arguments(
                        [target, proto.clone().map_or(Value::Null, Value::Object)],
                        span,
                    )?,
                    ObjectOperation::GetOwnProperty(key)
                    | ObjectOperation::HasProperty(key)
                    | ObjectOperation::Delete(key) => {
                        self.object_arguments([target, key_value(key)], span)?
                    }
                    ObjectOperation::Get(key, receiver) => {
                        let receiver = self.object_copy_value(receiver, span)?;
                        self.object_arguments([target, key_value(key), receiver], span)?
                    }
                    ObjectOperation::Set(key, assigned, receiver) => {
                        let assigned = self.object_copy_value(assigned, span)?;
                        let receiver = self.object_copy_value(receiver, span)?;
                        self.object_arguments([target, key_value(key), assigned, receiver], span)?
                    }
                    ObjectOperation::DefineOwnProperty(key, descriptor) => {
                        let descriptor = self.partial_descriptor_object(descriptor, span)?;
                        self.object_arguments([target, key_value(key), descriptor], span)?
                    }
                };
                let handler = Value::Object(snapshot.handler.clone());
                self.object_push(
                    pending,
                    ObjectContinuation::ProxyTrap(request, snapshot),
                    span,
                )?;
                Ok(ObjectStep::Call(method, handler, args))
            }
            ObjectContinuation::ProxyTrap(request, snapshot) => self.object_trap_result(
                request.operation,
                snapshot.target,
                value(reply),
                pending,
                span,
            ),
            ObjectContinuation::Invariant(invariant) => {
                self.object_invariant(invariant, reply, pending, span)
            }
            ObjectContinuation::SetterReturned => {
                let _ = value(reply);
                Ok(ObjectStep::Reply(ObjectReply::Boolean(true)))
            }
            ObjectContinuation::SetReceiverOwn(receiver, key, assigned) => {
                let descriptor = match own(reply) {
                    Some(Property::Data(data)) if data.writable => DataDescriptor {
                        value: Some(assigned),
                        ..Default::default()
                    },
                    Some(_) => return Ok(ObjectStep::Reply(ObjectReply::Boolean(false))),
                    None => DataDescriptor {
                        value: Some(assigned),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                };
                Ok(ObjectStep::Run(internal(
                    receiver,
                    ObjectOperation::DefineOwnProperty(key, descriptor.into()),
                )))
            }
            ObjectContinuation::OrdinaryArrayLength(object, key) => {
                let ObjectReply::Descriptor(descriptor) = reply else {
                    unreachable!("Array length descriptor reply")
                };
                self.object_work(span, |objects, budget| {
                    objects.define(&object, key, descriptor, budget)
                })
                .map(|result| ObjectStep::Reply(ObjectReply::Boolean(result)))
            }
            ObjectContinuation::DescriptorHas(walk) => {
                if boolean(reply) {
                    let task = internal(
                        walk.object.clone(),
                        ObjectOperation::Get(
                            JsString::from(walk.field.name()).into(),
                            Value::Object(walk.object.clone()),
                        ),
                    );
                    self.object_then(pending, ObjectContinuation::DescriptorGet(walk), task, span)
                } else {
                    self.object_descriptor_next(walk, pending, span)
                }
            }
            ObjectContinuation::DescriptorGet(mut walk) => {
                let result = value(reply);
                match walk.field {
                    DescriptorField::Enumerable => walk.enumerable = Some(result.to_boolean()),
                    DescriptorField::Configurable => walk.configurable = Some(result.to_boolean()),
                    DescriptorField::Value => walk.value = Some(result),
                    DescriptorField::Writable => walk.writable = Some(result.to_boolean()),
                    DescriptorField::Get | DescriptorField::Set => {
                        let accessor = match result {
                            Value::Undefined => None,
                            result if self.is_callable(&result, span)? => {
                                let Value::Object(accessor) = result else {
                                    unreachable!("callable accessor")
                                };
                                Some(accessor)
                            }
                            _ => {
                                return Err(type_error(
                                    span,
                                    "property accessor must be callable or undefined",
                                ));
                            }
                        };
                        if matches!(walk.field, DescriptorField::Get) {
                            walk.get = Some(accessor);
                        } else {
                            walk.set = Some(accessor);
                        }
                    }
                }
                self.object_descriptor_next(walk, pending, span)
            }
            ObjectContinuation::KeyListLengthGet(walk) => {
                self.object_push(
                    pending,
                    ObjectContinuation::KeyListLengthCoerced(walk),
                    span,
                )?;
                Ok(ObjectStep::CoerceLength(value(reply)))
            }
            ObjectContinuation::KeyListLengthCoerced(mut walk) => {
                let ObjectReply::Length(length) = reply else {
                    unreachable!("array-like length reply")
                };
                walk.length = length;
                self.object_key_list_next(walk, pending, span)
            }
            ObjectContinuation::KeyListElement(mut walk) => {
                let key = match value(reply) {
                    Value::String(key) => PropertyKey::String(key),
                    Value::Symbol(key) => PropertyKey::Symbol(key),
                    _ => {
                        return Err(type_error(
                            span,
                            "ownKeys trap entries must be strings or symbols",
                        ));
                    }
                };
                walk.keys
                    .try_reserve(1)
                    .map_err(|_| allocation_error(span))?;
                walk.keys.push(key);
                walk.next += 1; // next < ToLength <= 2^53-1
                self.object_key_list_next(walk, pending, span)
            }
        }
    }

    fn object_descriptor_field(
        &mut self,
        walk: DescriptorWalk,
        pending: &mut Vec<ObjectContinuation>,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        let task = internal(
            walk.object.clone(),
            ObjectOperation::HasProperty(JsString::from(walk.field.name()).into()),
        );
        self.object_then(pending, ObjectContinuation::DescriptorHas(walk), task, span)
    }

    fn object_descriptor_next(
        &mut self,
        mut walk: DescriptorWalk,
        pending: &mut Vec<ObjectContinuation>,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        if let Some(next) = walk.field.next() {
            walk.field = next;
            return self.object_descriptor_field(walk, pending, span);
        }
        let kind = if walk.get.is_some() || walk.set.is_some() {
            if walk.value.is_some() || walk.writable.is_some() {
                return Err(type_error(
                    span,
                    "property descriptor mixes data and accessors",
                ));
            }
            DescriptorKind::Accessor {
                get: walk.get,
                set: walk.set,
            }
        } else {
            DescriptorKind::Data {
                value: walk.value,
                writable: walk.writable,
            }
        };
        Ok(ObjectStep::Reply(ObjectReply::Descriptor(
            PropertyDescriptor {
                kind,
                enumerable: walk.enumerable,
                configurable: walk.configurable,
            },
        )))
    }

    fn object_key_list_next(
        &mut self,
        walk: KeyListWalk,
        pending: &mut Vec<ObjectContinuation>,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        if walk.next == walk.length {
            return Ok(ObjectStep::Reply(ObjectReply::Keys(walk.keys)));
        }
        self.object_work(span, |_, budget| budget.charge(1))?;
        let key = JsString::from(walk.next.to_string().as_str());
        let task = internal(
            walk.object.clone(),
            ObjectOperation::Get(key.into(), Value::Object(walk.object.clone())),
        );
        self.object_then(
            pending,
            ObjectContinuation::KeyListElement(walk),
            task,
            span,
        )
    }

    fn object_trap_result(
        &mut self,
        operation: ObjectOperation,
        target: ObjectHandle,
        result: Value,
        pending: &mut Vec<ObjectContinuation>,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        let (frame, task) = match operation {
            ObjectOperation::GetPrototypeOf => {
                let proto = match result {
                    Value::Null => None,
                    Value::Object(object) => Some(object),
                    _ => {
                        return Err(type_error(
                            span,
                            "getPrototypeOf trap must return an object or null",
                        ));
                    }
                };
                (
                    ProxyInvariant::GetPrototypeExtensible(target.clone(), proto),
                    internal(target, ObjectOperation::IsExtensible),
                )
            }
            ObjectOperation::SetPrototypeOf(proto) => {
                if !result.to_boolean() {
                    return Ok(ObjectStep::Reply(ObjectReply::Boolean(false)));
                }
                (
                    ProxyInvariant::SetPrototypeExtensible(target.clone(), proto),
                    internal(target, ObjectOperation::IsExtensible),
                )
            }
            ObjectOperation::IsExtensible => (
                ProxyInvariant::IsExtensibleResult(result.to_boolean()),
                internal(target, ObjectOperation::IsExtensible),
            ),
            ObjectOperation::PreventExtensions => {
                if !result.to_boolean() {
                    return Ok(ObjectStep::Reply(ObjectReply::Boolean(false)));
                }
                (
                    ProxyInvariant::PreventExtensionsResult,
                    internal(target, ObjectOperation::IsExtensible),
                )
            }
            ObjectOperation::GetOwnProperty(key) => {
                let descriptor = match result {
                    Value::Undefined => None,
                    Value::Object(object) => Some(object),
                    _ => {
                        return Err(type_error(
                            span,
                            "getOwnPropertyDescriptor trap must return an object or undefined",
                        ));
                    }
                };
                (
                    ProxyInvariant::GetOwnTarget(target.clone(), descriptor),
                    internal(target, ObjectOperation::GetOwnProperty(key)),
                )
            }
            ObjectOperation::DefineOwnProperty(key, descriptor) => {
                if !result.to_boolean() {
                    return Ok(ObjectStep::Reply(ObjectReply::Boolean(false)));
                }
                (
                    ProxyInvariant::DefineTarget(target.clone(), descriptor),
                    internal(target, ObjectOperation::GetOwnProperty(key)),
                )
            }
            ObjectOperation::HasProperty(key) => {
                if result.to_boolean() {
                    return Ok(ObjectStep::Reply(ObjectReply::Boolean(true)));
                }
                (
                    ProxyInvariant::HasTarget(target.clone()),
                    internal(target, ObjectOperation::GetOwnProperty(key)),
                )
            }
            ObjectOperation::Get(key, _) => (
                ProxyInvariant::GetTarget(result),
                internal(target, ObjectOperation::GetOwnProperty(key)),
            ),
            ObjectOperation::Set(key, assigned, _) => {
                if !result.to_boolean() {
                    return Ok(ObjectStep::Reply(ObjectReply::Boolean(false)));
                }
                (
                    ProxyInvariant::SetTarget(assigned),
                    internal(target, ObjectOperation::GetOwnProperty(key)),
                )
            }
            ObjectOperation::Delete(key) => {
                if !result.to_boolean() {
                    return Ok(ObjectStep::Reply(ObjectReply::Boolean(false)));
                }
                (
                    ProxyInvariant::DeleteTarget(target.clone()),
                    internal(target, ObjectOperation::GetOwnProperty(key)),
                )
            }
            ObjectOperation::OwnPropertyKeys => (
                ProxyInvariant::KeysConverted(target),
                ObjectTask::PropertyKeyList(result),
            ),
        };
        self.object_then(pending, ObjectContinuation::Invariant(frame), task, span)
    }

    fn object_invariant(
        &mut self,
        invariant: ProxyInvariant,
        reply: ObjectReply,
        pending: &mut Vec<ObjectContinuation>,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        let result = match invariant {
            ProxyInvariant::GetPrototypeExtensible(target, reported) => {
                if boolean(reply) {
                    ObjectReply::Prototype(reported)
                } else {
                    return self.object_then(
                        pending,
                        ObjectContinuation::Invariant(ProxyInvariant::GetPrototypeFixed(reported)),
                        internal(target, ObjectOperation::GetPrototypeOf),
                        span,
                    );
                }
            }
            ProxyInvariant::GetPrototypeFixed(reported) => {
                if prototype(reply) != reported {
                    return Err(type_error(
                        span,
                        "getPrototypeOf trap contradicts a non-extensible target",
                    ));
                }
                ObjectReply::Prototype(reported)
            }
            ProxyInvariant::SetPrototypeExtensible(target, reported) => {
                if boolean(reply) {
                    ObjectReply::Boolean(true)
                } else {
                    return self.object_then(
                        pending,
                        ObjectContinuation::Invariant(ProxyInvariant::SetPrototypeFixed(reported)),
                        internal(target, ObjectOperation::GetPrototypeOf),
                        span,
                    );
                }
            }
            ProxyInvariant::SetPrototypeFixed(reported) => {
                if prototype(reply) != reported {
                    return Err(type_error(
                        span,
                        "setPrototypeOf trap contradicts a non-extensible target",
                    ));
                }
                ObjectReply::Boolean(true)
            }
            ProxyInvariant::IsExtensibleResult(reported) => {
                if boolean(reply) != reported {
                    return Err(type_error(span, "isExtensible trap contradicts its target"));
                }
                ObjectReply::Boolean(reported)
            }
            ProxyInvariant::PreventExtensionsResult => {
                if boolean(reply) {
                    return Err(type_error(
                        span,
                        "preventExtensions trap left its target extensible",
                    ));
                }
                ObjectReply::Boolean(true)
            }
            ProxyInvariant::GetOwnTarget(target, reported) => {
                let current = own(reply);
                match reported {
                    None => {
                        let Some(current) = current else {
                            return Ok(ObjectStep::Reply(ObjectReply::OwnProperty(None)));
                        };
                        if !current.configurable() {
                            return Err(type_error(
                                span,
                                "descriptor trap hid a non-configurable target property",
                            ));
                        }
                        return self.object_then(
                            pending,
                            ObjectContinuation::Invariant(ProxyInvariant::GetOwnAbsentExtensible),
                            internal(target, ObjectOperation::IsExtensible),
                            span,
                        );
                    }
                    Some(reported) => {
                        return self.object_then(
                            pending,
                            ObjectContinuation::Invariant(ProxyInvariant::GetOwnPresentExtensible(
                                current, reported,
                            )),
                            internal(target, ObjectOperation::IsExtensible),
                            span,
                        );
                    }
                }
            }
            ProxyInvariant::GetOwnAbsentExtensible => {
                if !boolean(reply) {
                    return Err(type_error(
                        span,
                        "descriptor trap hid a property of a non-extensible target",
                    ));
                }
                ObjectReply::OwnProperty(None)
            }
            ProxyInvariant::GetOwnPresentExtensible(current, reported) => {
                let extensible = boolean(reply);
                return self.object_then(
                    pending,
                    ObjectContinuation::Invariant(ProxyInvariant::GetOwnDescriptor(
                        current, extensible,
                    )),
                    ObjectTask::ToPropertyDescriptor(Value::Object(reported)),
                    span,
                );
            }
            ProxyInvariant::GetOwnDescriptor(current, extensible) => {
                let ObjectReply::Descriptor(mut descriptor) = reply else {
                    unreachable!("trap descriptor reply")
                };
                // Complete the owned record before compatibility checks, retaining
                // captured target state across all descriptor conversion hooks.
                descriptor.enumerable.get_or_insert(false);
                descriptor.configurable.get_or_insert(false);
                match &mut descriptor.kind {
                    DescriptorKind::Generic => {
                        descriptor.kind = DescriptorKind::Data {
                            value: Some(Value::Undefined),
                            writable: Some(false),
                        }
                    }
                    DescriptorKind::Data { value, writable } => {
                        value.get_or_insert(Value::Undefined);
                        writable.get_or_insert(false);
                    }
                    DescriptorKind::Accessor { get, set } => {
                        get.get_or_insert(None);
                        set.get_or_insert(None);
                    }
                }
                if !self.object_descriptor_compatible(
                    extensible,
                    &descriptor,
                    current.as_ref(),
                    span,
                )? {
                    return Err(type_error(
                        span,
                        "descriptor trap returned an incompatible property",
                    ));
                }
                if descriptor.configurable == Some(false) {
                    let Some(current) = &current else {
                        return Err(type_error(
                            span,
                            "descriptor trap invented a non-configurable property",
                        ));
                    };
                    if current.configurable() {
                        return Err(type_error(
                            span,
                            "descriptor trap froze a configurable target property",
                        ));
                    }
                    if matches!((&descriptor.kind, current),
                        (DescriptorKind::Data { writable: Some(false), .. }, Property::Data(current)) if current.writable)
                    {
                        return Err(type_error(
                            span,
                            "descriptor trap made a writable target property non-writable",
                        ));
                    }
                }
                ObjectReply::OwnProperty(Some(object::complete_property_descriptor(descriptor)))
            }
            ProxyInvariant::DefineTarget(target, descriptor) => {
                let current = own(reply);
                return self.object_then(
                    pending,
                    ObjectContinuation::Invariant(ProxyInvariant::DefineExtensible(
                        descriptor, current,
                    )),
                    internal(target, ObjectOperation::IsExtensible),
                    span,
                );
            }
            ProxyInvariant::DefineExtensible(descriptor, current) => {
                let extensible = boolean(reply);
                if let Some(current) = current.as_ref() {
                    if !self.object_descriptor_compatible(
                        extensible,
                        &descriptor,
                        Some(current),
                        span,
                    )? {
                        return Err(type_error(
                            span,
                            "defineProperty trap accepted an incompatible property",
                        ));
                    }
                    if descriptor.configurable == Some(false) && current.configurable() {
                        return Err(type_error(
                            span,
                            "defineProperty trap froze a configurable target property",
                        ));
                    }
                    if !current.configurable()
                        && matches!((&descriptor.kind, current),
                        (DescriptorKind::Data { writable: Some(false), .. }, Property::Data(current)) if current.writable)
                    {
                        return Err(type_error(
                            span,
                            "defineProperty trap made a writable target property non-writable",
                        ));
                    }
                } else if !extensible || descriptor.configurable == Some(false) {
                    return Err(type_error(
                        span,
                        "defineProperty trap invented a forbidden target property",
                    ));
                }
                ObjectReply::Boolean(true)
            }
            ProxyInvariant::HasTarget(target) => {
                if let Some(current) = own(reply) {
                    if !current.configurable() {
                        return Err(type_error(span, "has trap hid a non-configurable property"));
                    }
                    return self.object_then(
                        pending,
                        ObjectContinuation::Invariant(ProxyInvariant::HasExtensible),
                        internal(target, ObjectOperation::IsExtensible),
                        span,
                    );
                }
                ObjectReply::Boolean(false)
            }
            ProxyInvariant::HasExtensible => {
                if !boolean(reply) {
                    return Err(type_error(
                        span,
                        "has trap hid a property of a non-extensible target",
                    ));
                }
                ObjectReply::Boolean(false)
            }
            ProxyInvariant::GetTarget(reported) => {
                match own(reply) {
                    Some(Property::Data(current)) if !current.configurable && !current.writable => {
                        if !self.object_same_value(&reported, &current.value, span)? {
                            return Err(type_error(
                                span,
                                "get trap changed a frozen data property's value",
                            ));
                        }
                    }
                    Some(Property::Accessor(current))
                        if !current.configurable
                            && current.get.is_none()
                            && !matches!(reported, Value::Undefined) =>
                    {
                        return Err(type_error(
                            span,
                            "get trap returned a value for an absent frozen getter",
                        ));
                    }
                    _ => {}
                }
                ObjectReply::Value(reported)
            }
            ProxyInvariant::SetTarget(assigned) => {
                match own(reply) {
                    Some(Property::Data(current)) if !current.configurable && !current.writable => {
                        if !self.object_same_value(&assigned, &current.value, span)? {
                            return Err(type_error(
                                span,
                                "set trap changed a frozen data property's value",
                            ));
                        }
                    }
                    Some(Property::Accessor(current))
                        if !current.configurable && current.set.is_none() =>
                    {
                        return Err(type_error(
                            span,
                            "set trap accepted a property with no frozen setter",
                        ));
                    }
                    _ => {}
                }
                ObjectReply::Boolean(true)
            }
            ProxyInvariant::DeleteTarget(target) => {
                if let Some(current) = own(reply) {
                    if !current.configurable() {
                        return Err(type_error(
                            span,
                            "deleteProperty trap hid a non-configurable property",
                        ));
                    }
                    return self.object_then(
                        pending,
                        ObjectContinuation::Invariant(ProxyInvariant::DeleteExtensible),
                        internal(target, ObjectOperation::IsExtensible),
                        span,
                    );
                }
                ObjectReply::Boolean(true)
            }
            ProxyInvariant::DeleteExtensible => {
                if !boolean(reply) {
                    return Err(type_error(
                        span,
                        "deleteProperty trap hid a property of a non-extensible target",
                    ));
                }
                ObjectReply::Boolean(true)
            }
            ProxyInvariant::KeysConverted(target) => {
                let trap_keys = keys(reply);
                self.object_unique_keys(&trap_keys, span)?;
                return self.object_then(
                    pending,
                    ObjectContinuation::Invariant(ProxyInvariant::KeysExtensible(
                        target.clone(),
                        trap_keys,
                    )),
                    internal(target, ObjectOperation::IsExtensible),
                    span,
                );
            }
            ProxyInvariant::KeysExtensible(target, trap_keys) => {
                let extensible = boolean(reply);
                return self.object_then(
                    pending,
                    ObjectContinuation::Invariant(ProxyInvariant::KeysTargetList(
                        target.clone(),
                        trap_keys,
                        extensible,
                    )),
                    internal(target, ObjectOperation::OwnPropertyKeys),
                    span,
                );
            }
            ProxyInvariant::KeysTargetList(target, trap_keys, extensible) => {
                return self.object_target_key_next(
                    KeyInvariantWalk {
                        target,
                        trap_keys,
                        target_keys: keys(reply),
                        next: 0,
                        extensible,
                        nonconfigurable: Vec::new(),
                        configurable: Vec::new(),
                    },
                    pending,
                    span,
                );
            }
            ProxyInvariant::KeysTargetDescriptor(mut walk) => {
                let key = walk.target_keys[walk.next].clone();
                let list = if own(reply).is_some_and(|property| !property.configurable()) {
                    &mut walk.nonconfigurable
                } else {
                    &mut walk.configurable
                };
                list.try_reserve(1).map_err(|_| allocation_error(span))?;
                list.push(key);
                walk.next += 1;
                return self.object_target_key_next(walk, pending, span);
            }
        };
        Ok(ObjectStep::Reply(result))
    }

    fn object_same_value(
        &mut self,
        left: &Value,
        right: &Value,
        span: Span,
    ) -> Result<bool, Error> {
        self.object_work(span, |_, budget| {
            budget.value(left)?;
            budget.value(right)?;
            Ok(left.same_value(right))
        })
    }

    fn object_descriptor_compatible(
        &mut self,
        extensible: bool,
        descriptor: &PropertyDescriptor,
        current: Option<&Property>,
        span: Span,
    ) -> Result<bool, Error> {
        self.object_work(span, |_, budget| {
            if let DescriptorKind::Data {
                value: Some(value), ..
            } = &descriptor.kind
            {
                budget.value(value)?;
            }
            if let Some(Property::Data(current)) = current {
                budget.value(&current.value)?;
            }
            Ok(object::is_compatible_property_descriptor(
                extensible, descriptor, current,
            ))
        })
    }

    fn object_unique_keys(&mut self, keys: &[PropertyKey], span: Span) -> Result<(), Error> {
        let unique = self.object_work(span, |_, budget| {
            // Keys use immutable string contents or immutable Symbol identity;
            // the Symbol's retirement flag does not affect Hash or Eq.
            #[allow(clippy::mutable_key_type)]
            let mut seen = std::collections::HashSet::new();
            seen.try_reserve(keys.len())
                .map_err(|_| object::Error::Heap(spite_heap::Error::Capacity))?;
            for key in keys {
                budget.charge(key.as_string().map_or(1, JsString::len))?;
                if !seen.insert(key) {
                    return Ok(false);
                }
            }
            Ok(true)
        })?;
        if !unique {
            return Err(type_error(span, "ownKeys trap returned duplicate keys"));
        }
        Ok(())
    }

    fn object_target_key_next(
        &mut self,
        walk: KeyInvariantWalk,
        pending: &mut Vec<ObjectContinuation>,
        span: Span,
    ) -> Result<ObjectStep, Error> {
        if walk.next < walk.target_keys.len() {
            let task = internal(
                walk.target.clone(),
                ObjectOperation::GetOwnProperty(walk.target_keys[walk.next].clone()),
            );
            return self.object_then(
                pending,
                ObjectContinuation::Invariant(ProxyInvariant::KeysTargetDescriptor(walk)),
                task,
                span,
            );
        }
        let valid = self.object_work(span, |_, budget| {
            // Hash/Eq use immutable string contents and Symbol allocation identity.
            #[allow(clippy::mutable_key_type)]
            let mut unchecked = std::collections::HashSet::new();
            unchecked
                .try_reserve(walk.trap_keys.len())
                .map_err(|_| object::Error::Heap(spite_heap::Error::Capacity))?;
            for key in &walk.trap_keys {
                budget.charge(key.as_string().map_or(1, JsString::len))?;
                unchecked.insert(key);
            }
            for key in &walk.nonconfigurable {
                budget.charge(key.as_string().map_or(1, JsString::len))?;
                if !unchecked.remove(key) {
                    return Ok(false);
                }
            }
            if walk.extensible {
                return Ok(true);
            }
            for key in &walk.configurable {
                budget.charge(key.as_string().map_or(1, JsString::len))?;
                if !unchecked.remove(key) {
                    return Ok(false);
                }
            }
            Ok(unchecked.is_empty())
        })?;
        if !valid {
            return Err(type_error(
                span,
                "ownKeys trap contradicted its target's keys",
            ));
        }
        Ok(ObjectStep::Reply(ObjectReply::Keys(walk.trap_keys)))
    }
}
