//! Ordered static fields/blocks and EvaluateClassStaticBlockBody (15.7).

use super::{ScriptFunction, class_field::ClassField};
use crate::{Error, ObjectHandle, Realm, Value};
use spite_core::Span;

pub(super) enum StaticElement {
    Field(ClassField),
    Block {
        code: ScriptFunction,
        home_object: ObjectHandle,
        span: Span,
    },
}

impl Realm {
    pub(super) fn initialize_static_elements(
        &mut self,
        constructor: &Value,
        elements: Vec<StaticElement>,
    ) -> Result<(), Error> {
        for element in elements {
            match element {
                StaticElement::Field(field) => self.initialize_fields(constructor, &[field])?,
                StaticElement::Block {
                    code,
                    home_object,
                    span,
                } => {
                    self.enter_call(span)?;
                    // The synthetic body is strict, takes no parameters, and
                    // performs ordinary declaration instantiation. Its internal
                    // function cannot be obtained by JavaScript (15.7.11).
                    let result = self.call_ordinary(
                        code,
                        home_object.clone(),
                        constructor.clone(),
                        crate::environment::FunctionContext {
                            home_object: Some(home_object),
                            ..Default::default()
                        },
                        Vec::new().into_iter(),
                        span,
                    );
                    self.call_depth -= 1;
                    result?;
                }
            }
        }
        Ok(())
    }
}
