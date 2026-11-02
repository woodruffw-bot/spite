//! Owned Realm data, separate from the Agent's heap, jobs and execution context (9.3).

use crate::{ObjectHandle, TimeZone, function::Intrinsics};
use spite_core::JsString;
use spite_parser::ast::TemplateElement;
use std::{collections::BTreeSet, rc::Rc};

#[derive(Debug, Default)]
pub(crate) struct RealmRecord {
    pub global_object: Option<ObjectHandle>,
    pub intrinsics: Option<Intrinsics>,
    pub template_map: Vec<(Rc<[TemplateElement]>, ObjectHandle)>,
    pub unsupported_host_globals: BTreeSet<String>,
    pub supported_import_attributes: BTreeSet<JsString>,
    pub time_zone: Option<TimeZone>,
}
