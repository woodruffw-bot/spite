//! Ordinary internal-method boundaries for Proxy dispatch.

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{Property, PropertyDescriptor, ProxyState},
};
use spite_core::{PropertyKey, PropertyKeyRef, Span};

pub(super) fn owned_key(key: PropertyKeyRef<'_>) -> PropertyKey {
    match key {
        PropertyKeyRef::String(key) => PropertyKey::String(key.clone()),
        PropertyKeyRef::Symbol(key) => PropertyKey::Symbol(key.clone()),
    }
}

fn foundation_result<T>(result: Result<T, ProxyState>, span: Span) -> Result<T, Error> {
    match result {
        Ok(result) => Ok(result),
        Err(ProxyState::Revoked) => Err(Realm::exception(
            ExceptionKind::TypeError,
            span,
            "proxy has been revoked",
        )),
        Err(ProxyState::Live(_)) => Err(Realm::unsupported(
            span,
            "Proxy property internal methods are not implemented",
        )),
    }
}

impl Realm {
    fn proxy_property_gap<T>(&mut self, object: &ObjectHandle, span: Span) -> Result<T, Error> {
        let state = self
            .object_work(span, |objects, budget| objects.proxy_state(object, budget))?
            .expect("Proxy dispatch target");
        foundation_result(Err(state), span)
    }

    pub(super) fn proxy_get_property(
        &mut self,
        object: &ObjectHandle,
        _key: PropertyKey,
        _receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        self.proxy_property_gap(object, span)
    }
    pub(super) fn proxy_has_property(
        &mut self,
        object: &ObjectHandle,
        _key: PropertyKey,
        span: Span,
    ) -> Result<bool, Error> {
        self.proxy_property_gap(object, span)
    }
    pub(super) fn proxy_set_property(
        &mut self,
        object: &ObjectHandle,
        _key: PropertyKey,
        _assigned: Value,
        _receiver: Value,
        span: Span,
    ) -> Result<bool, Error> {
        self.proxy_property_gap(object, span)
    }
    pub(super) fn proxy_delete_property(
        &mut self,
        object: &ObjectHandle,
        _key: PropertyKey,
        span: Span,
    ) -> Result<bool, Error> {
        self.proxy_property_gap(object, span)
    }
    pub(super) fn proxy_own_property_keys(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Vec<PropertyKey>, Error> {
        self.proxy_property_gap(object, span)
    }
    pub(super) fn proxy_own_property_descriptor(
        &mut self,
        object: &ObjectHandle,
        _key: PropertyKey,
        span: Span,
    ) -> Result<Option<Property>, Error> {
        self.proxy_property_gap(object, span)
    }
    pub(crate) fn proxy_define_property(
        &mut self,
        object: &ObjectHandle,
        _key: PropertyKey,
        _descriptor: PropertyDescriptor,
        span: Span,
    ) -> Result<bool, Error> {
        self.proxy_property_gap(object, span)
    }
}
