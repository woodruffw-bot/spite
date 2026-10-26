//! Untagged template substitution evaluation and immediate ToString (13.2.8.6).

use super::{AsyncExecutionData, child_path, expression_at, push};
use crate::{
    Error, Realm, Value,
    object::async_expression::{AsyncExpressionData, ExprChild, ExprPath, ExprTask},
};
use spite_core::JsString;
use spite_parser::ast::ExprKind;

impl Realm {
    pub(super) fn async_template_next(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        index: usize,
        mut units: Vec<u16>,
    ) -> Result<(), Error> {
        let expression = expression_at(data, state.root, &path);
        let ExprKind::Template {
            elements,
            substitutions,
        } = &expression.kind
        else {
            unreachable!("template source path")
        };
        let span = expression.span;
        self.append_template_element(&mut units, &elements[index])?;
        if index < substitutions.len() {
            let child = child_path(&path, ExprChild::TemplateSubstitution(index), span)?;
            push(state, ExprTask::TemplateAppend { path, index, units }, span)?;
            push(state, ExprTask::Evaluate(child), span)
        } else {
            state.current = Some(Value::String(JsString::from_code_units(units)));
            Ok(())
        }
    }

    pub(super) fn async_template_append(
        &mut self,
        data: &AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        index: usize,
        mut units: Vec<u16>,
    ) -> Result<(), Error> {
        let expression = expression_at(data, state.root, &path);
        let ExprKind::Template { substitutions, .. } = &expression.kind else {
            unreachable!("template source path")
        };
        let value = state.current.take().expect("template substitution value");
        let span = substitutions[index].span;
        let part = self.string(value, span)?;
        self.append_string(&mut units, &part, span)?;
        push(
            state,
            ExprTask::TemplateNext {
                path,
                index: index + 1,
                units,
            },
            expression.span,
        )
    }
}
