//! Per-record import.meta with the default host hooks (13.3.12).

use super::*;

impl Realm {
    pub(crate) fn import_meta_value(&mut self, span: Span) -> Result<Value, Error> {
        let record = self
            .current_module_record(span)?
            .expect("Module-only expression retains its defining module environment");
        self.object_work(span, |objects, budget| {
            objects.module_import_meta(&record, budget)
        })
    }

    pub(crate) fn current_module_record(
        &mut self,
        span: Span,
    ) -> Result<Option<ObjectHandle>, Error> {
        self.object_work(span, |_, budget| budget.charge(1))?;
        Ok(self.active_module.clone())
    }
}

#[cfg(test)]
mod tests;
