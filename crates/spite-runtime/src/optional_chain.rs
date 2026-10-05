//! Iterative ChainEvaluation preserving final references and call receivers (13.3.10).

use crate::{Error, Realm, Reference, Value, reference_expression};
use spite_parser::ast::{ChainStep, ChainStepKind, Expr, PropertyName};

impl Realm {
    pub(super) fn optional_chain_reference<'a>(
        &mut self,
        base: &'a Expr,
        steps: &'a [ChainStep],
    ) -> Result<Reference<'a>, Error> {
        let mut reference = if reference_expression(base) {
            self.reference(base)?
        } else {
            Reference::Value(self.expression(base)?)
        };
        for step in steps {
            self.tick(step.span)?;
            let value = self.get(&mut reference, step.span)?;
            if step.optional && matches!(value, Value::Undefined | Value::Null) {
                // Skip the entire ungrouped suffix, including ordinary calls
                // and properties. Parentheses create a separate outer node.
                return Ok(Reference::Value(Value::Undefined));
            }
            reference = match &step.kind {
                ChainStepKind::Property(name) => {
                    let key = match name {
                        PropertyName::Private(_) => {
                            return Err(Self::unsupported(
                                step.span,
                                "private elements are not implemented",
                            ));
                        }
                        PropertyName::Literal(literal) => self.literal_value(literal, step.span)?,
                        PropertyName::Computed(expression) => self.expression(expression)?,
                    };
                    // Edition 17 defers coercibility and computed key conversion
                    // to GetValue/PutValue/delete. Keep the final reference so
                    // delete never reads the property and calls retain its base.
                    Reference::Property { base: value, key }
                }
                ChainStepKind::Call(arguments) => {
                    let this = reference.call_receiver();
                    let arguments = self.argument_list(arguments)?;
                    Reference::Value(self.call(value, this, arguments, step.span)?)
                }
            };
        }
        Ok(reference)
    }
}
