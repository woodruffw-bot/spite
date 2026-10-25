//! StatementList completion values and lexical/conditional frame unwinding.

use super::AsyncExecutionData;
use crate::{
    Completion, CompletionKind, Error, Realm, Value, object::async_execution::AsyncStatementFrame,
};

impl Realm {
    pub(super) fn handle_async_completion(
        &mut self,
        data: &mut AsyncExecutionData,
        mut result: Result<Completion, Error>,
    ) -> Result<Option<Completion>, Error> {
        if !matches!(&result, Ok(completion) if completion.kind == CompletionKind::Normal) {
            // An abandoned expression or initializer must not retain operands
            // and target environments through later completion handling.
            data.expression = None;
            data.var_target = None;
            data.next = None;
        }
        loop {
            result = match result {
                Err(error) if !error.is_language_exception() => return Err(error),
                result => result,
            };
            match data.frames.last_mut() {
                Some(AsyncStatementFrame::Block(block)) if matches!(&result, Ok(completion) if completion.kind == CompletionKind::Normal) =>
                {
                    let Ok(mut completion) = result else {
                        unreachable!("normal statement");
                    };
                    if completion.value.is_some() {
                        block.normal = completion.value.take();
                    }
                    return Ok(None);
                }
                Some(AsyncStatementFrame::Block(_)) => {
                    let Some(AsyncStatementFrame::Block(block)) = data.frames.pop() else {
                        unreachable!("block frame");
                    };
                    result = self.finish_async_block(block, result);
                }
                Some(AsyncStatementFrame::If) => {
                    data.frames.pop();
                    result =
                        result.map(|completion| completion.update_empty(Some(Value::Undefined)));
                }
                None => {
                    return match result {
                        Ok(mut completion) if completion.kind == CompletionKind::Normal => {
                            if completion.value.is_some() {
                                data.normal = completion.value.take();
                            }
                            Ok(None)
                        }
                        Ok(completion) => Ok(Some(completion.update_empty(data.normal.take()))),
                        Err(error) => {
                            data.normal = None;
                            Err(error)
                        }
                    };
                }
            }
        }
    }
}
