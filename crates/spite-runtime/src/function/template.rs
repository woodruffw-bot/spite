//! Tagged call evaluation and the realm's template registry (13.3.11, 13.2.8.4).

use crate::{
    Error, ObjectHandle, Realm, Reference, Value, object::DataDescriptor, reference_expression,
};
use spite_core::{JsString, Span};
use spite_parser::ast::{Expr, TemplateElement};
use std::rc::Rc;

impl Realm {
    pub(crate) fn tagged_template(
        &mut self,
        tag: &Expr,
        elements: &Rc<[TemplateElement]>,
        substitutions: &[Expr],
        span: Span,
    ) -> Result<Value, Error> {
        let (function, this) = if reference_expression(tag) {
            let mut reference = self.reference(tag)?;
            let function = self.get(&mut reference, tag.span)?;
            let this = match reference {
                Reference::Property { base, .. } => base,
                _ => Value::Undefined,
            };
            (function, this)
        } else {
            (self.expression(tag)?, Value::Undefined)
        };
        let template = self.get_template_object(elements, span)?;
        let mut arguments = Vec::new();
        self.append_argument(&mut arguments, Value::Object(template), span)?;
        for substitution in substitutions {
            let value = self.expression(substitution)?;
            self.append_argument(&mut arguments, value, substitution.span)?;
        }
        // EvaluateCall checks callability after ArgumentListEvaluation.
        self.call(function, this, arguments, span)
    }

    fn get_template_object(
        &mut self,
        elements: &Rc<[TemplateElement]>,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        for index in 0..self.template_map.len() {
            self.tick(span)?;
            let (site, array) = &self.template_map[index];
            if Rc::ptr_eq(site, elements) {
                return Ok(array.clone());
            }
        }
        self.template_map.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "template registry allocation failed".into(),
        })?;
        // Parsing enforces the normative 2^32 component early error.
        let count = u32::try_from(elements.len()).expect("validated template count");
        let template = self.create_intrinsic_array(u64::from(count), span)?;
        let raw = self.create_intrinsic_array(u64::from(count), span)?;
        for (index, element) in elements.iter().enumerate() {
            self.tick(element.span)?;
            let key = JsString::from(index.to_string().as_str());
            let cooked = match &element.cooked {
                Some(string) => self.template_string(string, element.span)?,
                None => Value::Undefined,
            };
            self.define_property_or_throw(
                &template,
                key.clone(),
                DataDescriptor {
                    value: Some(cooked),
                    writable: Some(false),
                    enumerable: Some(true),
                    configurable: Some(false),
                }
                .into(),
                span,
            )?;
            let value = self.template_string(&element.raw, element.span)?;
            self.define_property_or_throw(
                &raw,
                key,
                DataDescriptor {
                    value: Some(value),
                    writable: Some(false),
                    enumerable: Some(true),
                    configurable: Some(false),
                }
                .into(),
                span,
            )?;
        }
        self.object_set_integrity(Value::Object(raw.clone()), true, span)?;
        self.define_property_or_throw(
            &template,
            JsString::from("raw"),
            DataDescriptor {
                value: Some(Value::Object(raw)),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )?;
        self.object_set_integrity(Value::Object(template.clone()), true, span)?;
        // Host failures before this point leave no partially initialized cache
        // entry. Evaluation never collects; completed Arrays become realm roots.
        self.template_map.push((elements.clone(), template.clone()));
        Ok(template)
    }

    fn template_string(&mut self, string: &JsString, span: Span) -> Result<Value, Error> {
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| string.len() > limit)
        {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        self.object_work(span, |_, budget| budget.charge(string.len()))?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(string.len())
            .map_err(|_| Error::Limit {
                span,
                message: "template string allocation failed".into(),
            })?;
        units.extend_from_slice(string.code_units());
        Ok(Value::String(JsString::from_code_units(units)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Limits, test_support::REALM_ENTRIES};

    #[test]
    fn failed_array_or_raw_construction_does_not_publish_a_partial_template() {
        for limits in [
            Limits {
                max_heap_entries: Some(REALM_ENTRIES + 2),
                ..Limits::default()
            },
            Limits {
                max_string_units: Some(3),
                ..Limits::default()
            },
        ] {
            let mut realm = Realm::new(limits);
            realm.eval("let tag=()=>{flag=3;},flag=0;").unwrap();
            let script =
                spite_parser::parse_script("try{tag`\\u0061`;}catch{flag=1;}finally{flag=2;}")
                    .unwrap();
            for _ in 0..2 {
                assert!(matches!(realm.evaluate(&script), Err(Error::Limit { .. })));
                assert!(realm.template_map.is_empty());
                assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 1);
                assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
            }
        }
    }
}
