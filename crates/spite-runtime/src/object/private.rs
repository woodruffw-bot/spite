//! Own private element storage; independent of properties and extensibility (7.3.27–31).

use super::{Budget, Error, GetAction, Objects, SetAction};
use crate::{
    Value,
    private::{PrivateMethod, PrivateMethodKind, PrivateName},
};
use spite_heap::{Handle, Trace};

#[derive(Debug)]
pub(super) enum PrivateElement {
    Field(Value),
    Method(PrivateMethodKind),
}

impl PrivateElement {
    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        let edges = match self {
            Self::Field(value) => [value.trace().next().flatten(), None],
            Self::Method(PrivateMethodKind::Method(method)) => [Some(method), None],
            Self::Method(PrivateMethodKind::Accessor { get, set }) => [get.as_ref(), set.as_ref()],
        };
        edges.into_iter()
    }
}

impl Objects {
    fn private_element_index(
        &self,
        object: &Handle,
        name: &PrivateName,
        budget: &mut Budget,
    ) -> Result<Option<usize>, Error> {
        let fields = &self.inspect(object)?.private_elements;
        budget.charge(fields.len().checked_add(1).ok_or(Error::WorkLimit)?)?;
        Ok(fields.iter().position(|(key, _)| key == name))
    }

    pub(crate) fn private_has(
        &self,
        object: &Handle,
        name: &PrivateName,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        Ok(self.private_element_index(object, name, budget)?.is_some())
    }

    pub(crate) fn private_get(
        &self,
        object: &Handle,
        name: &PrivateName,
        budget: &mut Budget,
    ) -> Result<Option<GetAction>, Error> {
        let Some(index) = self.private_element_index(object, name, budget)? else {
            return Ok(None);
        };
        Ok(match &self.inspect(object)?.private_elements[index].1 {
            PrivateElement::Field(value) => {
                budget.value(value)?;
                Some(GetAction::Value(value.clone()))
            }
            PrivateElement::Method(PrivateMethodKind::Method(method)) => {
                budget.charge(1)?;
                Some(GetAction::Value(Value::Object(method.clone())))
            }
            PrivateElement::Method(PrivateMethodKind::Accessor { get, .. }) => {
                budget.charge(1)?;
                get.clone().map(GetAction::Call)
            }
        })
    }

    pub(crate) fn private_field_add(
        &mut self,
        object: &Handle,
        name: PrivateName,
        value: Value,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        if self.private_element_index(object, &name, budget)?.is_some() {
            return Ok(false);
        }
        budget.value(&value)?;
        if let Value::Object(handle) = &value {
            self.inspect(handle)?;
        }
        let fields = &mut self.object_mut(object)?.private_elements;
        fields
            .try_reserve(1)
            .map_err(|_| Error::Heap(spite_heap::Error::Capacity))?;
        fields.push((name, PrivateElement::Field(value)));
        Ok(true)
    }

    pub(crate) fn private_set(
        &mut self,
        object: &Handle,
        name: &PrivateName,
        value: &Value,
        budget: &mut Budget,
    ) -> Result<SetAction, Error> {
        let Some(index) = self.private_element_index(object, name, budget)? else {
            return Ok(SetAction::Done(false));
        };
        match &self.inspect(object)?.private_elements[index].1 {
            PrivateElement::Method(PrivateMethodKind::Method(_)) => Ok(SetAction::Done(false)),
            PrivateElement::Method(PrivateMethodKind::Accessor { set, .. }) => {
                budget.charge(1)?;
                Ok(set.clone().map_or(SetAction::Done(false), SetAction::Call))
            }
            PrivateElement::Field(_) => {
                budget.value(value)?;
                if let Value::Object(handle) = value {
                    self.inspect(handle)?;
                }
                self.object_mut(object)?.private_elements[index].1 =
                    PrivateElement::Field(value.clone());
                Ok(SetAction::Done(true))
            }
        }
    }

    pub(super) fn check_private_method(
        &self,
        kind: &PrivateMethodKind,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(1)?;
        for handle in kind.trace().flatten() {
            budget.charge(1)?;
            if !self.inspect(handle)?.is_callable() {
                return Err(Error::NotCallable);
            }
        }
        Ok(())
    }

    pub(crate) fn private_method_add(
        &mut self,
        object: &Handle,
        method: &PrivateMethod,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        if self
            .private_element_index(object, &method.name, budget)?
            .is_some()
        {
            return Ok(false);
        }
        self.check_private_method(&method.kind, budget)?;
        let elements = &mut self.object_mut(object)?.private_elements;
        elements
            .try_reserve(1)
            .map_err(|_| Error::Heap(spite_heap::Error::Capacity))?;
        elements.push((
            method.name.clone(),
            PrivateElement::Method(method.kind.clone()),
        ));
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unlimited() -> Budget {
        Budget::with_work_limit(None)
    }

    #[test]
    fn private_fields_are_own_and_ignore_property_capacity_and_extensibility() {
        let mut objects = Objects::new(5, 0);
        let parent = objects.create(None).unwrap();
        let child = objects.create(Some(&parent)).unwrap();
        let name = PrivateName::new("#x");
        let other = PrivateName::new("#x");
        objects.prevent_extensions(&parent).unwrap();
        assert!(
            objects
                .private_field_add(&parent, name.clone(), Value::Number(7.0), &mut unlimited())
                .unwrap()
        );
        assert!(
            !objects
                .private_has(&child, &name, &mut unlimited())
                .unwrap()
        );
        assert!(
            !objects
                .private_has(&parent, &other, &mut unlimited())
                .unwrap()
        );
        assert!(
            !objects
                .private_field_add(&parent, name.clone(), Value::Number(8.0), &mut unlimited())
                .unwrap()
        );
        assert_eq!(
            objects
                .private_set(&parent, &name, &Value::Number(9.0), &mut unlimited())
                .unwrap(),
            SetAction::Done(true)
        );
        assert_eq!(
            objects
                .private_get(&parent, &name, &mut unlimited())
                .unwrap(),
            Some(GetAction::Value(Value::Number(9.0)))
        );
        assert_eq!(objects.inspect(&parent).unwrap().property_count(), 0);
    }

    #[test]
    fn invalid_edges_and_work_fail_before_private_mutation() {
        let mut objects = Objects::new(8, 0);
        let object = objects.create(None).unwrap();
        let name = PrivateName::new("#x");
        objects
            .private_field_add(&object, name.clone(), Value::Number(7.0), &mut unlimited())
            .unwrap();
        let stale = objects.create(None).unwrap();
        let foreign = Objects::new(1, 0).create(None).unwrap();
        objects.collect([&object], 100).unwrap();
        let environment = objects
            .create_environment(None, Default::default(), &mut unlimited())
            .unwrap();
        for (bad, expected) in [
            (foreign, Error::Heap(spite_heap::Error::ForeignHandle)),
            (stale, Error::Heap(spite_heap::Error::StaleHandle)),
            (environment.0, Error::WrongKind),
        ] {
            assert_eq!(
                objects.private_set(
                    &object,
                    &name,
                    &Value::Object(bad.clone()),
                    &mut unlimited()
                ),
                Err(expected)
            );
            assert_eq!(
                objects.private_field_add(
                    &object,
                    PrivateName::new("#new"),
                    Value::Object(bad.clone()),
                    &mut unlimited()
                ),
                Err(expected)
            );
            assert_eq!(
                objects.private_has(&bad, &name, &mut unlimited()),
                Err(expected)
            );
            assert_eq!(
                objects
                    .private_get(&object, &name, &mut unlimited())
                    .unwrap(),
                Some(GetAction::Value(Value::Number(7.0)))
            );
            assert_eq!(objects.inspect(&object).unwrap().private_elements.len(), 1);
        }
        assert_eq!(
            objects.private_set(&object, &name, &Value::Number(9.0), &mut Budget::new(2)),
            Err(Error::WorkLimit)
        );
        assert_eq!(
            objects.private_field_add(
                &object,
                PrivateName::new("#new"),
                Value::Number(9.0),
                &mut Budget::new(2)
            ),
            Err(Error::WorkLimit)
        );
        assert_eq!(
            objects.private_get(&object, &name, &mut Budget::new(2)),
            Err(Error::WorkLimit)
        );
        assert_eq!(
            objects
                .private_get(&object, &name, &mut unlimited())
                .unwrap(),
            Some(GetAction::Value(Value::Number(7.0)))
        );
    }

    #[test]
    fn collection_traces_private_values_and_releases_replaced_values_and_cycles() {
        let mut objects = Objects::new(5, 0);
        let object = objects.create(None).unwrap();
        let child = objects.create(None).unwrap();
        let name = PrivateName::new("#x");
        objects
            .private_field_add(
                &object,
                name.clone(),
                Value::Object(child.clone()),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .private_field_add(
                &child,
                PrivateName::new("#back"),
                Value::Object(object.clone()),
                &mut unlimited(),
            )
            .unwrap();
        assert_eq!(objects.collect([&object], 100).unwrap().live, 2);
        objects
            .private_set(&object, &name, &Value::Undefined, &mut unlimited())
            .unwrap();
        assert_eq!(objects.collect([&object], 100).unwrap().reclaimed, 1);
        assert!(objects.inspect(&child).is_err());
        assert_eq!(objects.collect([], 100).unwrap().reclaimed, 1);
    }
}
