//! AllImportAttributesSupported and the fixed HostGetSupportedImportAttributes list.

use crate::{Error, ExceptionKind, Limits, Realm};
use spite_core::{JsString, Span};
use spite_parser::ast::ImportAttribute;

impl Realm {
    /// Creates a realm with a fixed set of supported import attribute keys.
    ///
    /// The default constructors support no keys. This host choice cannot change
    /// during execution. Values remain strings passed to the embedding loader;
    /// registering a key does not enable a new module format or a loader.
    pub fn new_with_supported_import_attributes(
        limits: Limits,
        keys: impl IntoIterator<Item = JsString>,
    ) -> Self {
        let mut realm = Self::new(limits);
        realm.supported_import_attributes.extend(keys);
        realm
    }

    pub(crate) fn check_import_attributes(
        &mut self,
        attributes: &[ImportAttribute],
        kind: ExceptionKind,
        span: Span,
    ) -> Result<(), Error> {
        if attributes.is_empty() {
            return Ok(());
        }
        // Account for UTF-16 key lengths and lookup depth before checking support.
        let levels = self.supported_import_attributes.len().max(1).ilog2() as usize + 1;
        let work = attributes.iter().try_fold(1usize, |work, attribute| {
            attribute
                .key
                .len()
                .checked_add(1)
                .and_then(|units| units.checked_mul(levels))
                .and_then(|units| work.checked_add(units))
                .ok_or_else(|| Error::Limit {
                    span,
                    message: "import attribute work capacity exceeded".into(),
                })
        })?;
        self.object_work(span, |_, budget| budget.charge(work))?;
        if attributes
            .iter()
            .any(|attribute| !self.supported_import_attributes.contains(&attribute.key))
        {
            return Err(Self::exception(
                kind,
                span,
                "unsupported import attribute key",
            ));
        }
        Ok(())
    }
}
