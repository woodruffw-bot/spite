//! Private ordered Promise.all results and shared completion count.

use super::{Budget, Error, Objects};
use crate::{
    Value,
    function::{Callable, PromiseCapability},
};
use spite_heap::{Handle, Trace};

#[derive(Debug)]
pub(super) struct PromiseAllData {
    values: Vec<Value>,
    remaining: usize,
    capability: PromiseCapability,
}

impl PromiseAllData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.values.iter().chain(self.capability.values())
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(None).chain(self.values().flat_map(Trace::trace))
    }
}

impl Objects {
    pub(crate) fn create_promise_all_state(
        &mut self,
        capability: PromiseCapability,
    ) -> Result<Handle, Error> {
        let state = self.create(None)?;
        self.object_mut(&state)?.promise_all = Some(Box::new(PromiseAllData {
            values: Vec::new(),
            remaining: 1,
            capability,
        }));
        Ok(state)
    }

    pub(crate) fn append_promise_all_slot(
        &mut self,
        state: &Handle,
        budget: &mut Budget,
    ) -> Result<usize, Error> {
        let data = self
            .object_mut(state)?
            .promise_all
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
        data.values.push(Value::Undefined);
        Ok(index)
    }

    pub(crate) fn create_promise_all_resolve_element(
        &mut self,
        prototype: &Handle,
        state: &Handle,
        index: usize,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        budget.charge(1)?;
        let data = self
            .object_mut(state)?
            .promise_all
            .as_mut()
            .ok_or(Error::WrongKind)?;
        let remaining = data
            .remaining
            .checked_add(1)
            .ok_or(spite_heap::Error::Capacity)?;
        let function = self.create(Some(prototype))?;
        self.object_mut(&function)?.callable = Some(Callable::PromiseAllResolveElement {
            state: state.clone(),
            index,
            called: false,
        });
        // No user code runs between creation and incrementing the shared count.
        self.object_mut(state)?
            .promise_all
            .as_mut()
            .expect("private all state")
            .remaining = remaining;
        Ok(function)
    }

    pub(crate) fn fulfill_promise_all_element(
        &mut self,
        function: &Handle,
        value: Value,
        budget: &mut Budget,
    ) -> Result<Option<(Handle, Value, usize)>, Error> {
        budget.charge(1)?;
        let Some(Callable::PromiseAllResolveElement {
            state,
            index,
            called,
        }) = self.object_mut(function)?.callable.as_mut()
        else {
            return Err(Error::WrongKind);
        };
        if *called {
            return Ok(None);
        }
        // Set AlreadyCalled before touching the shared result or calling resolve.
        *called = true;
        let (state, index) = (state.clone(), *index);
        if let Value::Object(object) = &value {
            self.inspect(object)?;
        }
        budget.value(&value)?;
        let data = self
            .object_mut(&state)?
            .promise_all
            .as_mut()
            .ok_or(Error::WrongKind)?;
        *data.values.get_mut(index).ok_or(Error::WrongKind)? = value;
        self.finish_promise_all_remaining(&state, budget)
            .map(|result| result.map(|(resolve, len)| (state, resolve, len)))
    }

    pub(crate) fn finish_promise_all_remaining(
        &mut self,
        state: &Handle,
        budget: &mut Budget,
    ) -> Result<Option<(Value, usize)>, Error> {
        budget.charge(1)?;
        let data = self
            .object_mut(state)?
            .promise_all
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

    pub(crate) fn promise_all_value(
        &self,
        state: &Handle,
        index: usize,
        budget: &mut Budget,
    ) -> Result<Value, Error> {
        let data = self
            .inspect(state)?
            .promise_all
            .as_ref()
            .ok_or(Error::WrongKind)?;
        let value = data.values.get(index).ok_or(Error::WrongKind)?;
        budget.value(value)?;
        Ok(value.clone())
    }
}
