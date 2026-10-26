//! Per-record import.meta with the default host hooks (13.3.12).

use super::*;

impl Realm {
    pub(crate) fn import_meta_value(&mut self, span: Span) -> Result<Value, Error> {
        // Source functions retain their defining environment. Looking through
        // those outer records selects their module even under a foreign caller,
        // including class initializers and resumed async/generator bodies.
        let mut next = self.scopes.last().cloned();
        while let Some(environment) = next {
            let (module, outer) = self.object_work(span, |objects, budget| {
                budget.charge(1)?;
                let environment = objects.environment(&environment)?;
                let module = match &environment.this {
                    Some(ThisBinding::Module(record)) => Some(record.clone()),
                    _ => None,
                };
                Ok((module, environment.outer.clone()))
            })?;
            if let Some(module) = module {
                return self.object_work(span, |objects, budget| {
                    objects.module_import_meta(&module, budget)
                });
            }
            next = outer;
        }
        unreachable!("Module-only expression retains its defining module environment")
    }
}

#[cfg(test)]
mod tests;
