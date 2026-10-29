//! ClassExpression suspension uses the shared ordered class builder (15.7.14).

use super::{AsyncExecutionData, child_path, expression_at, has_supported_await, object, push};
use crate::{
    Error, Realm, Value,
    object::async_expression::{
        AsyncClassExpression, AsyncExpressionData, ExprChild, ExprPath, ExprTask,
    },
};
use spite_core::PropertyKey;
use spite_parser::ast::{Expr, ExprKind, PropertyName};

pub(super) fn named_task(expression: &Expr, path: ExprPath, name: PropertyKey) -> ExprTask {
    if crate::function::class::anonymous_class(expression) {
        let name = match name {
            PropertyKey::String(name) => Value::String(name),
            PropertyKey::Symbol(name) => Value::Symbol(name),
        };
        ExprTask::EvaluateNamed { path, name }
    } else {
        ExprTask::Evaluate(path)
    }
}

impl Realm {
    pub(super) fn async_named_class(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        name: Value,
    ) -> Result<(), Error> {
        let expression = expression_at(data, state.root, &path);
        if crate::function::class::anonymous_class(expression) && has_supported_await(expression) {
            let span = expression.span;
            self.tick(span)?;
            push(state, ExprTask::CheckString(span), span)?;
            if matches!(expression.kind, ExprKind::Parenthesized(_)) {
                let path = child_path(&path, ExprChild::Inner, span)?;
                push(state, ExprTask::EvaluateNamed { path, name }, span)
            } else {
                self.begin_async_class_expression(data, state, path, Some(object::value_key(name)))
            }
        } else {
            push(state, ExprTask::Evaluate(path), expression.span)
        }
    }

    pub(super) fn begin_async_class_expression(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        name: Option<PropertyKey>,
    ) -> Result<(), Error> {
        let expression = expression_at(data, state.root, &path);
        let ExprKind::Class(class) = &expression.kind else {
            unreachable!("class expression")
        };
        let class = class.clone();
        let span = expression.span;
        data.expression_classes
            .try_reserve(1)
            .map_err(|_| Error::Limit {
                span,
                message: "class continuation allocation failed".into(),
            })?;
        let context = self.begin_class_definition(&class)?;
        data.expression_classes.push(AsyncClassExpression {
            syntax: class.clone(),
            context,
            definition: None,
            name,
        });
        if class.heritage.as_ref().is_some_and(has_supported_await) {
            let child = child_path(&path, ExprChild::ClassHeritage, span)?;
            push(state, ExprTask::ClassHeritage { path }, span)?;
            push(state, ExprTask::Evaluate(child), span)
        } else {
            let superclass = class
                .heritage
                .as_ref()
                .map(|heritage| self.expression(heritage))
                .transpose()?;
            self.prepare_async_class_expression(data, state, path, superclass)
        }
    }

    pub(super) fn prepare_async_class_expression(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        superclass: Option<Value>,
    ) -> Result<(), Error> {
        let record = data
            .expression_classes
            .last_mut()
            .expect("active class expression");
        record.definition =
            Some(self.prepare_class_definition(&record.syntax, record.name.take(), superclass)?);
        self.advance_async_class_expression(data, state, path, 0)
    }

    pub(super) fn advance_async_class_expression(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        first: usize,
    ) -> Result<(), Error> {
        let class = data
            .expression_classes
            .last()
            .expect("active class expression")
            .syntax
            .clone();
        for (element, definition) in class.elements.iter().enumerate().skip(first) {
            let span = definition.span();
            self.tick(span)?;
            if matches!(definition.name(), Some(PropertyName::Computed(key)) if has_supported_await(key))
            {
                let child = child_path(&path, ExprChild::ClassKey(element), span)?;
                push(state, ExprTask::ClassKey { path, element }, span)?;
                return push(state, ExprTask::Evaluate(child), span);
            }
            let name = self.class_element_name(definition)?;
            let record = data
                .expression_classes
                .last_mut()
                .expect("active class expression");
            self.define_class_element(
                record.definition.as_mut().expect("prepared class"),
                &class,
                definition,
                name,
            )?;
        }
        let record = data
            .expression_classes
            .pop()
            .expect("active class expression");
        state.current = Some(self.finish_class_definition(
            &class,
            record.context,
            record.definition.expect("prepared class"),
        )?);
        Ok(())
    }

    pub(super) fn finish_async_class_expression_key(
        &mut self,
        data: &mut AsyncExecutionData,
        state: &mut AsyncExpressionData,
        path: ExprPath,
        element: usize,
        value: Value,
    ) -> Result<(), Error> {
        let record = data
            .expression_classes
            .last_mut()
            .expect("active class expression");
        let class = record.syntax.clone();
        let definition = &class.elements[element];
        let key = self.property_key(value, definition.span())?;
        self.define_class_element(
            record.definition.as_mut().expect("prepared class"),
            &class,
            definition,
            Some(crate::function::class_field::ClassFieldName::Public(key)),
        )?;
        push(
            state,
            ExprTask::ClassNext {
                path,
                element: element + 1,
            },
            definition.span(),
        )
    }
}
