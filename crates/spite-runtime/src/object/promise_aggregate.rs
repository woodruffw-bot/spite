//! Private ordered Promise aggregation results and shared completion count.

use super::{Budget, Error, Objects};
use crate::{
    Value,
    function::{Callable, PromiseAggregateKind, PromiseCapability},
};
use spite_heap::{Handle, Trace};

#[derive(Debug)]
struct Element {
    value: Value,
    called: bool,
}

#[derive(Debug)]
pub(super) struct PromiseAggregateData {
    values: Vec<Element>,
    kind: PromiseAggregateKind,
    remaining: usize,
    capability: PromiseCapability,
}

impl PromiseAggregateData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.values
            .iter()
            .map(|entry| &entry.value)
            .chain(self.capability.values())
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(None).chain(self.values().flat_map(Trace::trace))
    }
}

impl Objects {
    pub(crate) fn create_promise_aggregate_state(
        &mut self,
        capability: PromiseCapability,
        kind: PromiseAggregateKind,
    ) -> Result<Handle, Error> {
        let state = self.create(None)?;
        self.object_mut(&state)?.promise_aggregate = Some(Box::new(PromiseAggregateData {
            values: Vec::new(),
            kind,
            remaining: 1,
            capability,
        }));
        Ok(state)
    }

    pub(crate) fn append_promise_aggregate_slot(
        &mut self,
        state: &Handle,
        budget: &mut Budget,
    ) -> Result<usize, Error> {
        let data = self
            .object_mut(state)?
            .promise_aggregate
            .as_mut()
            .ok_or(Error::WrongKind)?;
        budget.charge(1)?;
        if data.values.len() == data.values.capacity() {
            budget.charge(data.values.len())?;
        }
        data.values
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        let index = data.values.len();
        data.values.push(Element {
            value: Value::Undefined,
            called: false,
        });
        Ok(index)
    }

    pub(crate) fn create_promise_aggregate_element(
        &mut self,
        prototype: &Handle,
        state: &Handle,
        index: usize,
        fulfilled: bool,
    ) -> Result<Handle, Error> {
        self.inspect(state)?;
        let function = self.create(Some(prototype))?;
        self.object_mut(&function)?.callable = Some(Callable::PromiseAggregateElement {
            state: state.clone(),
            index,
            fulfilled,
        });
        Ok(function)
    }

    pub(crate) fn increment_promise_aggregate_remaining(
        &mut self,
        state: &Handle,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        budget.charge(1)?;
        let data = self
            .object_mut(state)?
            .promise_aggregate
            .as_mut()
            .ok_or(Error::WrongKind)?;
        data.remaining = data
            .remaining
            .checked_add(1)
            .ok_or(spite_heap::Error::Capacity)?;
        Ok(())
    }

    pub(crate) fn begin_promise_aggregate_element(
        &mut self,
        function: &Handle,
        budget: &mut Budget,
    ) -> Result<Option<(Handle, usize, PromiseAggregateKind)>, Error> {
        budget.charge(2)?;
        let Some(Callable::PromiseAggregateElement { state, index, .. }) =
            self.inspect(function)?.callable.as_ref()
        else {
            return Err(Error::WrongKind);
        };
        let (state, index) = (state.clone(), *index);
        let data = self
            .object_mut(&state)?
            .promise_aggregate
            .as_mut()
            .ok_or(Error::WrongKind)?;
        let entry = data.values.get_mut(index).ok_or(Error::WrongKind)?;
        if entry.called {
            return Ok(None);
        }
        // Fulfill and reject closures share this guard for their input index.
        entry.called = true;
        Ok(Some((state, index, data.kind)))
    }

    pub(crate) fn store_promise_aggregate_element(
        &mut self,
        state: &Handle,
        index: usize,
        value: Value,
        budget: &mut Budget,
    ) -> Result<Option<(Value, usize)>, Error> {
        if let Value::Object(object) = &value {
            self.inspect(object)?;
        }
        budget.value(&value)?;
        let data = self
            .object_mut(state)?
            .promise_aggregate
            .as_mut()
            .ok_or(Error::WrongKind)?;
        data.values.get_mut(index).ok_or(Error::WrongKind)?.value = value;
        self.finish_promise_aggregate_remaining(state, budget)
    }

    pub(crate) fn finish_promise_aggregate_remaining(
        &mut self,
        state: &Handle,
        budget: &mut Budget,
    ) -> Result<Option<(Value, usize)>, Error> {
        budget.charge(1)?;
        let data = self
            .object_mut(state)?
            .promise_aggregate
            .as_mut()
            .ok_or(Error::WrongKind)?;
        data.remaining = data.remaining.checked_sub(1).ok_or(Error::WrongKind)?;
        if data.remaining == 0 {
            budget.value(&data.capability.resolve)?;
            Ok(Some((data.capability.resolve.clone(), data.values.len())))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn promise_aggregate_value(
        &self,
        state: &Handle,
        index: usize,
        budget: &mut Budget,
    ) -> Result<Value, Error> {
        let data = self
            .inspect(state)?
            .promise_aggregate
            .as_ref()
            .ok_or(Error::WrongKind)?;
        let value = data.values.get(index).ok_or(Error::WrongKind)?;
        budget.value(&value.value)?;
        Ok(value.value.clone())
    }
}
