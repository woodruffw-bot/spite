//! String pattern-hook delegation and matchAll flag checks (22.1.3.11–12, 21).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    pub(crate) fn string_pattern_hook(
        &mut self,
        receiver: Value,
        pattern: Value,
        symbol: WellKnownSymbol,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        // Edition 17 only consults hooks on object arguments; primitive
        // prototype hooks are ignored. Delegation retains the original receiver.
        if let Value::Object(object) = &pattern {
            if matches!(symbol, WellKnownSymbol::MatchAll) && self.is_regexp(&pattern, span)? {
                let flags = self.get_property(object, &JsString::from("flags"), span)?;
                Self::require_object_coercible(&flags, span)?;
                let flags = self.string(flags, span)?;
                self.object_work(span, |_, budget| budget.charge(flags.len()))?;
                if !flags.code_units().contains(&u16::from(b'g')) {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "String.matchAll requires a global regular expression",
                    ));
                }
            }
            if let Some(method) = self.get_method(&pattern, &symbol.symbol(), span)? {
                return self.call(method, pattern, vec![receiver], span);
            }
        }
        // RegExpCreate uses the intrinsic directly without RegExp's callable
        // identity or native-copy rules. Invoke then reads the new object's live
        // Symbol method; only matchAll supplies the global flag (22.1.3.11–12, 21).
        let string = self.string(receiver, span)?;
        let flags = if matches!(symbol, WellKnownSymbol::MatchAll) {
            Value::String(JsString::from("g"))
        } else {
            Value::Undefined
        };
        let regexp = self.regexp_create(pattern, flags, span)?;
        let Value::Object(object) = &regexp else {
            unreachable!("RegExpCreate returns Object")
        };
        let method = self.get_property(object, &symbol.symbol(), span)?;
        self.call(method, regexp, vec![Value::String(string)], span)
    }
}
