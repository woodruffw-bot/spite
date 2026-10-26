//! Source generator state and retained call environments (27.5.3).

use super::{Budget, Error, Objects, source_context::SourceExecutionContext};
use crate::environment::EnvironmentHandle;
use spite_core::Span;
use spite_heap::Handle;
use spite_parser::ast::FunctionBody;

#[derive(Debug)]
pub(crate) struct GeneratorStart {
    pub body: FunctionBody,
    pub callee: Handle,
    pub context: SourceExecutionContext,
    pub root: EnvironmentHandle,
    pub span: Span,
}

impl GeneratorStart {
    fn environments(&self) -> impl Iterator<Item = &EnvironmentHandle> {
        self.context
            .environments()
            .chain(std::iter::once(&self.root))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeneratorStatus {
    SuspendedStart,
    SuspendedYield,
    Executing,
    Completed,
}

#[derive(Debug)]
pub(crate) enum GeneratorData {
    SuspendedStart(GeneratorStart),
    SuspendedYield(Box<super::async_execution::AsyncExecutionData>),
    Executing,
    Completed,
}

impl GeneratorData {
    pub(super) fn execution(&self) -> Option<&super::async_execution::AsyncExecutionData> {
        match self {
            Self::SuspendedYield(data) => Some(data),
            Self::SuspendedStart(_) | Self::Executing | Self::Completed => None,
        }
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        let start = match self {
            Self::SuspendedStart(start) => Some(start),
            Self::SuspendedYield(_) | Self::Executing | Self::Completed => None,
        };
        start
            .into_iter()
            .flat_map(|start| {
                std::iter::once(Some(&start.callee))
                    .chain(start.environments().map(|environment| Some(&environment.0)))
            })
            .chain(self.execution().into_iter().flat_map(|data| data.trace()))
    }
}

impl Objects {
    pub(crate) fn create_generator(
        &mut self,
        prototype: &Handle,
        start: GeneratorStart,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        budget.charge(1)?;
        self.inspect(&start.callee)?;
        for environment in start.environments() {
            self.environment(environment)?;
        }
        let object = self.create(Some(prototype))?;
        self.object_mut(&object)?.generator = Some(Box::new(GeneratorData::SuspendedStart(start)));
        Ok(object)
    }

    pub(crate) fn generator_status(
        &self,
        generator: &Handle,
    ) -> Result<Option<GeneratorStatus>, Error> {
        match self.inspect(generator)?.generator.as_deref() {
            Some(GeneratorData::SuspendedStart(_)) => Ok(Some(GeneratorStatus::SuspendedStart)),
            Some(GeneratorData::SuspendedYield(_)) => Ok(Some(GeneratorStatus::SuspendedYield)),
            Some(GeneratorData::Executing) => Ok(Some(GeneratorStatus::Executing)),
            Some(GeneratorData::Completed) => Ok(Some(GeneratorStatus::Completed)),
            None => Ok(None),
        }
    }

    pub(crate) fn take_generator_start(
        &mut self,
        generator: &Handle,
    ) -> Result<GeneratorStart, Error> {
        let data = self
            .object_mut(generator)?
            .generator
            .as_deref_mut()
            .ok_or(Error::WrongKind)?;
        if !matches!(data, GeneratorData::SuspendedStart(_)) {
            return Err(Error::WrongKind);
        }
        let GeneratorData::SuspendedStart(start) =
            std::mem::replace(data, GeneratorData::Executing)
        else {
            unreachable!("validated suspended generator")
        };
        Ok(start)
    }

    pub(crate) fn store_generator_execution(
        &mut self,
        generator: &Handle,
        data: Box<super::async_execution::AsyncExecutionData>,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        debug_assert!(data.function.generator().is_some());
        self.validate_source_execution_data(&data, budget)?;
        let slot = self
            .object_mut(generator)?
            .generator
            .as_deref_mut()
            .ok_or(Error::WrongKind)?;
        if !matches!(slot, GeneratorData::Executing) {
            return Err(Error::WrongKind);
        }
        *slot = GeneratorData::SuspendedYield(data);
        Ok(())
    }

    pub(crate) fn take_generator_execution(
        &mut self,
        generator: &Handle,
    ) -> Result<Box<super::async_execution::AsyncExecutionData>, Error> {
        let slot = self
            .object_mut(generator)?
            .generator
            .as_deref_mut()
            .ok_or(Error::WrongKind)?;
        if !matches!(slot, GeneratorData::SuspendedYield(_)) {
            return Err(Error::WrongKind);
        }
        let GeneratorData::SuspendedYield(data) = std::mem::replace(slot, GeneratorData::Executing)
        else {
            unreachable!("suspended generator execution")
        };
        Ok(data)
    }

    pub(crate) fn complete_generator(&mut self, generator: &Handle) -> Result<(), Error> {
        let data = self
            .object_mut(generator)?
            .generator
            .as_deref_mut()
            .ok_or(Error::WrongKind)?;
        *data = GeneratorData::Completed;
        Ok(())
    }
}
