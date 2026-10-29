//! ClassDeclaration heritage and computed-name suspension (15.7.14).

use super::{AsyncAwaitAction, AsyncExecutionData, Step, active_statement, expression};
use crate::{Completion, Error, Realm, Value, object::async_expression::ExprRoot};
use spite_parser::ast::{Class, PropertyName, StatementKind};
use std::rc::Rc;

pub(super) fn has_class_await(class: &Class) -> bool {
    class.heritage.as_ref().is_some_and(expression::has_supported_await)
        || class.elements.iter().any(|element| matches!(element.name(), Some(PropertyName::Computed(key)) if expression::has_supported_await(key)))
}

fn syntax(data: &AsyncExecutionData) -> Rc<Class> {
    let StatementKind::Class(class) = &active_statement(&data.body, data.statement.as_ref()).kind
    else {
        unreachable!("class declaration")
    };
    class.clone()
}

fn is_default(data: &AsyncExecutionData) -> bool {
    data.function.module_default().is_some_and(|index| {
        data.statement
            .as_ref()
            .is_some_and(|location| location.root == index && location.children.is_empty())
    })
}

impl Realm {
    pub(super) fn start_class_heritage(
        &mut self,
        data: &mut AsyncExecutionData,
    ) -> Result<Step, Error> {
        let class = syntax(data);
        data.class_context = Some(self.begin_class_definition(&class)?);
        if class
            .heritage
            .as_ref()
            .is_some_and(expression::has_supported_await)
        {
            return self.start_async_expression(
                data,
                ExprRoot::ClassHeritage,
                AsyncAwaitAction::ClassHeritage,
            );
        }
        let superclass = class
            .heritage
            .as_ref()
            .map(|heritage| self.expression(heritage))
            .transpose()?;
        self.prepare_async_class(data, superclass)
    }

    pub(super) fn finish_class_heritage(
        &mut self,
        data: &mut AsyncExecutionData,
        superclass: Value,
    ) -> Result<Step, Error> {
        self.prepare_async_class(data, Some(superclass))
    }

    fn prepare_async_class(
        &mut self,
        data: &mut AsyncExecutionData,
        superclass: Option<Value>,
    ) -> Result<Step, Error> {
        data.class_definition = Some(self.prepare_class_definition(
            &syntax(data),
            is_default(data).then(|| "default".into()),
            superclass,
        )?);
        self.advance_async_class(data, 0)
    }

    fn advance_async_class(
        &mut self,
        data: &mut AsyncExecutionData,
        first: usize,
    ) -> Result<Step, Error> {
        let class = syntax(data);
        for (element, definition) in class.elements.iter().enumerate().skip(first) {
            self.tick(definition.span())?;
            if matches!(definition.name(), Some(PropertyName::Computed(key)) if expression::has_supported_await(key))
            {
                return self.start_async_expression(
                    data,
                    ExprRoot::ClassElement { element },
                    AsyncAwaitAction::ClassElement { element },
                );
            }
            let name = self.class_element_name(definition)?;
            self.define_class_element(
                data.class_definition
                    .as_mut()
                    .expect("active class definition"),
                &class,
                definition,
                name,
            )?;
        }
        let context = data.class_context.take().expect("class name environment");
        let state = data
            .class_definition
            .take()
            .expect("active class definition");
        let value = self.finish_class_definition(&class, context, state)?;
        if is_default(data) {
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

    pub(super) fn finish_async_class_key(
        &mut self,
        data: &mut AsyncExecutionData,
        element: usize,
        value: Value,
    ) -> Result<Step, Error> {
        let class = syntax(data);
        let definition = &class.elements[element];
        let key = self.property_key(value, definition.span())?;
        self.define_class_element(
            data.class_definition
                .as_mut()
                .expect("active class definition"),
            &class,
            definition,
            Some(super::super::class_field::ClassFieldName::Public(key)),
        )?;
        self.advance_async_class(data, element + 1)
    }
}
