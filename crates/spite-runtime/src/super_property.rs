//! SuperProperty and MakeSuperPropertyReference (13.3.7.1, 13.3.7.3).

use crate::{Error, Realm, Reference, Value};
use spite_core::Span;
use spite_parser::ast::PropertyName;

impl Realm {
    pub(super) fn super_property_reference<'a>(
        &mut self,
        name: &'a PropertyName,
        span: Span,
    ) -> Result<Reference<'a>, Error> {
        // GetThisBinding precedes computed-name evaluation. GetSuperBase follows
        // it, so the key expression can replace the home object's prototype.
        let this_value = self.this_value(span)?;
        let key = match name {
            PropertyName::Literal(literal) => self.literal_value(literal, span)?,
            PropertyName::Computed(expression) => self.expression(expression)?,
        };
        let mut next = self.scopes.last().cloned();
        let mut base = Value::Undefined;
        while let Some(environment) = next {
            let (function, home, outer) = self.object_work(span, |objects, budget| {
                budget.charge(1)?;
                let environment = objects.environment(&environment)?;
                Ok((
                    environment.this.is_some(),
                    environment.home_object.clone(),
                    environment.outer.clone(),
                ))
            })?;
            if function {
                if let Some(home) = home {
                    base = self.object_work(span, |objects, _| {
                        Ok(objects
                            .inspect(&home)?
                            .prototype()
                            .cloned()
                            .map_or(Value::Null, Value::Object))
                    })?;
                }
                break;
            }
            next = outer;
        }
        Ok(Reference::SuperProperty {
            base,
            key,
            this_value,
        })
    }
}
