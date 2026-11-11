//! ECMAScript evaluation, objects, Realms and host jobs.

mod agent_control;
mod assignment_pattern;
mod binary_data;
mod binding;
mod completion;
mod conversion;
mod declaration;
mod environment;
mod error;
mod eval;
mod expression;
mod for_in;
mod for_of;
mod function;
mod global;
mod iterator_count;
mod limits;
#[cfg(test)]
mod loop_limit_tests;
mod module;
pub mod object;
mod optional_chain;
mod private;
mod realm;
mod realm_finalization;
mod realm_object;
mod realm_proxy;
mod realm_record;
mod realm_typed_array;
mod reference;
mod resource;
mod shared_buffer;
mod statement;
mod super_property;
#[cfg(test)]
#[path = "../tests/common/realm_entries.rs"]
mod test_support;
mod time_zone;
#[cfg(test)]
mod typed_array_tests;
mod unary;
mod value;
mod with;

pub use agent_control::AgentControl;
pub use error::{Error, ExceptionKind};
pub use function::PromiseJobOutcome;
pub use limits::Limits;
pub use module::{ModuleHandle, ModuleImportRequest};
pub use object::PromiseState;
pub use realm::Realm;
pub use realm_finalization::FinalizationCleanupOutcome;
pub use realm_object::RootedValue;
pub use shared_buffer::SharedBuffer;
pub use spite_heap::{Collection, Handle as ObjectHandle};
pub use time_zone::{TimeZone, TimeZoneError};
pub use value::{ConversionError, Value};

use completion::{Completion, CompletionKind};
use environment::BindingState;
use expression::reference_expression;
use realm_object::Hint;
use reference::Reference;
