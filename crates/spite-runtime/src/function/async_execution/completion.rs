//! StatementList completion values and lexical/conditional frame unwinding.

use super::{AsyncExecutionData, FrameStep};
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
            data.iteration_value = None;
            data.next = None;
        }
        if data.array_binding.is_some() {
            result = result.map_err(|error| self.close_async_array_binding_error(data, error));
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
                Some(AsyncStatementFrame::Switch(_)) => {
                    match self.advance_async_switch(data, result) {
                        FrameStep::Continue => return Ok(None),
                        FrameStep::Complete(completion) => result = completion,
                    }
                }
                Some(AsyncStatementFrame::With(_)) => {
                    let Some(AsyncStatementFrame::With(environment)) = data.frames.pop() else {
                        unreachable!("with completion frame")
                    };
                    let active = self.scopes.pop().expect("active with environment");
                    debug_assert_eq!(active, environment);
                    result =
                        result.map(|completion| completion.update_empty(Some(Value::Undefined)));
                }
                Some(AsyncStatementFrame::If) => {
                    data.frames.pop();
                    result =
                        result.map(|completion| completion.update_empty(Some(Value::Undefined)));
                }
                Some(AsyncStatementFrame::For(_)) => match self.advance_async_for(data, result) {
                    FrameStep::Continue => return Ok(None),
                    FrameStep::Complete(completion) => result = completion,
                },
                Some(AsyncStatementFrame::Loop(_)) => match self.advance_async_loop(data, result) {
                    FrameStep::Continue => return Ok(None),
                    FrameStep::Complete(completion) => result = completion,
                },
                Some(AsyncStatementFrame::Try(_)) => match self.advance_async_try(data, result) {
                    FrameStep::Continue => return Ok(None),
                    FrameStep::Complete(completion) => result = completion,
                },
                Some(AsyncStatementFrame::Label(_)) => {
                    let Some(AsyncStatementFrame::Label(location)) = data.frames.pop() else {
                        unreachable!("label completion frame");
                    };
                    let spite_parser::ast::StatementKind::Labelled { label, .. } =
                        &super::active_statement(&data.body, Some(&location)).kind
                    else {
                        unreachable!("label source path");
                    };
                    result = result.map(|mut completion| {
                        if completion.kind == CompletionKind::Break
                            && completion.target.as_deref() == Some(label.name.as_str())
                        {
                            completion.kind = CompletionKind::Normal;
                            completion.target = None;
                        }
                        completion
                    });
                }
                Some(AsyncStatementFrame::IterationHead(_)) => {
                    let Some(AsyncStatementFrame::IterationHead(head)) = data.frames.pop() else {
                        unreachable!("iteration head frame");
                    };
                    if let Some(environment) = head.environment {
                        let scope = self.scopes.pop().expect("iteration head scope");
                        debug_assert_eq!(scope, environment);
                    }
                }
                Some(AsyncStatementFrame::ForIn(_)) => {
                    match self.advance_async_for_in(data, result) {
                        FrameStep::Continue => return Ok(None),
                        FrameStep::Complete(completion) => result = completion,
                    }
                }
                Some(AsyncStatementFrame::ForAwait(_)) => {
                    match self.advance_async_for_await(data, result) {
                        FrameStep::Continue => return Ok(None),
                        FrameStep::Complete(completion) => result = completion,
                    }
                }
                Some(AsyncStatementFrame::ForOf(_)) => {
                    match self.advance_async_for_of(data, result) {
                        FrameStep::Continue => return Ok(None),
                        FrameStep::Complete(completion) => result = completion,
                    }
                }
                Some(AsyncStatementFrame::Scope { .. }) => {
                    let Some(AsyncStatementFrame::Scope {
                        environment,
                        span,
                        disposable,
                    }) = data.frames.pop()
                    else {
                        unreachable!("lexical scope frame");
                    };
                    if disposable {
                        result = self.dispose_environment_resources(&environment, result, span);
                    }
                    let scope = self.scopes.pop().expect("active lexical scope");
                    debug_assert_eq!(scope, environment);
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
