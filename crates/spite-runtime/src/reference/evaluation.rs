//! Identifier resolution and Reference GetValue/PutValue.

use crate::environment::BindingState;
use crate::reference::Reference;
use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};
use spite_parser::ast::{Expr, ExprKind, PropertyName};

impl Realm {
    pub(crate) fn resolve<'a>(
        &mut self,
        name: &'a str,
        span: Span,
    ) -> Result<Reference<'a>, Error> {
        let mut next = self.scopes.last().cloned();
        while let Some(handle) = next {
            self.tick(span)?;
            let scope = self
                .objects
                .environment(&handle)
                .expect("active environment");
            if scope.bindings.contains_key(name) {
                return Ok(Reference::Lexical(handle, name.into()));
            }
            if let Some(binding) = scope.imports.get(name) {
                return Ok(Reference::Import(binding.clone()));
            }

            // Release the heap borrow before observable unscopables getters.
            let object = scope.binding_object.clone();
            next = scope.outer.clone();
            if let Some(object) = object
                && self.with_has_binding(&object, name, span)?
            {
                return Ok(Reference::ObjectBinding {
                    object,
                    name: name.into(),
                });
            }
        }
        if self.record.unsupported_host_globals.contains(name)
            && self.global_own(name, span)?.is_none()
        {
            return Ok(Reference::UnsupportedGlobal(name.into()));
        }
        let object = self.global_object();
        let key = JsString::from(name);
        Ok(if self.has_property(&object, &key, span)? {
            Reference::Global {
                name: name.into(),
                key,
            }
        } else {
            Reference::Unresolvable(name.into())
        })
    }

    pub(crate) fn reference<'a>(&mut self, target: &'a Expr) -> Result<Reference<'a>, Error> {
        match &target.kind {
            ExprKind::Identifier(name) => self.resolve(name, target.span),
            ExprKind::OptionalChain { base, steps } => self.optional_chain_reference(base, steps),
            ExprKind::Parenthesized(inner) => self.reference(inner),
            ExprKind::SuperProperty(name) => self.super_property_reference(name, target.span),
            ExprKind::Member(base, name) => {
                let base = self.expression(base)?;
                let key = match name {
                    PropertyName::Private(name) => {
                        let name = self.resolve_private_name(&name.name, name.span)?;
                        return Ok(Reference::Private { base, name });
                    }
                    PropertyName::Literal(literal) => self.literal_value(literal, target.span)?,
                    PropertyName::Computed(expression) => self.expression(expression)?,
                };
                Ok(Reference::Property { base, key })
            }
            _ => unreachable!("parser validated reference target"),
        }
    }
    pub(crate) fn get(
        &mut self,
        reference: &mut Reference<'_>,
        span: Span,
    ) -> Result<Value, Error> {
        match reference {
            Reference::Import(binding) => self.read_module_binding(binding, span),
            Reference::Value(value) => {
                self.object_work(span, |_, budget| budget.value(value))?;
                Ok(value.clone())
            }
            Reference::Property { base, key } => {
                Self::require_object_coercible(base, span)?;
                let key = self.reference_key(key, span)?;
                self.get_property_value(base, &key, span)
            }
            Reference::Private { base, name } => self.private_get(base, name, span),
            Reference::SuperProperty {
                base,
                key,
                this_value,
            } => {
                Self::require_object_coercible(base, span)?;
                let key = self.reference_key(key, span)?;
                let Value::Object(base) = base else {
                    unreachable!("a non-null super base is an object")
                };
                self.get_property_with_receiver(base, &key, this_value.clone(), span)
            }
            Reference::ObjectBinding { object, name } => self.with_get_binding(object, name, span),
            Reference::Lexical(handle, name) => self
                .objects
                .environment(handle)
                .expect("resolved environment")
                .bindings[name.as_ref()]
            .value
            .clone()
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::ReferenceError,
                    span,
                    format!("{name} is uninitialized"),
                )
            }),
            Reference::Global { name, key } => self.get_global(name, key, span),
            Reference::Unresolvable(name) => Err(Self::exception(
                ExceptionKind::ReferenceError,
                span,
                format!("{name} is not defined"),
            )),
            Reference::UnsupportedGlobal(name) => Err(Self::unsupported(
                span,
                format!("{name} is not implemented"),
            )),
        }
    }
    pub(crate) fn put(
        &mut self,
        reference: Reference<'_>,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        match reference {
            Reference::Import(_) => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "cannot assign to an imported binding",
                ));
            }

            Reference::Value(_) => unreachable!("parser rejects optional-chain assignment targets"),
            Reference::Private { base, name } => self.private_set(&base, &name, value, span)?,
            Reference::Property { base, mut key } => {
                Self::require_object_coercible(&base, span)?;
                let key = self.reference_key(&mut key, span)?;
                let written = self.set_property_value(&base, key, value, span)?;
                if !written && self.strict {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "property is not writable",
                    ));
                }
            }
            Reference::ObjectBinding { object, name } => {
                self.with_set_binding(&object, &name, value, span)?;
            }
            Reference::SuperProperty {
                base,
                mut key,
                this_value,
            } => {
                Self::require_object_coercible(&base, span)?;
                let key = self.reference_key(&mut key, span)?;
                let Value::Object(base) = base else {
                    unreachable!("a non-null super base is an object")
                };
                let written =
                    self.set_property_with_receiver(&base, key, value, this_value, span)?;
                if !written && self.strict {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "super property is not writable",
                    ));
                }
            }
            Reference::Lexical(handle, name) => {
                let bindings = &mut self
                    .objects
                    .environment_mut(&handle)
                    .expect("resolved environment")
                    .bindings;
                let Some(binding) = bindings.get_mut(name.as_ref()) else {
                    // SetMutableBinding (9.1.1.1.5): an eval-created binding
                    // can be deleted after resolving the assignment's LHS.
                    if self.strict {
                        return Err(Self::exception(
                            ExceptionKind::ReferenceError,
                            span,
                            format!("{name} is not defined"),
                        ));
                    }
                    bindings.insert(
                        name.into_owned(),
                        BindingState {
                            value: Some(value),
                            mutable: true,
                            deletable: true,
                            strict: false,
                        },
                    );
                    return Ok(());
                };
                if binding.value.is_none() {
                    return Err(Self::exception(
                        ExceptionKind::ReferenceError,
                        span,
                        format!("{name} is uninitialized"),
                    ));
                }
                if !binding.mutable {
                    if !binding.strict && !self.strict {
                        return Ok(());
                    }
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        format!("{name} is immutable"),
                    ));
                }
                binding.value = Some(value);
            }
            Reference::Unresolvable(name) if self.strict => {
                return Err(Self::exception(
                    ExceptionKind::ReferenceError,
                    span,
                    format!("{name} is not defined"),
                ));
            }
            Reference::Global { name, key } => self.put_global(&name, key, value, true, span)?,
            Reference::Unresolvable(name) => {
                self.put_global(&name, JsString::from(name.as_ref()), value, false, span)?;
            }
            Reference::UnsupportedGlobal(name) => {
                return Err(Self::unsupported(
                    span,
                    format!("{name} is not implemented"),
                ));
            }
        }
        Ok(())
    }

    // ECMA-262 14.15.2: the parameter environment encloses the block environment
    // and is restored before any pending finally or outer handler executes.
}
