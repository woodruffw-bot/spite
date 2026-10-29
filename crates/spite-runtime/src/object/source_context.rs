//! Owned function scopes retained between source execution resumes.

use crate::environment::EnvironmentHandle;

#[derive(Debug)]
pub(crate) struct SourceExecutionContext {
    pub scopes: Vec<EnvironmentHandle>,
    pub variable: Option<EnvironmentHandle>,
    pub strict: bool,
    pub module: Option<spite_heap::Handle>,
}

impl SourceExecutionContext {
    pub(crate) fn environments(&self) -> impl Iterator<Item = &EnvironmentHandle> {
        self.scopes.iter().chain(self.variable.iter())
    }
}
