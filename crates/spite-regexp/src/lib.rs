//! ECMAScript RegExp validation, UTF-16 matching, escaping and replacement syntax.

mod escape;
mod pattern;
mod pattern_source;
mod program;
mod replacement;

pub use escape::regexp_escape_units;
pub use pattern::{
    RegExpNamedCapture, RegExpNamedReference, RegExpPatternMetadata, parse_regexp_pattern,
    validate_regexp_pattern,
};
pub use pattern_source::regexp_pattern_source_units;
pub use program::{RegExpProgram, RegExpProgramMatch, RegExpProgramResult};
pub use replacement::{ReplacementPart, replacement_parts};
pub use spite_unicode::regexp_canonicalize_character;
