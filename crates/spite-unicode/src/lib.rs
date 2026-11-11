//! Pinned Unicode properties, casing, normalization and ECMAScript character rules.

mod case;
mod case_data;
mod characters;
mod normalization;
mod normalization_data;
mod regexp_canonicalize;
mod unicode;
mod unicode_data;

pub use case::{is_unicode_case_ignorable, is_unicode_cased, unicode_case_mapping};
pub use characters::{is_line_terminator, is_whitespace};
pub use normalization::{
    canonical_combining_class, canonical_composition, hangul_decomposition, unicode_decomposition,
};
pub use regexp_canonicalize::regexp_canonicalize_character;
pub use unicode::{is_identifier_part, is_identifier_start};
pub use unicode_data::UNICODE_VERSION;
