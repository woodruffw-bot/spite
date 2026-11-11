//! Opt-in host resource quotas.

/// Optional host resource quotas. All default to `None` (no quota).
/// They do not alter ECMAScript exceptions or disable platform safety checks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Limits {
    /// Optional maximum encoded bytes in Script, eval, or dynamic Function source.
    /// Scalars use UTF-8 lengths; each lone surrogate occupies three bytes.
    pub max_source_bytes: Option<usize>,
    /// Optional maximum work per Script or host job, including bindings and arithmetic.
    /// Defaults to `None`, which disables the execution work limit.
    /// Realm initialization is outside this per-Script allowance.
    pub max_steps: Option<usize>,
    /// Optional maximum code units in any produced string.
    pub max_string_units: Option<usize>,
    /// Optional maximum magnitude bits in a produced BigInt.
    pub max_bigint_bits: Option<usize>,
    /// Optional maximum values in a call argument list, including apply and bound arguments.
    pub max_arguments: Option<usize>,
    /// Optional maximum heap slots shared by objects and lexical environments.
    pub max_heap_entries: Option<usize>,
    /// Optional maximum own properties in each ordinary object.
    pub max_properties: Option<usize>,
}
