//! Full default case mappings from the versioned Unicode Character Database.

use crate::{
    case_data::{CASE_IGNORABLE, CASED, LOWERCASE, UPPERCASE},
    unicode::contains,
};

/// Returns the full, context-independent Unicode default case mapping as UTF-16.
///
/// Returns `None` for unchanged scalars. Lowercase callers must additionally apply
/// Unicode's Final_Sigma context rule to U+03A3 using the original input text.
pub fn unicode_case_mapping(point: char, uppercase: bool) -> Option<&'static [u16]> {
    let table = if uppercase { UPPERCASE } else { LOWERCASE };
    table
        .binary_search_by_key(&(point as u32), |&(cp, _)| cp)
        .ok()
        .map(|i| table[i].1)
}

/// Returns the pinned Unicode Cased property used in default contextual casing.
pub fn is_unicode_cased(point: char) -> bool {
    contains(CASED, point as u32)
}

/// Returns the pinned Unicode Case_Ignorable property used in contextual casing.
///
/// A scalar may be both Cased and Case_Ignorable. Context scans ignore it first.
pub fn is_unicode_case_ignorable(point: char) -> bool {
    contains(CASE_IGNORABLE, point as u32)
}
