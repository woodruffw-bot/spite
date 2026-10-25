//! Traced continuation state for Array.fromAsync (23.1.2.2).

use super::{Budget, Error, IteratorWrapper, Objects};
use crate::{ObjectHandle, Value, function::PromiseCapability};
use spite_heap::{Handle, Trace};

#[derive(Debug)]
pub(crate) enum AsyncArrayInput {
    Iterator(IteratorWrapper),
    ArrayLike { object: ObjectHandle, length: u64 },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AsyncArrayPhase {
    Next,
    Item,
    Map,
    Close,
}

#[derive(Debug)]
pub(crate) struct AsyncArrayData {
    pub capability: PromiseCapability,
    pub array: ObjectHandle,
    pub input: AsyncArrayInput,
    pub index: u64,
    pub mapper: Value,
    pub this_arg: Value,
    pub phase: AsyncArrayPhase,
    pub error: Option<Value>,
}

impl AsyncArrayData {
    pub(super) fn values(&self) -> impl Iterator<Item = &Value> {
        self.capability
            .values()
            .chain([&self.mapper, &self.this_arg])
            .chain(&self.error)
            .chain(match &self.input {
                AsyncArrayInput::Iterator(record) => Some(&record.next),
                AsyncArrayInput::ArrayLike { .. } => None,
            })
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        let source = match &self.input {
            AsyncArrayInput::Iterator(record) => &record.iterator,
            AsyncArrayInput::ArrayLike { object, .. } => object,
        };
        [Some(&self.array), Some(source)]
            .into_iter()
            .chain(self.values().flat_map(Trace::trace))
    }
}

impl Objects {
    pub(crate) fn create_array_from_async(
        &mut self,
        data: AsyncArrayData,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        for value in data.values() {
            budget.value(value)?;
        }
        for handle in data.trace().flatten() {
            self.inspect(handle)?;
        }
        let state = self.create(None)?;
        self.object_mut(&state)?.array_from_async = Some(Box::new(data));
        Ok(state)
    }

    pub(crate) fn array_from_async_mut(
        &mut self,
        state: &Handle,
    ) -> Result<&mut AsyncArrayData, Error> {
        self.object_mut(state)?
            .array_from_async
            .as_deref_mut()
            .ok_or(Error::WrongKind)
    }

    pub(crate) fn take_array_from_async(
        &mut self,
        state: &Handle,
    ) -> Result<Box<AsyncArrayData>, Error> {
        self.object_mut(state)?
            .array_from_async
            .take()
            .ok_or(Error::WrongKind)
    }
}
