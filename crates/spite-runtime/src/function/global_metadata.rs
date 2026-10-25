//! Edition-17 intrinsic metadata for deferred globals and generic species getters.
//!
//! Ordinary metadata objects expose the specified reflection graph. The
//! Generic species getters, Proxy and weak-reference operations, and ordinary
//! buffer Atomics run their algorithms. Deferred constructors and methods report
//! host Unsupported and cannot create unbranded fake instances.

use super::Builtin;
use crate::{
    Error, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, PropertyKey, Span, WellKnownSymbol};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Constructor {
    Promise,
    Proxy,
    SharedArrayBuffer,
    WeakRef,
    FinalizationRegistry,
}

impl Constructor {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Promise => "Promise",
            Self::Proxy => "Proxy",
            Self::SharedArrayBuffer => "SharedArrayBuffer",
            Self::WeakRef => "WeakRef",
            Self::FinalizationRegistry => "FinalizationRegistry",
        }
    }

    pub(super) fn length(self) -> f64 {
        if matches!(self, Self::Proxy) {
            2.0
        } else {
            1.0
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Member {
    PromiseAll,
    PromiseAllSettled,
    PromiseAny,
    PromiseRace,
    PromiseReject,
    PromiseResolve,
    PromiseTry,
    PromiseWithResolvers,
    PromiseSpecies,
    PromiseCatch,
    PromiseFinally,
    PromiseThen,
    ProxyRevocable,
    SharedArrayBufferSpecies,
    SharedArrayBufferByteLength,
    SharedArrayBufferGrow,
    SharedArrayBufferGrowable,
    SharedArrayBufferMaxByteLength,
    SharedArrayBufferSlice,
    WeakRefDeref,
    FinalizationRegistryRegister,
    FinalizationRegistryUnregister,
    AtomicsAdd,
    AtomicsAnd,
    AtomicsCompareExchange,
    AtomicsExchange,
    AtomicsIsLockFree,
    AtomicsLoad,
    AtomicsOr,
    AtomicsPause,
    AtomicsStore,
    AtomicsSub,
    AtomicsWait,
    AtomicsWaitAsync,
    AtomicsNotify,
    AtomicsXor,
}

impl Member {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::PromiseAll => "all",
            Self::PromiseAllSettled => "allSettled",
            Self::PromiseAny => "any",
            Self::PromiseRace => "race",
            Self::PromiseReject => "reject",
            Self::PromiseResolve => "resolve",
            Self::PromiseTry => "try",
            Self::PromiseWithResolvers => "withResolvers",
            Self::PromiseSpecies | Self::SharedArrayBufferSpecies => "get [Symbol.species]",
            Self::PromiseCatch => "catch",
            Self::PromiseFinally => "finally",
            Self::PromiseThen => "then",
            Self::ProxyRevocable => "revocable",
            Self::SharedArrayBufferByteLength => "get byteLength",
            Self::SharedArrayBufferGrow => "grow",
            Self::SharedArrayBufferGrowable => "get growable",
            Self::SharedArrayBufferMaxByteLength => "get maxByteLength",
            Self::SharedArrayBufferSlice => "slice",
            Self::WeakRefDeref => "deref",
            Self::FinalizationRegistryRegister => "register",
            Self::FinalizationRegistryUnregister => "unregister",
            Self::AtomicsAdd => "add",
            Self::AtomicsAnd => "and",
            Self::AtomicsCompareExchange => "compareExchange",
            Self::AtomicsExchange => "exchange",
            Self::AtomicsIsLockFree => "isLockFree",
            Self::AtomicsLoad => "load",
            Self::AtomicsOr => "or",
            Self::AtomicsPause => "pause",
            Self::AtomicsStore => "store",
            Self::AtomicsSub => "sub",
            Self::AtomicsWait => "wait",
            Self::AtomicsWaitAsync => "waitAsync",
            Self::AtomicsNotify => "notify",
            Self::AtomicsXor => "xor",
        }
    }

    pub(super) fn length(self) -> f64 {
        match self {
            Self::AtomicsPause
            | Self::PromiseWithResolvers
            | Self::PromiseSpecies
            | Self::SharedArrayBufferSpecies
            | Self::SharedArrayBufferByteLength
            | Self::SharedArrayBufferGrowable
            | Self::SharedArrayBufferMaxByteLength
            | Self::WeakRefDeref => 0.0,
            Self::PromiseAll
            | Self::PromiseAllSettled
            | Self::PromiseAny
            | Self::PromiseRace
            | Self::PromiseReject
            | Self::PromiseResolve
            | Self::PromiseTry
            | Self::PromiseCatch
            | Self::PromiseFinally
            | Self::SharedArrayBufferGrow
            | Self::FinalizationRegistryUnregister
            | Self::AtomicsIsLockFree => 1.0,
            Self::PromiseThen
            | Self::ProxyRevocable
            | Self::SharedArrayBufferSlice
            | Self::FinalizationRegistryRegister
            | Self::AtomicsLoad => 2.0,
            Self::AtomicsAdd
            | Self::AtomicsAnd
            | Self::AtomicsExchange
            | Self::AtomicsOr
            | Self::AtomicsStore
            | Self::AtomicsSub
            | Self::AtomicsNotify
            | Self::AtomicsXor => 3.0,
            Self::AtomicsCompareExchange | Self::AtomicsWait | Self::AtomicsWaitAsync => 4.0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct GlobalMetadataIntrinsics {
    pub promise: ObjectHandle,
    // Already retained by objects; preserve the intrinsic fallback across mutation.
    pub promise_prototype: ObjectHandle,
    pub proxy: ObjectHandle,
    pub shared_array_buffer: ObjectHandle,
    pub weak_ref: ObjectHandle,
    // Already rooted in objects; retain this exact fallback independently of globals.
    pub weak_ref_prototype: ObjectHandle,
    pub finalization_registry: ObjectHandle,
    // This exact prototype is already rooted in objects, independently of globals.
    pub finalization_registry_prototype: ObjectHandle,
    pub atomics: ObjectHandle,
    // Four prototypes and 36 distinct method/getter objects remain rooted even
    // after scripts replace their configurable properties and global bindings.
    objects: Vec<ObjectHandle>,
}

impl GlobalMetadataIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [
            &self.promise,
            &self.proxy,
            &self.shared_array_buffer,
            &self.weak_ref,
            &self.finalization_registry,
            &self.atomics,
        ]
        .into_iter()
        .chain(&self.objects)
    }
}

impl Realm {
    pub(super) fn global_metadata_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<GlobalMetadataIntrinsics, Error> {
        let mut objects = Vec::new();
        objects.try_reserve_exact(40).map_err(|_| Error::Limit {
            span,
            message: "global intrinsic metadata capacity exceeded".into(),
        })?;
        let promise = self.new_builtin(
            function_prototype,
            Builtin::GlobalMetadataConstructor(Constructor::Promise),
            span,
        )?;
        let proxy = self.new_builtin(
            function_prototype,
            Builtin::GlobalMetadataConstructor(Constructor::Proxy),
            span,
        )?;
        let shared_array_buffer = self.new_builtin(
            function_prototype,
            Builtin::GlobalMetadataConstructor(Constructor::SharedArrayBuffer),
            span,
        )?;
        let weak_ref = self.new_builtin(
            function_prototype,
            Builtin::GlobalMetadataConstructor(Constructor::WeakRef),
            span,
        )?;
        let finalization_registry = self.new_builtin(
            function_prototype,
            Builtin::GlobalMetadataConstructor(Constructor::FinalizationRegistry),
            span,
        )?;
        let promise_prototype = self.metadata_prototype(&promise, object_prototype, span)?;
        let shared_array_buffer_prototype =
            self.metadata_prototype(&shared_array_buffer, object_prototype, span)?;
        let weak_ref_prototype = self.metadata_prototype(&weak_ref, object_prototype, span)?;
        let finalization_registry_prototype =
            self.metadata_prototype(&finalization_registry, object_prototype, span)?;
        objects.extend([
            promise_prototype.clone(),
            shared_array_buffer_prototype.clone(),
            weak_ref_prototype.clone(),
            finalization_registry_prototype.clone(),
        ]);
        let atomics =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;

        for member in [
            Member::PromiseAll,
            Member::PromiseAllSettled,
            Member::PromiseAny,
            Member::PromiseRace,
            Member::PromiseReject,
            Member::PromiseResolve,
            Member::PromiseTry,
            Member::PromiseWithResolvers,
            Member::PromiseSpecies,
        ] {
            self.install_metadata_member(&promise, function_prototype, member, &mut objects, span)?;
        }
        self.define_builtin_property(
            &promise_prototype,
            "constructor",
            Value::Object(promise.clone()),
            true,
            span,
        )?;
        for member in [
            Member::PromiseCatch,
            Member::PromiseFinally,
            Member::PromiseThen,
        ] {
            self.install_metadata_member(
                &promise_prototype,
                function_prototype,
                member,
                &mut objects,
                span,
            )?;
        }
        self.install_metadata_member(
            &proxy,
            function_prototype,
            Member::ProxyRevocable,
            &mut objects,
            span,
        )?;
        self.install_metadata_member(
            &shared_array_buffer,
            function_prototype,
            Member::SharedArrayBufferSpecies,
            &mut objects,
            span,
        )?;
        // 25.2.5 places byteLength before constructor in the property clauses.
        self.install_metadata_member(
            &shared_array_buffer_prototype,
            function_prototype,
            Member::SharedArrayBufferByteLength,
            &mut objects,
            span,
        )?;
        self.define_builtin_property(
            &shared_array_buffer_prototype,
            "constructor",
            Value::Object(shared_array_buffer.clone()),
            true,
            span,
        )?;
        for member in [
            Member::SharedArrayBufferGrow,
            Member::SharedArrayBufferGrowable,
            Member::SharedArrayBufferMaxByteLength,
            Member::SharedArrayBufferSlice,
        ] {
            self.install_metadata_member(
                &shared_array_buffer_prototype,
                function_prototype,
                member,
                &mut objects,
                span,
            )?;
        }
        self.define_builtin_property(
            &weak_ref_prototype,
            "constructor",
            Value::Object(weak_ref.clone()),
            true,
            span,
        )?;
        self.install_metadata_member(
            &weak_ref_prototype,
            function_prototype,
            Member::WeakRefDeref,
            &mut objects,
            span,
        )?;
        self.define_builtin_property(
            &finalization_registry_prototype,
            "constructor",
            Value::Object(finalization_registry.clone()),
            true,
            span,
        )?;
        for member in [
            Member::FinalizationRegistryRegister,
            Member::FinalizationRegistryUnregister,
        ] {
            self.install_metadata_member(
                &finalization_registry_prototype,
                function_prototype,
                member,
                &mut objects,
                span,
            )?;
        }
        for member in [
            Member::AtomicsAdd,
            Member::AtomicsAnd,
            Member::AtomicsCompareExchange,
            Member::AtomicsExchange,
            Member::AtomicsIsLockFree,
            Member::AtomicsLoad,
            Member::AtomicsOr,
            Member::AtomicsPause,
            Member::AtomicsStore,
            Member::AtomicsSub,
            Member::AtomicsWait,
            Member::AtomicsWaitAsync,
            Member::AtomicsNotify,
            Member::AtomicsXor,
        ] {
            self.install_metadata_member(&atomics, function_prototype, member, &mut objects, span)?;
        }
        for (prototype, name) in [
            (&promise_prototype, "Promise"),
            (&shared_array_buffer_prototype, "SharedArrayBuffer"),
            (&weak_ref_prototype, "WeakRef"),
            (&finalization_registry_prototype, "FinalizationRegistry"),
            (&atomics, "Atomics"),
        ] {
            self.define_property_or_throw(
                prototype,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from(name))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        debug_assert_eq!(objects.len(), 40);
        Ok(GlobalMetadataIntrinsics {
            promise,
            promise_prototype,
            proxy,
            shared_array_buffer,
            weak_ref,
            weak_ref_prototype,
            finalization_registry,
            finalization_registry_prototype,
            atomics,
            objects,
        })
    }

    fn metadata_prototype(
        &mut self,
        constructor: &ObjectHandle,
        object_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        self.define_property_or_throw(
            constructor,
            JsString::from("prototype"),
            DataDescriptor {
                value: Some(Value::Object(prototype.clone())),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )?;
        Ok(prototype)
    }

    fn install_metadata_member(
        &mut self,
        target: &ObjectHandle,
        function_prototype: &ObjectHandle,
        member: Member,
        roots: &mut Vec<ObjectHandle>,
        span: Span,
    ) -> Result<(), Error> {
        let function = self.new_builtin(
            function_prototype,
            Builtin::GlobalMetadataMember(member),
            span,
        )?;
        let key: Option<PropertyKey> = match member {
            Member::PromiseSpecies | Member::SharedArrayBufferSpecies => {
                Some(WellKnownSymbol::Species.symbol().into())
            }
            Member::SharedArrayBufferByteLength => Some(JsString::from("byteLength").into()),
            Member::SharedArrayBufferGrowable => Some(JsString::from("growable").into()),
            Member::SharedArrayBufferMaxByteLength => Some(JsString::from("maxByteLength").into()),
            _ => None,
        };
        if let Some(key) = key {
            self.define_property_or_throw(
                target,
                key,
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(function.clone())),
                        set: Some(None),
                    },
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                span,
            )?;
        } else {
            self.define_builtin_property(
                target,
                member.name(),
                Value::Object(function.clone()),
                true,
                span,
            )?;
        }
        roots.push(function);
        Ok(())
    }

    pub(super) fn global_metadata_method(
        &mut self,
        member: Member,
        receiver: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        match member {
            // sec-get-promise-%symbol.species% / the SharedArrayBuffer species
            // getter: these generic accessors return this without
            // receiver branding, coercion, allocation, or species recursion.
            Member::PromiseSpecies | Member::SharedArrayBufferSpecies => Ok(receiver),
            Member::WeakRefDeref => self.weak_ref_deref(receiver, span),
            Member::PromiseWithResolvers => self.promise_with_resolvers(receiver, span),
            Member::PromiseThen => self.promise_then(
                receiver,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Member::PromiseResolve => {
                self.promise_resolve(receiver, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Member::PromiseCatch => {
                self.promise_catch(receiver, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Member::PromiseFinally => {
                self.promise_finally(receiver, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Member::PromiseReject => {
                self.promise_reject(receiver, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Member::PromiseTry => self.promise_try(receiver, arguments, span),
            // Atomics.pause (sec-atomics.pause) observes no receiver/arguments.
            // The platform's spin-loop hint has no JavaScript-visible result.
            Member::AtomicsPause => {
                std::hint::spin_loop();
                Ok(Value::Undefined)
            }
            Member::AtomicsAdd
            | Member::AtomicsAnd
            | Member::AtomicsCompareExchange
            | Member::AtomicsExchange
            | Member::AtomicsIsLockFree
            | Member::AtomicsLoad
            | Member::AtomicsOr
            | Member::AtomicsStore
            | Member::AtomicsSub
            | Member::AtomicsWait
            | Member::AtomicsWaitAsync
            | Member::AtomicsNotify
            | Member::AtomicsXor => self.atomics_method(member, arguments, span),
            _ => Err(Self::unsupported(
                span,
                format!("{} is not implemented", member.name()),
            )),
        }
    }
}
