//! Encoded source ranges.

/// A half-open range of encoded byte offsets in source text.
///
/// Scalar code points use UTF-8 lengths. In dynamically compiled UTF-16 source,
/// each unpaired surrogate occupies three bytes, preserving ordinary UTF-8 offsets.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Span {
    /// Inclusive start offset.
    pub start: usize,
    /// Exclusive end offset.
    pub end: usize,
}

impl Span {
    /// Creates a source range.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
