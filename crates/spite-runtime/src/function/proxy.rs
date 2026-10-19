//! Explicit invocation boundary for private foundation-only Proxy objects.

use crate::{Error, ExceptionKind, Realm, Value, object::ProxyState};
use spite_core::Span;

impl Realm {
    pub(super) fn proxy_unimplemented_invocation(
        &mut self,
        state: ProxyState,
        span: Span,
    ) -> Result<Value, Error> {
        match state {
            ProxyState::Revoked => Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "revoked Proxy",
            )),
            ProxyState::Live(_) => Err(Self::unsupported(
                span,
                "Proxy invocation is not implemented in the private foundation",
            )),
        }
    }
}
