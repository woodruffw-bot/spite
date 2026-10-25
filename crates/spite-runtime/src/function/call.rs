//! Shared call target preparation and invocation (13.3.6.1).

use crate::{Error, Realm, Value, reference_expression};
use spite_core::Span;
use spite_parser::ast::Expr;

#[derive(Debug)]
pub(crate) struct CallTarget {
    pub function: Value,
    pub this: Value,
    pub direct_eval: bool,
}

impl CallTarget {
    pub(crate) fn values(&self) -> impl Iterator<Item = &Value> {
        [&self.function, &self.this].into_iter()
    }
}

impl Realm {
    pub(crate) fn prepare_call(&mut self, callee: &Expr) -> Result<CallTarget, Error> {
        let (function, this, direct_eval) = if reference_expression(callee) {
            let mut reference = self.reference(callee)?;
            let function = self.get(&mut reference, callee.span)?;
            // With/local eval references are direct only for the intrinsic eval.
            let direct_eval = reference.is_eval_environment_reference()
                && matches!(&function, Value::Object(handle) if handle == &self.intrinsics.as_ref().expect("initialized").eval);
            (function, reference.call_receiver(), direct_eval)
        } else {
            (self.expression(callee)?, Value::Undefined, false)
        };
        Ok(CallTarget {
            function,
            this,
            direct_eval,
        })
    }

    pub(crate) fn invoke_call(
        &mut self,
        target: CallTarget,
        values: Vec<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        if target.direct_eval {
            self.perform_eval(
                values.into_iter().next().unwrap_or(Value::Undefined),
                true,
                span,
            )
        } else {
            self.call(target.function, target.this, values, span)
        }
    }
}
