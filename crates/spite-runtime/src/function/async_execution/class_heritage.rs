//! ClassDeclaration heritage suspension shares ClassDefinitionEvaluation (15.7.14).

use super::{AsyncAwaitAction, AsyncExecutionData, Step, active_statement};
use crate::{Completion, Error, Realm, Value, object::async_expression::ExprRoot};
use spite_parser::ast::StatementKind;

impl Realm {
    pub(super) fn start_class_heritage(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Step, Error> {
        let StatementKind::Class(class) =
            &active_statement(&data.body, data.statement.as_ref()).kind
        else {
            unreachable!("class declaration")
        };
        data.class_heritage = Some(self.begin_class_definition(class)?);
        self.start_async_expression(
            data,
            ExprRoot::ClassHeritage,
            AsyncAwaitAction::ClassHeritage,
        )
    }

    pub(super) fn finish_class_heritage(
        &mut self,
        data: &mut AsyncExecutionData,
        superclass: Value,
    ) -> Result<Step, Error> {
        let StatementKind::Class(class) =
            &active_statement(&data.body, data.statement.as_ref()).kind
        else {
            unreachable!("class declaration")
        };
        let default = data.function.module_default().is_some_and(|index| {
            data.statement
                .as_ref()
                .is_some_and(|location| location.root == index && location.children.is_empty())
        });
        let context = data.class_heritage.take().expect("class name environment");
        let value = self.class_definition_from_heritage(
            class,
            default.then(|| "default".into()),
            context,
            Some(superclass),
        )?;
        if default {
            self.initialize_module_default(
                data.root_scope.as_ref().expect("module environment"),
                value,
                class.source.span(),
            )?;
        } else {
            let name = class.name.as_ref().expect("class declaration name");
            let environment = self.scopes.last().expect("declaration environment");
            self.objects
                .environment_mut(environment)
                .expect("declaration environment")
                .bindings
                .get_mut(&name.name)
                .expect("instantiated class binding")
                .value = Some(value);
        }
        Ok(Step::Complete(Completion::normal(None)))
    }
}
