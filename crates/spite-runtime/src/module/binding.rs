//! ResolvedBinding Records and read-only imported references (16.2.1, 9.1.1.5).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::Span;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModuleBinding {
    pub module: ObjectHandle,
    // None denotes the namespace marker, distinct from every String name.
    pub name: Option<Rc<str>>,
}

impl Realm {
    pub(crate) fn read_module_binding(
        &mut self,
        binding: &ModuleBinding,
        span: Span,
    ) -> Result<Value, Error> {
        let Some(name) = &binding.name else {
            return self.module_namespace_record(&binding.module, span);
        };
        let value = self
            .objects
            .module_environment(&binding.module)
            .map_err(Error::InvalidObject)?
            .and_then(|environment| {
                self.objects
                    .environment(&environment)
                    .expect("module environment")
                    .bindings
                    .get(name.as_ref())
                    .and_then(|binding| binding.value.clone())
            });
        value.ok_or_else(|| {
            Self::exception(
                ExceptionKind::ReferenceError,
                span,
                format!("{name} is uninitialized"),
            )
        })
    }
}
