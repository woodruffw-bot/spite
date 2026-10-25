//! The standard doneprintHandle.js print protocol in a fresh conformance realm.

use super::{Outcome, Stage, failed, host_failure};
use spite_core::JsString;
use spite_runtime::{Error, Realm, RootedValue, Value};

const PRINT_HOST: &str = r#"
(function() {
    const messages = Object.create(null), asString = String;
    let next = 0;
    globalThis.print = function print(value) {
        messages[next++] = asString(value);
    };
    return messages;
})()
"#;

pub(super) struct AsyncCompletion {
    messages: RootedValue,
}

impl AsyncCompletion {
    pub(super) fn install(realm: &mut Realm) -> Result<Self, Error> {
        let messages = realm.eval(PRINT_HOST)?;
        let messages = realm
            .root_value(messages, usize::MAX)
            .map_err(Error::InvalidObject)?;
        Ok(Self { messages })
    }

    pub(super) fn finish(&self, realm: &mut Realm) -> Result<(), Outcome> {
        let mut index = 0usize;
        let mut complete = 0usize;
        let mut failure = None;
        loop {
            let value = realm
                .read_property(
                    self.messages.value(),
                    &JsString::from(index.to_string().as_str()),
                )
                .map_err(|error| {
                    host_failure(&error, Stage::Runtime).unwrap_or_else(|| {
                        failed(
                            Stage::Runtime,
                            format!("async completion inspection failed: {error}"),
                        )
                    })
                })?;
            let message = match value {
                Value::Undefined => break,
                Value::String(value) => String::from_utf16_lossy(value.code_units()),
                _ => {
                    return Err(Outcome::SetupFailure(
                        "async print capture was not a String".into(),
                    ));
                }
            };
            if message == "Test262:AsyncTestComplete" {
                complete += 1;
            } else if message.starts_with("Test262:AsyncTestFailure:") {
                failure.get_or_insert(message);
                complete += 1;
            }
            if complete > 1 {
                return Err(failed(
                    Stage::Runtime,
                    "asynchronous test completed more than once",
                ));
            }
            index = index.checked_add(1).ok_or_else(|| Outcome::Limit {
                stage: Stage::Runtime,
                message: "async print index capacity exceeded".into(),
            })?;
        }
        if let Some(message) = failure {
            return Err(failed(Stage::Runtime, message));
        }
        if complete != 1 {
            return Err(failed(
                Stage::Runtime,
                "asynchronous test did not complete through $DONE",
            ));
        }
        Ok(())
    }
}
