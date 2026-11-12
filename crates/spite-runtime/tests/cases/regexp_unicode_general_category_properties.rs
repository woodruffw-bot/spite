//! All exact general-category aliases over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use test_case::test_case;

use crate::common::assert_true;

const CATEGORY_WITNESSES: &[u32] = &[
    0x0000,   // NUL
    0x000a,   // line feed
    0x0020,   // space
    0x0041,   // Latin capital A
    0x0061,   // Latin small a
    0x004b,   // Latin capital K
    0x006b,   // Latin small k
    0x00b5,   // micro sign
    0x0345,   // combining Greek ypogegrammeni
    0x0378,   // unassigned
    0x03a3,   // Greek capital sigma
    0x03c2,   // Greek final sigma
    0x03c3,   // Greek small sigma
    0x03b9,   // Greek small iota
    0x1f88,   // Greek capital alpha with psili and prosgegrammeni
    0x1f80,   // Greek small alpha with psili and ypogegrammeni
    0x2000,   // en quad
    0x2126,   // ohm sign
    0x212a,   // Kelvin sign
    0xd800,   // lone high surrogate
    0xdc00,   // lone low surrogate
    0xe000,   // private use
    0xfdd0,   // noncharacter
    0x10000,  // Linear B syllable
    0x1d400,  // mathematical bold capital A
    0x1f600,  // emoji
    0x10ffff, // last code point
];

#[test_case(
    &["C", "Other"],
    &["u", "ui", "v", "vi"],
    [
        "110000000100000000011110001", // property
        "001111111011111111100001110", // complement
        "110000000100000000011110001", // property in a class
        "001111111011111111100001110", // complement in a class
        "001111111011111111100001110", // negated property class
        "110000000100000000011110001", // negated complement class
    ];
    "other_all_modes"
)]
#[test_case(
    &["Cc", "cntrl", "Control"],
    &["u", "ui", "v", "vi"],
    [
        "110000000000000000000000000", // property
        "001111111111111111111111111", // complement
        "110000000000000000000000000", // property in a class
        "001111111111111111111111111", // complement in a class
        "001111111111111111111111111", // negated property class
        "110000000000000000000000000", // negated complement class
    ];
    "control_all_modes"
)]
#[test_case(
    &["Cf", "Format"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "format_all_modes"
)]
#[test_case(
    &["Cn", "Unassigned"],
    &["u", "ui", "v", "vi"],
    [
        "000000000100000000000010001", // property
        "111111111011111111111101110", // complement
        "000000000100000000000010001", // property in a class
        "111111111011111111111101110", // complement in a class
        "111111111011111111111101110", // negated property class
        "000000000100000000000010001", // negated complement class
    ];
    "unassigned_all_modes"
)]
#[test_case(
    &["Co", "Private_Use"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000100000", // property
        "111111111111111111111011111", // complement
        "000000000000000000000100000", // property in a class
        "111111111111111111111011111", // complement in a class
        "111111111111111111111011111", // negated property class
        "000000000000000000000100000", // negated complement class
    ];
    "private_use_all_modes"
)]
#[test_case(
    &["Cs", "Surrogate"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000011000000", // property
        "111111111111111111100111111", // complement
        "000000000000000000011000000", // property in a class
        "111111111111111111100111111", // complement in a class
        "111111111111111111100111111", // negated property class
        "000000000000000000011000000", // negated complement class
    ];
    "surrogate_all_modes"
)]
#[test_case(
    &["L", "Letter"],
    &["u", "v"],
    [
        "000111110011111101100001100", // property
        "111000001100000010011110011", // complement
        "000111110011111101100001100", // property in a class
        "111000001100000010011110011", // complement in a class
        "111000001100000010011110011", // negated property class
        "000111110011111101100001100", // negated complement class
    ];
    "letter_case_sensitive"
)]
#[test_case(
    &["L", "Letter"],
    &["ui"],
    [
        "000111111011111101100001100", // property
        "111000001100010010011110011", // complement
        "000111111011111101100001100", // property in a class
        "111000001100010010011110011", // complement in a class
        "111000000100000010011110011", // negated property class
        "000111110011101101100001100", // negated complement class
    ];
    "letter_unicode_ignore_case"
)]
#[test_case(
    &["L", "Letter"],
    &["vi"],
    [
        "000111111011111101100001100", // property
        "111000000100000010011110011", // complement
        "000111111011111101100001100", // property in a class
        "111000000100000010011110011", // complement in a class
        "111000000100000010011110011", // negated property class
        "000111111011111101100001100", // negated complement class
    ];
    "letter_unicode_sets_ignore_case"
)]
#[test_case(
    &["LC", "Cased_Letter"],
    &["u", "v"],
    [
        "000111110011111101100000100", // property
        "111000001100000010011111011", // complement
        "000111110011111101100000100", // property in a class
        "111000001100000010011111011", // complement in a class
        "111000001100000010011111011", // negated property class
        "000111110011111101100000100", // negated complement class
    ];
    "cased_letter_case_sensitive"
)]
#[test_case(
    &["LC", "Cased_Letter"],
    &["ui"],
    [
        "000111111011111101100000100", // property
        "111000001100010010011111011", // complement
        "000111111011111101100000100", // property in a class
        "111000001100010010011111011", // complement in a class
        "111000000100000010011111011", // negated property class
        "000111110011101101100000100", // negated complement class
    ];
    "cased_letter_unicode_ignore_case"
)]
#[test_case(
    &["LC", "Cased_Letter"],
    &["vi"],
    [
        "000111111011111101100000100", // property
        "111000000100000010011111011", // complement
        "000111111011111101100000100", // property in a class
        "111000000100000010011111011", // complement in a class
        "111000000100000010011111011", // negated property class
        "000111111011111101100000100", // negated complement class
    ];
    "cased_letter_unicode_sets_ignore_case"
)]
#[test_case(
    &["Ll", "Lowercase_Letter"],
    &["u", "v"],
    [
        "000010110001110100000000000", // property
        "111101001110001011111111111", // complement
        "000010110001110100000000000", // property in a class
        "111101001110001011111111111", // complement in a class
        "111101001110001011111111111", // negated property class
        "000010110001110100000000000", // negated complement class
    ];
    "lowercase_letter_case_sensitive"
)]
#[test_case(
    &["Ll", "Lowercase_Letter"],
    &["ui"],
    [
        "000111111011111101100000000", // property
        "111111111111111111111111111", // complement
        "000111111011111101100000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111000000100000010011111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "lowercase_letter_unicode_ignore_case"
)]
#[test_case(
    &["Ll", "Lowercase_Letter"],
    &["vi"],
    [
        "000111111011111101100000000", // property
        "111000000100000010011111111", // complement
        "000111111011111101100000000", // property in a class
        "111000000100000010011111111", // complement in a class
        "111000000100000010011111111", // negated property class
        "000111111011111101100000000", // negated complement class
    ];
    "lowercase_letter_unicode_sets_ignore_case"
)]
#[test_case(
    &["Lm", "Modifier_Letter"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "modifier_letter_all_modes"
)]
#[test_case(
    &["Lo", "Other_Letter"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000001000", // property
        "111111111111111111111110111", // complement
        "000000000000000000000001000", // property in a class
        "111111111111111111111110111", // complement in a class
        "111111111111111111111110111", // negated property class
        "000000000000000000000001000", // negated complement class
    ];
    "other_letter_all_modes"
)]
#[test_case(
    &["Lt", "Titlecase_Letter"],
    &["u", "v"],
    [
        "000000000000001000000000000", // property
        "111111111111110111111111111", // complement
        "000000000000001000000000000", // property in a class
        "111111111111110111111111111", // complement in a class
        "111111111111110111111111111", // negated property class
        "000000000000001000000000000", // negated complement class
    ];
    "titlecase_letter_case_sensitive"
)]
#[test_case(
    &["Lt", "Titlecase_Letter"],
    &["ui"],
    [
        "000000000000001100000000000", // property
        "111111111111111111111111111", // complement
        "000000000000001100000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111110011111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "titlecase_letter_unicode_ignore_case"
)]
#[test_case(
    &["Lt", "Titlecase_Letter"],
    &["vi"],
    [
        "000000000000001100000000000", // property
        "111111111111110011111111111", // complement
        "000000000000001100000000000", // property in a class
        "111111111111110011111111111", // complement in a class
        "111111111111110011111111111", // negated property class
        "000000000000001100000000000", // negated complement class
    ];
    "titlecase_letter_unicode_sets_ignore_case"
)]
#[test_case(
    &["Lu", "Uppercase_Letter"],
    &["u", "v"],
    [
        "000101000010000001100000100", // property
        "111010111101111110011111011", // complement
        "000101000010000001100000100", // property in a class
        "111010111101111110011111011", // complement in a class
        "111010111101111110011111011", // negated property class
        "000101000010000001100000100", // negated complement class
    ];
    "uppercase_letter_case_sensitive"
)]
#[test_case(
    &["Lu", "Uppercase_Letter"],
    &["ui"],
    [
        "000111111011110001100000100", // property
        "111111111111111111111111011", // complement
        "000111111011110001100000100", // property in a class
        "111111111111111111111111011", // complement in a class
        "111000000100001110011111011", // negated property class
        "000000000000000000000000100", // negated complement class
    ];
    "uppercase_letter_unicode_ignore_case"
)]
#[test_case(
    &["Lu", "Uppercase_Letter"],
    &["vi"],
    [
        "000111111011110001100000100", // property
        "111000000100001110011111011", // complement
        "000111111011110001100000100", // property in a class
        "111000000100001110011111011", // complement in a class
        "111000000100001110011111011", // negated property class
        "000111111011110001100000100", // negated complement class
    ];
    "uppercase_letter_unicode_sets_ignore_case"
)]
#[test_case(
    &["M", "Mark", "Combining_Mark"],
    &["u", "v"],
    [
        "000000001000000000000000000", // property
        "111111110111111111111111111", // complement
        "000000001000000000000000000", // property in a class
        "111111110111111111111111111", // complement in a class
        "111111110111111111111111111", // negated property class
        "000000001000000000000000000", // negated complement class
    ];
    "mark_case_sensitive"
)]
#[test_case(
    &["M", "Mark", "Combining_Mark"],
    &["ui"],
    [
        "000000001000010000000000000", // property
        "111111111111111111111111111", // complement
        "000000001000010000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111110111101111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "mark_unicode_ignore_case"
)]
#[test_case(
    &["M", "Mark", "Combining_Mark"],
    &["vi"],
    [
        "000000001000010000000000000", // property
        "111111110111101111111111111", // complement
        "000000001000010000000000000", // property in a class
        "111111110111101111111111111", // complement in a class
        "111111110111101111111111111", // negated property class
        "000000001000010000000000000", // negated complement class
    ];
    "mark_unicode_sets_ignore_case"
)]
#[test_case(
    &["Mc", "Spacing_Mark"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "spacing_mark_all_modes"
)]
#[test_case(
    &["Me", "Enclosing_Mark"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "enclosing_mark_all_modes"
)]
#[test_case(
    &["Mn", "Nonspacing_Mark"],
    &["u", "v"],
    [
        "000000001000000000000000000", // property
        "111111110111111111111111111", // complement
        "000000001000000000000000000", // property in a class
        "111111110111111111111111111", // complement in a class
        "111111110111111111111111111", // negated property class
        "000000001000000000000000000", // negated complement class
    ];
    "nonspacing_mark_case_sensitive"
)]
#[test_case(
    &["Mn", "Nonspacing_Mark"],
    &["ui"],
    [
        "000000001000010000000000000", // property
        "111111111111111111111111111", // complement
        "000000001000010000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111110111101111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "nonspacing_mark_unicode_ignore_case"
)]
#[test_case(
    &["Mn", "Nonspacing_Mark"],
    &["vi"],
    [
        "000000001000010000000000000", // property
        "111111110111101111111111111", // complement
        "000000001000010000000000000", // property in a class
        "111111110111101111111111111", // complement in a class
        "111111110111101111111111111", // negated property class
        "000000001000010000000000000", // negated complement class
    ];
    "nonspacing_mark_unicode_sets_ignore_case"
)]
#[test_case(
    &["N", "Number"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "number_all_modes"
)]
#[test_case(
    &["Nd", "digit", "Decimal_Number"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "decimal_number_all_modes"
)]
#[test_case(
    &["Nl", "Letter_Number"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "letter_number_all_modes"
)]
#[test_case(
    &["No", "Other_Number"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "other_number_all_modes"
)]
#[test_case(
    &["P", "punct", "Punctuation"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "punctuation_all_modes"
)]
#[test_case(
    &["Pc", "Connector_Punctuation"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "connector_punctuation_all_modes"
)]
#[test_case(
    &["Pd", "Dash_Punctuation"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "dash_punctuation_all_modes"
)]
#[test_case(
    &["Pe", "Close_Punctuation"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "close_punctuation_all_modes"
)]
#[test_case(
    &["Pf", "Final_Punctuation"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "final_punctuation_all_modes"
)]
#[test_case(
    &["Pi", "Initial_Punctuation"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "initial_punctuation_all_modes"
)]
#[test_case(
    &["Po", "Other_Punctuation"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "other_punctuation_all_modes"
)]
#[test_case(
    &["Ps", "Open_Punctuation"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "open_punctuation_all_modes"
)]
#[test_case(
    &["S", "Symbol"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000010", // property
        "111111111111111111111111101", // complement
        "000000000000000000000000010", // property in a class
        "111111111111111111111111101", // complement in a class
        "111111111111111111111111101", // negated property class
        "000000000000000000000000010", // negated complement class
    ];
    "symbol_all_modes"
)]
#[test_case(
    &["Sc", "Currency_Symbol"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "currency_symbol_all_modes"
)]
#[test_case(
    &["Sk", "Modifier_Symbol"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "modifier_symbol_all_modes"
)]
#[test_case(
    &["Sm", "Math_Symbol"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "math_symbol_all_modes"
)]
#[test_case(
    &["So", "Other_Symbol"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000010", // property
        "111111111111111111111111101", // complement
        "000000000000000000000000010", // property in a class
        "111111111111111111111111101", // complement in a class
        "111111111111111111111111101", // negated property class
        "000000000000000000000000010", // negated complement class
    ];
    "other_symbol_all_modes"
)]
#[test_case(
    &["Z", "Separator"],
    &["u", "ui", "v", "vi"],
    [
        "001000000000000010000000000", // property
        "110111111111111101111111111", // complement
        "001000000000000010000000000", // property in a class
        "110111111111111101111111111", // complement in a class
        "110111111111111101111111111", // negated property class
        "001000000000000010000000000", // negated complement class
    ];
    "separator_all_modes"
)]
#[test_case(
    &["Zl", "Line_Separator"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "line_separator_all_modes"
)]
#[test_case(
    &["Zp", "Paragraph_Separator"],
    &["u", "ui", "v", "vi"],
    [
        "000000000000000000000000000", // property
        "111111111111111111111111111", // complement
        "000000000000000000000000000", // property in a class
        "111111111111111111111111111", // complement in a class
        "111111111111111111111111111", // negated property class
        "000000000000000000000000000", // negated complement class
    ];
    "paragraph_separator_all_modes"
)]
#[test_case(
    &["Zs", "Space_Separator"],
    &["u", "ui", "v", "vi"],
    [
        "001000000000000010000000000", // property
        "110111111111111101111111111", // complement
        "001000000000000010000000000", // property in a class
        "110111111111111101111111111", // complement in a class
        "110111111111111101111111111", // negated property class
        "001000000000000010000000000", // negated complement class
    ];
    "space_separator_all_modes"
)]
fn general_category_membership(aliases: &[&str], flags: &[&str], expected_rows: [&str; 6]) {
    // Each row is a pattern form; each column is a CATEGORY_WITNESSES code point.
    let expected = Value::String(JsString::from(expected_rows.join("|").as_str()));
    for alias in aliases {
        for prefix in ["", "gc=", "General_Category="] {
            let property = format!("{prefix}{alias}");
            let patterns = [
                format!(r"^\p{{{property}}}$"),
                format!(r"^\P{{{property}}}$"),
                format!(r"^[\p{{{property}}}]$"),
                format!(r"^[\P{{{property}}}]$"),
                format!(r"^[^\p{{{property}}}]$"),
                format!(r"^[^\P{{{property}}}]$"),
            ]
            .map(|pattern| JsString::from(pattern.as_str()));

            for mode in flags {
                let program = format!(
                    r#"
                        const patterns = {patterns:?};
                        const points = {CATEGORY_WITNESSES:?};
                        patterns.map(pattern => {{
                            const regexp = new RegExp(pattern, "{mode}");
                            return points.map(point =>
                                regexp.test(String.fromCodePoint(point)) ? "1" : "0"
                            ).join("");
                        }}).join("|")
                    "#
                );
                let result = Realm::default().eval(&program);
                assert_eq!(
                    result,
                    Ok(expected.clone()),
                    "property={property} flags={mode}"
                );
            }
        }
    }
}

#[test]
fn unicode_general_category_unicode18_membership_and_folding_changes() {
    assert_true(
        r#"
            let patterns = ["^\\p{C}$", "^\\P{C}$", "^[\\p{C}]$", "^[\\P{C}]$", "^[^\\p{C}]$",
                    "^[^\\P{C}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894,
                    6896, 8335, 8349, 8350, 8351, 8386, 8387, 8388, 11872, 11873, 11875, 42973, 42978,
                    43851, 43852, 43884, 43885, 67515, 67517, 67519, 69321, 69324, 69327, 69337, 69347,
                    69358, 69360, 69364, 69369, 72458, 73200, 73201, 74863, 74869, 74874, 74879, 75088,
                    75243, 75398, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 118482, 118483, 118484, 118493, 118494,
                    118495, 118513, 118519, 118525, 119079, 119080, 119275, 119285, 119295, 119376, 119400,
                    119425, 120486, 121600, 121614, 121628, 122655, 122657, 122660, 122667, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768,
                    122771, 122773, 122774, 122829, 122854, 122879, 127406, 128729, 128986, 128987, 129009,
                    129016, 129023, 129675, 129676, 129677, 129740, 129757, 129771, 129785, 129786, 178206,
                    249856, 255519, 261183
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{C}$", "^\\P{C}$", "^[\\p{C}]$", "^[\\P{C}]$", "^[^\\p{C}]$",
                    "^[^\\P{C}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894,
                    6896, 8335, 8349, 8350, 8351, 8386, 8387, 8388, 11872, 11873, 11875, 42973, 42978,
                    43851, 43852, 43884, 43885, 67515, 67517, 67519, 69321, 69324, 69327, 69337, 69347,
                    69358, 69360, 69364, 69369, 72458, 73200, 73201, 74863, 74869, 74874, 74879, 75088,
                    75243, 75398, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 118482, 118483, 118484, 118493, 118494,
                    118495, 118513, 118519, 118525, 119079, 119080, 119275, 119285, 119295, 119376, 119400,
                    119425, 120486, 121600, 121614, 121628, 122655, 122657, 122660, 122667, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768,
                    122771, 122773, 122774, 122829, 122854, 122879, 127406, 128729, 128986, 128987, 129009,
                    129016, 129023, 129675, 129676, 129677, 129740, 129757, 129771, 129785, 129786, 178206,
                    249856, 255519, 261183
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{C}$", "^\\P{C}$", "^[\\p{C}]$", "^[\\P{C}]$", "^[^\\p{C}]$",
                    "^[^\\P{C}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894,
                    6896, 8335, 8349, 8350, 8351, 8386, 8387, 8388, 11872, 11873, 11875, 42973, 42978,
                    43851, 43852, 43884, 43885, 67515, 67517, 67519, 69321, 69324, 69327, 69337, 69347,
                    69358, 69360, 69364, 69369, 72458, 73200, 73201, 74863, 74869, 74874, 74879, 75088,
                    75243, 75398, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 118482, 118483, 118484, 118493, 118494,
                    118495, 118513, 118519, 118525, 119079, 119080, 119275, 119285, 119295, 119376, 119400,
                    119425, 120486, 121600, 121614, 121628, 122655, 122657, 122660, 122667, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768,
                    122771, 122773, 122774, 122829, 122854, 122879, 127406, 128729, 128986, 128987, 129009,
                    129016, 129023, 129675, 129676, 129677, 129740, 129757, 129771, 129785, 129786, 178206,
                    249856, 255519, 261183
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{C}$", "^\\P{C}$", "^[\\p{C}]$", "^[\\P{C}]$", "^[^\\p{C}]$",
                    "^[^\\P{C}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894,
                    6896, 8335, 8349, 8350, 8351, 8386, 8387, 8388, 11872, 11873, 11875, 42973, 42978,
                    43851, 43852, 43884, 43885, 67515, 67517, 67519, 69321, 69324, 69327, 69337, 69347,
                    69358, 69360, 69364, 69369, 72458, 73200, 73201, 74863, 74869, 74874, 74879, 75088,
                    75243, 75398, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 118482, 118483, 118484, 118493, 118494,
                    118495, 118513, 118519, 118525, 119079, 119080, 119275, 119285, 119295, 119376, 119400,
                    119425, 120486, 121600, 121614, 121628, 122655, 122657, 122660, 122667, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768,
                    122771, 122773, 122774, 122829, 122854, 122879, 127406, 128729, 128986, 128987, 129009,
                    129016, 129023, 129675, 129676, 129677, 129740, 129757, 129771, 129785, 129786, 178206,
                    249856, 255519, 261183
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cc}$", "^\\P{Cc}$", "^[\\p{Cc}]$", "^[\\P{Cc}]$", "^[^\\p{Cc}]$",
                    "^[^\\P{Cc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cc}$", "^\\P{Cc}$", "^[\\p{Cc}]$", "^[\\P{Cc}]$", "^[^\\p{Cc}]$",
                    "^[^\\P{Cc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cc}$", "^\\P{Cc}$", "^[\\p{Cc}]$", "^[\\P{Cc}]$", "^[^\\p{Cc}]$",
                    "^[^\\P{Cc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cc}$", "^\\P{Cc}$", "^[\\p{Cc}]$", "^[\\P{Cc}]$", "^[^\\p{Cc}]$",
                    "^[^\\P{Cc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cf}$", "^\\P{Cf}$", "^[\\p{Cf}]$", "^[\\P{Cf}]$", "^[^\\p{Cf}]$",
                    "^[^\\P{Cf}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cf}$", "^\\P{Cf}$", "^[\\p{Cf}]$", "^[\\P{Cf}]$", "^[^\\p{Cf}]$",
                    "^[^\\P{Cf}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cf}$", "^\\P{Cf}$", "^[\\p{Cf}]$", "^[\\P{Cf}]$", "^[^\\p{Cf}]$",
                    "^[^\\P{Cf}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cf}$", "^\\P{Cf}$", "^[\\p{Cf}]$", "^[\\P{Cf}]$", "^[^\\p{Cf}]$",
                    "^[^\\P{Cf}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cn}$", "^\\P{Cn}$", "^[\\p{Cn}]$", "^[\\P{Cn}]$", "^[^\\p{Cn}]$",
                    "^[^\\P{Cn}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894,
                    6896, 8335, 8349, 8350, 8351, 8386, 8387, 8388, 11872, 11873, 11875, 42973, 42978,
                    43851, 43852, 43884, 43885, 67515, 67517, 67519, 69321, 69324, 69327, 69337, 69347,
                    69358, 69360, 69364, 69369, 72458, 73200, 73201, 74863, 74869, 74874, 74879, 75088,
                    75243, 75398, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 118482, 118483, 118484, 118493, 118494,
                    118495, 118513, 118519, 118525, 119079, 119080, 119275, 119285, 119295, 119376, 119400,
                    119425, 120486, 121600, 121614, 121628, 122655, 122657, 122660, 122667, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768,
                    122771, 122773, 122774, 122829, 122854, 122879, 127406, 128729, 128986, 128987, 129009,
                    129016, 129023, 129675, 129676, 129677, 129740, 129757, 129771, 129785, 129786, 178206,
                    249856, 255519, 261183
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cn}$", "^\\P{Cn}$", "^[\\p{Cn}]$", "^[\\P{Cn}]$", "^[^\\p{Cn}]$",
                    "^[^\\P{Cn}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894,
                    6896, 8335, 8349, 8350, 8351, 8386, 8387, 8388, 11872, 11873, 11875, 42973, 42978,
                    43851, 43852, 43884, 43885, 67515, 67517, 67519, 69321, 69324, 69327, 69337, 69347,
                    69358, 69360, 69364, 69369, 72458, 73200, 73201, 74863, 74869, 74874, 74879, 75088,
                    75243, 75398, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 118482, 118483, 118484, 118493, 118494,
                    118495, 118513, 118519, 118525, 119079, 119080, 119275, 119285, 119295, 119376, 119400,
                    119425, 120486, 121600, 121614, 121628, 122655, 122657, 122660, 122667, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768,
                    122771, 122773, 122774, 122829, 122854, 122879, 127406, 128729, 128986, 128987, 129009,
                    129016, 129023, 129675, 129676, 129677, 129740, 129757, 129771, 129785, 129786, 178206,
                    249856, 255519, 261183
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cn}$", "^\\P{Cn}$", "^[\\p{Cn}]$", "^[\\P{Cn}]$", "^[^\\p{Cn}]$",
                    "^[^\\P{Cn}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894,
                    6896, 8335, 8349, 8350, 8351, 8386, 8387, 8388, 11872, 11873, 11875, 42973, 42978,
                    43851, 43852, 43884, 43885, 67515, 67517, 67519, 69321, 69324, 69327, 69337, 69347,
                    69358, 69360, 69364, 69369, 72458, 73200, 73201, 74863, 74869, 74874, 74879, 75088,
                    75243, 75398, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 118482, 118483, 118484, 118493, 118494,
                    118495, 118513, 118519, 118525, 119079, 119080, 119275, 119285, 119295, 119376, 119400,
                    119425, 120486, 121600, 121614, 121628, 122655, 122657, 122660, 122667, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768,
                    122771, 122773, 122774, 122829, 122854, 122879, 127406, 128729, 128986, 128987, 129009,
                    129016, 129023, 129675, 129676, 129677, 129740, 129757, 129771, 129785, 129786, 178206,
                    249856, 255519, 261183
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cn}$", "^\\P{Cn}$", "^[\\p{Cn}]$", "^[\\P{Cn}]$", "^[^\\p{Cn}]$",
                    "^[^\\P{Cn}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894,
                    6896, 8335, 8349, 8350, 8351, 8386, 8387, 8388, 11872, 11873, 11875, 42973, 42978,
                    43851, 43852, 43884, 43885, 67515, 67517, 67519, 69321, 69324, 69327, 69337, 69347,
                    69358, 69360, 69364, 69369, 72458, 73200, 73201, 74863, 74869, 74874, 74879, 75088,
                    75243, 75398, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 118482, 118483, 118484, 118493, 118494,
                    118495, 118513, 118519, 118525, 119079, 119080, 119275, 119285, 119295, 119376, 119400,
                    119425, 120486, 121600, 121614, 121628, 122655, 122657, 122660, 122667, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768,
                    122771, 122773, 122774, 122829, 122854, 122879, 127406, 128729, 128986, 128987, 129009,
                    129016, 129023, 129675, 129676, 129677, 129740, 129757, 129771, 129785, 129786, 178206,
                    249856, 255519, 261183
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Co}$", "^\\P{Co}$", "^[\\p{Co}]$", "^[\\P{Co}]$", "^[^\\p{Co}]$",
                    "^[^\\P{Co}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Co}$", "^\\P{Co}$", "^[\\p{Co}]$", "^[\\P{Co}]$", "^[^\\p{Co}]$",
                    "^[^\\P{Co}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Co}$", "^\\P{Co}$", "^[\\p{Co}]$", "^[\\P{Co}]$", "^[^\\p{Co}]$",
                    "^[^\\P{Co}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Co}$", "^\\P{Co}$", "^[\\p{Co}]$", "^[\\P{Co}]$", "^[^\\p{Co}]$",
                    "^[^\\P{Co}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cs}$", "^\\P{Cs}$", "^[\\p{Cs}]$", "^[\\P{Cs}]$", "^[^\\p{Cs}]$",
                    "^[^\\P{Cs}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cs}$", "^\\P{Cs}$", "^[\\p{Cs}]$", "^[\\P{Cs}]$", "^[^\\p{Cs}]$",
                    "^[^\\P{Cs}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cs}$", "^\\P{Cs}$", "^[\\p{Cs}]$", "^[\\P{Cs}]$", "^[^\\p{Cs}]$",
                    "^[^\\P{Cs}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Cs}$", "^\\P{Cs}$", "^[\\p{Cs}]$", "^[\\P{Cs}]$", "^[^\\p{Cs}]$",
                    "^[^\\P{Cs}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{L}$", "^\\P{L}$", "^[\\p{L}]$", "^[\\P{L}]$", "^[^\\p{L}]$",
                    "^[^\\P{L}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 8335, 8349, 8350, 8351, 42973, 42978, 43851,
                    43852, 43884, 43885, 67515, 67517, 67519, 69337, 69347, 69358, 72458, 73201, 101590,
                    101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816, 102841, 102866, 110883,
                    110885, 110888, 110952, 120486, 122655, 122657, 122660, 122667, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768, 122771,
                    122773, 122774, 122829, 122854, 122879, 178206, 249856, 255519, 261183
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{L}$", "^\\P{L}$", "^[\\p{L}]$", "^[\\P{L}]$", "^[^\\p{L}]$",
                    "^[^\\P{L}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 8335, 8349, 8350, 8351, 42973, 42978, 43851,
                    43852, 43884, 43885, 67515, 67517, 67519, 69337, 69347, 69358, 72458, 73201, 101590,
                    101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816, 102841, 102866, 110883,
                    110885, 110888, 110952, 120486, 122655, 122657, 122660, 122667, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768, 122771,
                    122773, 122774, 122829, 122854, 122879, 178206, 249856, 255519, 261183
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{L}$", "^\\P{L}$", "^[\\p{L}]$", "^[\\P{L}]$", "^[^\\p{L}]$",
                    "^[^\\P{L}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 8335, 8349, 8350, 8351, 42973, 42978, 43851,
                    43852, 43884, 43885, 67515, 67517, 67519, 69337, 69347, 69358, 72458, 73201, 101590,
                    101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816, 102841, 102866, 110883,
                    110885, 110888, 110952, 120486, 122655, 122657, 122660, 122667, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768, 122771,
                    122773, 122774, 122829, 122854, 122879, 178206, 249856, 255519, 261183
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{L}$", "^\\P{L}$", "^[\\p{L}]$", "^[\\P{L}]$", "^[^\\p{L}]$",
                    "^[^\\P{L}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 8335, 8349, 8350, 8351, 42973, 42978, 43851,
                    43852, 43884, 43885, 67515, 67517, 67519, 69337, 69347, 69358, 72458, 73201, 101590,
                    101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816, 102841, 102866, 110883,
                    110885, 110888, 110952, 120486, 122655, 122657, 122660, 122667, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122710, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122753, 122768, 122771,
                    122773, 122774, 122829, 122854, 122879, 178206, 249856, 255519, 261183
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000|11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{LC}$", "^\\P{LC}$", "^[\\p{LC}]$", "^[\\P{LC}]$", "^[^\\p{LC}]$",
                    "^[^\\P{LC}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 120486, 122655, 122657,
                    122660, 122667, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705,
                    122706, 122709, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738,
                    122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749,
                    122750, 122751, 122768, 122771, 122773, 122774
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000|000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{LC}$", "^\\P{LC}$", "^[\\p{LC}]$", "^[\\P{LC}]$", "^[^\\p{LC}]$",
                    "^[^\\P{LC}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 120486, 122655, 122657,
                    122660, 122667, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705,
                    122706, 122709, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738,
                    122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749,
                    122750, 122751, 122768, 122771, 122773, 122774
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000|000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{LC}$", "^\\P{LC}$", "^[\\p{LC}]$", "^[\\P{LC}]$", "^[^\\p{LC}]$",
                    "^[^\\P{LC}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 120486, 122655, 122657,
                    122660, 122667, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705,
                    122706, 122709, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738,
                    122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749,
                    122750, 122751, 122768, 122771, 122773, 122774
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000|000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{LC}$", "^\\P{LC}$", "^[\\p{LC}]$", "^[\\P{LC}]$", "^[^\\p{LC}]$",
                    "^[^\\P{LC}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 120486, 122655, 122657,
                    122660, 122667, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705,
                    122706, 122709, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738,
                    122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749,
                    122750, 122751, 122768, 122771, 122773, 122774
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111|000000000000000000000000000000000000000000000000000|000000000000000000000000000000000000000000000000000|111111111111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Ll}$", "^\\P{Ll}$", "^[\\p{Ll}]$", "^[\\P{Ll}]$", "^[^\\p{Ll}]$",
                    "^[^\\P{Ll}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 120486, 122655, 122657,
                    122660, 122667, 122677, 122687, 122688, 122689, 122692, 122695, 122696, 122697, 122698,
                    122699, 122700, 122701, 122702, 122703, 122704, 122705, 122706, 122716, 122727, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122736, 122737, 122738, 122739,
                    122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750,
                    122751, 122768, 122771, 122773, 122774
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "1110011001111111011101011011101110101010111010101010101011111|0001100110000000100010100100010001010101000101010101010100000|1110011001111111011101011011101110101010111010101010101011111|0001100110000000100010100100010001010101000101010101010100000|0001100110000000100010100100010001010101000101010101010100000|1110011001111111011101011011101110101010111010101010101011111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Ll}$", "^\\P{Ll}$", "^[\\p{Ll}]$", "^[\\P{Ll}]$", "^[^\\p{Ll}]$",
                    "^[^\\P{Ll}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 120486, 122655, 122657,
                    122660, 122667, 122677, 122687, 122688, 122689, 122692, 122695, 122696, 122697, 122698,
                    122699, 122700, 122701, 122702, 122703, 122704, 122705, 122706, 122716, 122727, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122736, 122737, 122738, 122739,
                    122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750,
                    122751, 122768, 122771, 122773, 122774
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "1111111111111111111111111111111111111111111111111111111111111|1111111110000000110011110110011001111111100111111111111110010|1111111111111111111111111111111111111111111111111111111111111|1111111110000000110011110110011001111111100111111111111110010|0000000000000000000000000000000000000000000000000000000000000|0000000001111111001100001001100110000000011000000000000001101"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Ll}$", "^\\P{Ll}$", "^[\\p{Ll}]$", "^[\\P{Ll}]$", "^[^\\p{Ll}]$",
                    "^[^\\P{Ll}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 120486, 122655, 122657,
                    122660, 122667, 122677, 122687, 122688, 122689, 122692, 122695, 122696, 122697, 122698,
                    122699, 122700, 122701, 122702, 122703, 122704, 122705, 122706, 122716, 122727, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122736, 122737, 122738, 122739,
                    122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750,
                    122751, 122768, 122771, 122773, 122774
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "1110011001111111011101011011101110101010111010101010101011111|0001100110000000100010100100010001010101000101010101010100000|1110011001111111011101011011101110101010111010101010101011111|0001100110000000100010100100010001010101000101010101010100000|0001100110000000100010100100010001010101000101010101010100000|1110011001111111011101011011101110101010111010101010101011111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Ll}$", "^\\P{Ll}$", "^[\\p{Ll}]$", "^[\\P{Ll}]$", "^[^\\p{Ll}]$",
                    "^[^\\P{Ll}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 120486, 122655, 122657,
                    122660, 122667, 122677, 122687, 122688, 122689, 122692, 122695, 122696, 122697, 122698,
                    122699, 122700, 122701, 122702, 122703, 122704, 122705, 122706, 122716, 122727, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122736, 122737, 122738, 122739,
                    122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750,
                    122751, 122768, 122771, 122773, 122774
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "1111111111111111111111111111111111111111111111111111111111111|0000000000000000000000000000000000000000000000000000000000000|1111111111111111111111111111111111111111111111111111111111111|0000000000000000000000000000000000000000000000000000000000000|0000000000000000000000000000000000000000000000000000000000000|1111111111111111111111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lm}$", "^\\P{Lm}$", "^[\\p{Lm}]$", "^[\\P{Lm}]$", "^[^\\p{Lm}]$",
                    "^[^\\P{Lm}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 8335, 8349, 8350, 8351, 42973, 42978, 43851,
                    43852, 43884, 43885, 67515, 67517, 67519, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122773, 122829, 122854, 122879
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001111111000000111000000000000000000000000000000000111|1110000000111111000111111111111111111111111111111111000|0001111111000000111000000000000000000000000000000000111|1110000000111111000111111111111111111111111111111111000|1110000000111111000111111111111111111111111111111111000|0001111111000000111000000000000000000000000000000000111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lm}$", "^\\P{Lm}$", "^[\\p{Lm}]$", "^[\\P{Lm}]$", "^[^\\p{Lm}]$",
                    "^[^\\P{Lm}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 8335, 8349, 8350, 8351, 42973, 42978, 43851,
                    43852, 43884, 43885, 67515, 67517, 67519, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122773, 122829, 122854, 122879
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001111111000000111000000000000000000000000000000000111|1110000000111111000111111111111111111111111111111111000|0001111111000000111000000000000000000000000000000000111|1110000000111111000111111111111111111111111111111111000|1110000000111111000111111111111111111111111111111111000|0001111111000000111000000000000000000000000000000000111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lm}$", "^\\P{Lm}$", "^[\\p{Lm}]$", "^[\\P{Lm}]$", "^[^\\p{Lm}]$",
                    "^[^\\P{Lm}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 8335, 8349, 8350, 8351, 42973, 42978, 43851,
                    43852, 43884, 43885, 67515, 67517, 67519, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122773, 122829, 122854, 122879
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001111111000000111000000000000000000000000000000000111|1110000000111111000111111111111111111111111111111111000|0001111111000000111000000000000000000000000000000000111|1110000000111111000111111111111111111111111111111111000|1110000000111111000111111111111111111111111111111111000|0001111111000000111000000000000000000000000000000000111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lm}$", "^\\P{Lm}$", "^[\\p{Lm}]$", "^[\\P{Lm}]$", "^[^\\p{Lm}]$",
                    "^[^\\P{Lm}]$"
                ],
                points = [223, 631, 636, 1368, 1419, 1420, 8335, 8349, 8350, 8351, 42973, 42978, 43851,
                    43852, 43884, 43885, 67515, 67517, 67519, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122773, 122829, 122854, 122879
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001111111000000111000000000000000000000000000000000111|1110000000111111000111111111111111111111111111111111000|0001111111000000111000000000000000000000000000000000111|1110000000111111000111111111111111111111111111111111000|1110000000111111000111111111111111111111111111111111000|0001111111000000111000000000000000000000000000000000111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lo}$", "^\\P{Lo}$", "^[\\p{Lo}]$", "^[\\P{Lo}]$", "^[^\\p{Lo}]$",
                    "^[^\\P{Lo}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 69337, 69347, 69358,
                    72458, 73201, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122752, 122753, 122773, 178206, 249856, 255519,
                    261183
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011111111111111111111000000000000000000000000000000001101111|11111111100000000000000000000111111111111111111111111111111110010000|00000000011111111111111111111000000000000000000000000000000001101111|11111111100000000000000000000111111111111111111111111111111110010000|11111111100000000000000000000111111111111111111111111111111110010000|00000000011111111111111111111000000000000000000000000000000001101111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lo}$", "^\\P{Lo}$", "^[\\p{Lo}]$", "^[\\P{Lo}]$", "^[^\\p{Lo}]$",
                    "^[^\\P{Lo}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 69337, 69347, 69358,
                    72458, 73201, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122752, 122753, 122773, 178206, 249856, 255519,
                    261183
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011111111111111111111000000000000000000000000000000001101111|11111111100000000000000000000111111111111111111111111111111110010000|00000000011111111111111111111000000000000000000000000000000001101111|11111111100000000000000000000111111111111111111111111111111110010000|11111111100000000000000000000111111111111111111111111111111110010000|00000000011111111111111111111000000000000000000000000000000001101111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lo}$", "^\\P{Lo}$", "^[\\p{Lo}]$", "^[\\P{Lo}]$", "^[^\\p{Lo}]$",
                    "^[^\\P{Lo}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 69337, 69347, 69358,
                    72458, 73201, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122752, 122753, 122773, 178206, 249856, 255519,
                    261183
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011111111111111111111000000000000000000000000000000001101111|11111111100000000000000000000111111111111111111111111111111110010000|00000000011111111111111111111000000000000000000000000000000001101111|11111111100000000000000000000111111111111111111111111111111110010000|11111111100000000000000000000111111111111111111111111111111110010000|00000000011111111111111111111000000000000000000000000000000001101111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lo}$", "^\\P{Lo}$", "^[\\p{Lo}]$", "^[\\P{Lo}]$", "^[^\\p{Lo}]$",
                    "^[^\\P{Lo}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 69337, 69347, 69358,
                    72458, 73201, 101590, 101592, 101594, 101663, 101664, 101888, 102344, 102801, 102816,
                    102841, 102866, 110883, 110885, 110888, 110952, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122752, 122753, 122773, 178206, 249856, 255519,
                    261183
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011111111111111111111000000000000000000000000000000001101111|11111111100000000000000000000111111111111111111111111111111110010000|00000000011111111111111111111000000000000000000000000000000001101111|11111111100000000000000000000111111111111111111111111111111110010000|11111111100000000000000000000111111111111111111111111111111110010000|00000000011111111111111111111000000000000000000000000000000001101111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lt}$", "^\\P{Lt}$", "^[\\p{Lt}]$", "^[\\P{Lt}]$", "^[^\\p{Lt}]$",
                    "^[^\\P{Lt}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lt}$", "^\\P{Lt}$", "^[\\p{Lt}]$", "^[\\P{Lt}]$", "^[^\\p{Lt}]$",
                    "^[^\\P{Lt}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lt}$", "^\\P{Lt}$", "^[\\p{Lt}]$", "^[\\P{Lt}]$", "^[^\\p{Lt}]$",
                    "^[^\\P{Lt}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lt}$", "^\\P{Lt}$", "^[\\p{Lt}]$", "^[\\P{Lt}]$", "^[^\\p{Lt}]$",
                    "^[^\\P{Lt}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lu}$", "^\\P{Lu}$", "^[\\p{Lu}]$", "^[\\P{Lu}]$", "^[^\\p{Lu}]$",
                    "^[^\\P{Lu}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000110011101010101010101010101010101010100|111001100010101010101010101010101010101011|000110011101010101010101010101010101010100|111001100010101010101010101010101010101011|111001100010101010101010101010101010101011|000110011101010101010101010101010101010100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lu}$", "^\\P{Lu}$", "^[\\p{Lu}]$", "^[\\P{Lu}]$", "^[^\\p{Lu}]$",
                    "^[^\\P{Lu}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lu}$", "^\\P{Lu}$", "^[\\p{Lu}]$", "^[\\P{Lu}]$", "^[^\\p{Lu}]$",
                    "^[^\\P{Lu}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000110011101010101010101010101010101010100|111001100010101010101010101010101010101011|000110011101010101010101010101010101010100|111001100010101010101010101010101010101011|111001100010101010101010101010101010101011|000110011101010101010101010101010101010100"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Lu}$", "^\\P{Lu}$", "^[\\p{Lu}]$", "^[\\P{Lu}]$", "^[^\\p{Lu}]$",
                    "^[^\\P{Lu}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{M}$", "^\\P{M}$", "^[\\p{M}]$", "^[\\P{M}]$", "^[^\\p{M}]$",
                    "^[^\\P{M}]$"
                ],
                points = [223, 631, 636, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894, 6896, 42973,
                    42978, 43851, 43852, 43884, 43885, 69323, 69325, 69327, 69360, 69364, 69369, 73200,
                    119079, 119080, 119376, 119377, 119378, 119387, 119388, 119391, 119424, 119425, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011111111100000011111111111111111000000000000000000000000000000000|11100000000011111100000000000000000111111111111111111111111111111111|00011111111100000011111111111111111000000000000000000000000000000000|11100000000011111100000000000000000111111111111111111111111111111111|11100000000011111100000000000000000111111111111111111111111111111111|00011111111100000011111111111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{M}$", "^\\P{M}$", "^[\\p{M}]$", "^[\\P{M}]$", "^[^\\p{M}]$",
                    "^[^\\P{M}]$"
                ],
                points = [223, 631, 636, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894, 6896, 42973,
                    42978, 43851, 43852, 43884, 43885, 69323, 69325, 69327, 69360, 69364, 69369, 73200,
                    119079, 119080, 119376, 119377, 119378, 119387, 119388, 119391, 119424, 119425, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011111111100000011111111111111111000000000000000000000000000000000|11100000000011111100000000000000000111111111111111111111111111111111|00011111111100000011111111111111111000000000000000000000000000000000|11100000000011111100000000000000000111111111111111111111111111111111|11100000000011111100000000000000000111111111111111111111111111111111|00011111111100000011111111111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{M}$", "^\\P{M}$", "^[\\p{M}]$", "^[\\P{M}]$", "^[^\\p{M}]$",
                    "^[^\\P{M}]$"
                ],
                points = [223, 631, 636, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894, 6896, 42973,
                    42978, 43851, 43852, 43884, 43885, 69323, 69325, 69327, 69360, 69364, 69369, 73200,
                    119079, 119080, 119376, 119377, 119378, 119387, 119388, 119391, 119424, 119425, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011111111100000011111111111111111000000000000000000000000000000000|11100000000011111100000000000000000111111111111111111111111111111111|00011111111100000011111111111111111000000000000000000000000000000000|11100000000011111100000000000000000111111111111111111111111111111111|11100000000011111100000000000000000111111111111111111111111111111111|00011111111100000011111111111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{M}$", "^\\P{M}$", "^[\\p{M}]$", "^[\\P{M}]$", "^[^\\p{M}]$",
                    "^[^\\P{M}]$"
                ],
                points = [223, 631, 636, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894, 6896, 42973,
                    42978, 43851, 43852, 43884, 43885, 69323, 69325, 69327, 69360, 69364, 69369, 73200,
                    119079, 119080, 119376, 119377, 119378, 119387, 119388, 119391, 119424, 119425, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011111111100000011111111111111111000000000000000000000000000000000|11100000000011111100000000000000000111111111111111111111111111111111|00011111111100000011111111111111111000000000000000000000000000000000|11100000000011111100000000000000000111111111111111111111111111111111|11100000000011111100000000000000000111111111111111111111111111111111|00011111111100000011111111111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Mc}$", "^\\P{Mc}$", "^[\\p{Mc}]$", "^[\\P{Mc}]$", "^[^\\p{Mc}]$",
                    "^[^\\P{Mc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 119376, 119377, 119378,
                    119391, 119424, 119425, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702,
                    122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738,
                    122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749,
                    122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000111111000000000000000000000000000000000|111111111000000111111111111111111111111111111111|000000000111111000000000000000000000000000000000|111111111000000111111111111111111111111111111111|111111111000000111111111111111111111111111111111|000000000111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Mc}$", "^\\P{Mc}$", "^[\\p{Mc}]$", "^[\\P{Mc}]$", "^[^\\p{Mc}]$",
                    "^[^\\P{Mc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 119376, 119377, 119378,
                    119391, 119424, 119425, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702,
                    122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738,
                    122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749,
                    122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000111111000000000000000000000000000000000|111111111000000111111111111111111111111111111111|000000000111111000000000000000000000000000000000|111111111000000111111111111111111111111111111111|111111111000000111111111111111111111111111111111|000000000111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Mc}$", "^\\P{Mc}$", "^[\\p{Mc}]$", "^[\\P{Mc}]$", "^[^\\p{Mc}]$",
                    "^[^\\P{Mc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 119376, 119377, 119378,
                    119391, 119424, 119425, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702,
                    122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738,
                    122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749,
                    122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000111111000000000000000000000000000000000|111111111000000111111111111111111111111111111111|000000000111111000000000000000000000000000000000|111111111000000111111111111111111111111111111111|111111111000000111111111111111111111111111111111|000000000111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Mc}$", "^\\P{Mc}$", "^[\\p{Mc}]$", "^[\\P{Mc}]$", "^[^\\p{Mc}]$",
                    "^[^\\P{Mc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 119376, 119377, 119378,
                    119391, 119424, 119425, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702,
                    122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738,
                    122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749,
                    122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000111111000000000000000000000000000000000|111111111000000111111111111111111111111111111111|000000000111111000000000000000000000000000000000|111111111000000111111111111111111111111111111111|111111111000000111111111111111111111111111111111|000000000111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Me}$", "^\\P{Me}$", "^[\\p{Me}]$", "^[\\P{Me}]$", "^[^\\p{Me}]$",
                    "^[^\\P{Me}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Me}$", "^\\P{Me}$", "^[\\p{Me}]$", "^[\\P{Me}]$", "^[^\\p{Me}]$",
                    "^[^\\P{Me}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Me}$", "^\\P{Me}$", "^[\\p{Me}]$", "^[\\P{Me}]$", "^[^\\p{Me}]$",
                    "^[^\\P{Me}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Me}$", "^\\P{Me}$", "^[\\p{Me}]$", "^[\\P{Me}]$", "^[^\\p{Me}]$",
                    "^[^\\P{Me}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Mn}$", "^\\P{Mn}$", "^[\\p{Mn}]$", "^[\\P{Mn}]$", "^[^\\p{Mn}]$",
                    "^[^\\P{Mn}]$"
                ],
                points = [223, 631, 636, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894, 6896, 42973,
                    42978, 43851, 43852, 43884, 43885, 69323, 69325, 69327, 69360, 69364, 69369, 73200,
                    119079, 119080, 119387, 119388, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011111111100000011111111111000000000000000000000000000000000|11100000000011111100000000000111111111111111111111111111111111|00011111111100000011111111111000000000000000000000000000000000|11100000000011111100000000000111111111111111111111111111111111|11100000000011111100000000000111111111111111111111111111111111|00011111111100000011111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Mn}$", "^\\P{Mn}$", "^[\\p{Mn}]$", "^[\\P{Mn}]$", "^[^\\p{Mn}]$",
                    "^[^\\P{Mn}]$"
                ],
                points = [223, 631, 636, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894, 6896, 42973,
                    42978, 43851, 43852, 43884, 43885, 69323, 69325, 69327, 69360, 69364, 69369, 73200,
                    119079, 119080, 119387, 119388, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011111111100000011111111111000000000000000000000000000000000|11100000000011111100000000000111111111111111111111111111111111|00011111111100000011111111111000000000000000000000000000000000|11100000000011111100000000000111111111111111111111111111111111|11100000000011111100000000000111111111111111111111111111111111|00011111111100000011111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Mn}$", "^\\P{Mn}$", "^[\\p{Mn}]$", "^[\\P{Mn}]$", "^[^\\p{Mn}]$",
                    "^[^\\P{Mn}]$"
                ],
                points = [223, 631, 636, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894, 6896, 42973,
                    42978, 43851, 43852, 43884, 43885, 69323, 69325, 69327, 69360, 69364, 69369, 73200,
                    119079, 119080, 119387, 119388, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011111111100000011111111111000000000000000000000000000000000|11100000000011111100000000000111111111111111111111111111111111|00011111111100000011111111111000000000000000000000000000000000|11100000000011111100000000000111111111111111111111111111111111|11100000000011111100000000000111111111111111111111111111111111|00011111111100000011111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Mn}$", "^\\P{Mn}$", "^[\\p{Mn}]$", "^[\\P{Mn}]$", "^[^\\p{Mn}]$",
                    "^[^\\P{Mn}]$"
                ],
                points = [223, 631, 636, 1480, 1481, 2899, 2900, 6878, 6879, 6892, 6894, 6896, 42973,
                    42978, 43851, 43852, 43884, 43885, 69323, 69325, 69327, 69360, 69364, 69369, 73200,
                    119079, 119080, 119387, 119388, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011111111100000011111111111000000000000000000000000000000000|11100000000011111100000000000111111111111111111111111111111111|00011111111100000011111111111000000000000000000000000000000000|11100000000011111100000000000111111111111111111111111111111111|11100000000011111100000000000111111111111111111111111111111111|00011111111100000011111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{N}$", "^\\P{N}$", "^[\\p{N}]$", "^[\\P{N}]$", "^[^\\p{N}]$",
                    "^[^\\P{N}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 74863, 74869, 74874,
                    74879, 75088, 75243, 75398, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{N}$", "^\\P{N}$", "^[\\p{N}]$", "^[\\P{N}]$", "^[^\\p{N}]$",
                    "^[^\\P{N}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 74863, 74869, 74874,
                    74879, 75088, 75243, 75398, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{N}$", "^\\P{N}$", "^[\\p{N}]$", "^[\\P{N}]$", "^[^\\p{N}]$",
                    "^[^\\P{N}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 74863, 74869, 74874,
                    74879, 75088, 75243, 75398, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{N}$", "^\\P{N}$", "^[\\p{N}]$", "^[\\P{N}]$", "^[^\\p{N}]$",
                    "^[^\\P{N}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 74863, 74869, 74874,
                    74879, 75088, 75243, 75398, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Nd}$", "^\\P{Nd}$", "^[\\p{Nd}]$", "^[\\P{Nd}]$", "^[^\\p{Nd}]$",
                    "^[^\\P{Nd}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Nd}$", "^\\P{Nd}$", "^[\\p{Nd}]$", "^[\\P{Nd}]$", "^[^\\p{Nd}]$",
                    "^[^\\P{Nd}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Nd}$", "^\\P{Nd}$", "^[\\p{Nd}]$", "^[\\P{Nd}]$", "^[^\\p{Nd}]$",
                    "^[^\\P{Nd}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Nd}$", "^\\P{Nd}$", "^[\\p{Nd}]$", "^[\\P{Nd}]$", "^[^\\p{Nd}]$",
                    "^[^\\P{Nd}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Nl}$", "^\\P{Nl}$", "^[\\p{Nl}]$", "^[\\P{Nl}]$", "^[^\\p{Nl}]$",
                    "^[^\\P{Nl}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 74863, 74869, 74874,
                    74879, 75088, 75243, 75398, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Nl}$", "^\\P{Nl}$", "^[\\p{Nl}]$", "^[\\P{Nl}]$", "^[^\\p{Nl}]$",
                    "^[^\\P{Nl}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 74863, 74869, 74874,
                    74879, 75088, 75243, 75398, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Nl}$", "^\\P{Nl}$", "^[\\p{Nl}]$", "^[\\P{Nl}]$", "^[^\\p{Nl}]$",
                    "^[^\\P{Nl}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 74863, 74869, 74874,
                    74879, 75088, 75243, 75398, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Nl}$", "^\\P{Nl}$", "^[\\p{Nl}]$", "^[\\P{Nl}]$", "^[^\\p{Nl}]$",
                    "^[^\\P{Nl}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 74863, 74869, 74874,
                    74879, 75088, 75243, 75398, 122688, 122689, 122696, 122697, 122698, 122699, 122701,
                    122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735,
                    122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748,
                    122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000|1111111110000000111111111111111111111111111111111|1111111110000000111111111111111111111111111111111|0000000001111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{No}$", "^\\P{No}$", "^[\\p{No}]$", "^[\\P{No}]$", "^[^\\p{No}]$",
                    "^[^\\P{No}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{No}$", "^\\P{No}$", "^[\\p{No}]$", "^[\\P{No}]$", "^[^\\p{No}]$",
                    "^[^\\P{No}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{No}$", "^\\P{No}$", "^[\\p{No}]$", "^[\\P{No}]$", "^[^\\p{No}]$",
                    "^[^\\P{No}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{No}$", "^\\P{No}$", "^[\\p{No}]$", "^[\\P{No}]$", "^[^\\p{No}]$",
                    "^[^\\P{No}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{P}$", "^\\P{P}$", "^[\\p{P}]$", "^[\\P{P}]$", "^[^\\p{P}]$",
                    "^[^\\P{P}]$"
                ],
                points = [223, 631, 636, 11872, 11873, 11875, 42973, 42978, 43851, 43852, 43884, 43885,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{P}$", "^\\P{P}$", "^[\\p{P}]$", "^[\\P{P}]$", "^[^\\p{P}]$",
                    "^[^\\P{P}]$"
                ],
                points = [223, 631, 636, 11872, 11873, 11875, 42973, 42978, 43851, 43852, 43884, 43885,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{P}$", "^\\P{P}$", "^[\\p{P}]$", "^[\\P{P}]$", "^[^\\p{P}]$",
                    "^[^\\P{P}]$"
                ],
                points = [223, 631, 636, 11872, 11873, 11875, 42973, 42978, 43851, 43852, 43884, 43885,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{P}$", "^\\P{P}$", "^[\\p{P}]$", "^[\\P{P}]$", "^[^\\p{P}]$",
                    "^[^\\P{P}]$"
                ],
                points = [223, 631, 636, 11872, 11873, 11875, 42973, 42978, 43851, 43852, 43884, 43885,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pc}$", "^\\P{Pc}$", "^[\\p{Pc}]$", "^[\\P{Pc}]$", "^[^\\p{Pc}]$",
                    "^[^\\P{Pc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pc}$", "^\\P{Pc}$", "^[\\p{Pc}]$", "^[\\P{Pc}]$", "^[^\\p{Pc}]$",
                    "^[^\\P{Pc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pc}$", "^\\P{Pc}$", "^[\\p{Pc}]$", "^[\\P{Pc}]$", "^[^\\p{Pc}]$",
                    "^[^\\P{Pc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pc}$", "^\\P{Pc}$", "^[\\p{Pc}]$", "^[\\P{Pc}]$", "^[^\\p{Pc}]$",
                    "^[^\\P{Pc}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pd}$", "^\\P{Pd}$", "^[\\p{Pd}]$", "^[\\P{Pd}]$", "^[^\\p{Pd}]$",
                    "^[^\\P{Pd}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pd}$", "^\\P{Pd}$", "^[\\p{Pd}]$", "^[\\P{Pd}]$", "^[^\\p{Pd}]$",
                    "^[^\\P{Pd}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pd}$", "^\\P{Pd}$", "^[\\p{Pd}]$", "^[\\P{Pd}]$", "^[^\\p{Pd}]$",
                    "^[^\\P{Pd}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pd}$", "^\\P{Pd}$", "^[\\p{Pd}]$", "^[\\P{Pd}]$", "^[^\\p{Pd}]$",
                    "^[^\\P{Pd}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pe}$", "^\\P{Pe}$", "^[\\p{Pe}]$", "^[\\P{Pe}]$", "^[^\\p{Pe}]$",
                    "^[^\\P{Pe}]$"
                ],
                points = [223, 631, 636, 11875, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pe}$", "^\\P{Pe}$", "^[\\p{Pe}]$", "^[\\P{Pe}]$", "^[^\\p{Pe}]$",
                    "^[^\\P{Pe}]$"
                ],
                points = [223, 631, 636, 11875, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pe}$", "^\\P{Pe}$", "^[\\p{Pe}]$", "^[\\P{Pe}]$", "^[^\\p{Pe}]$",
                    "^[^\\P{Pe}]$"
                ],
                points = [223, 631, 636, 11875, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pe}$", "^\\P{Pe}$", "^[\\p{Pe}]$", "^[\\P{Pe}]$", "^[^\\p{Pe}]$",
                    "^[^\\P{Pe}]$"
                ],
                points = [223, 631, 636, 11875, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pf}$", "^\\P{Pf}$", "^[\\p{Pf}]$", "^[\\P{Pf}]$", "^[^\\p{Pf}]$",
                    "^[^\\P{Pf}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pf}$", "^\\P{Pf}$", "^[\\p{Pf}]$", "^[\\P{Pf}]$", "^[^\\p{Pf}]$",
                    "^[^\\P{Pf}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pf}$", "^\\P{Pf}$", "^[\\p{Pf}]$", "^[\\P{Pf}]$", "^[^\\p{Pf}]$",
                    "^[^\\P{Pf}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pf}$", "^\\P{Pf}$", "^[\\p{Pf}]$", "^[\\P{Pf}]$", "^[^\\p{Pf}]$",
                    "^[^\\P{Pf}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pi}$", "^\\P{Pi}$", "^[\\p{Pi}]$", "^[\\P{Pi}]$", "^[^\\p{Pi}]$",
                    "^[^\\P{Pi}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pi}$", "^\\P{Pi}$", "^[\\p{Pi}]$", "^[\\P{Pi}]$", "^[^\\p{Pi}]$",
                    "^[^\\P{Pi}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pi}$", "^\\P{Pi}$", "^[\\p{Pi}]$", "^[\\P{Pi}]$", "^[^\\p{Pi}]$",
                    "^[^\\P{Pi}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Pi}$", "^\\P{Pi}$", "^[\\p{Pi}]$", "^[\\P{Pi}]$", "^[^\\p{Pi}]$",
                    "^[^\\P{Pi}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Po}$", "^\\P{Po}$", "^[\\p{Po}]$", "^[\\P{Po}]$", "^[^\\p{Po}]$",
                    "^[^\\P{Po}]$"
                ],
                points = [223, 631, 636, 11872, 11873, 42973, 42978, 43851, 43852, 43884, 43885, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011000000000000000000000000000000000000000|11100111111111111111111111111111111111111111|00011000000000000000000000000000000000000000|11100111111111111111111111111111111111111111|11100111111111111111111111111111111111111111|00011000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Po}$", "^\\P{Po}$", "^[\\p{Po}]$", "^[\\P{Po}]$", "^[^\\p{Po}]$",
                    "^[^\\P{Po}]$"
                ],
                points = [223, 631, 636, 11872, 11873, 42973, 42978, 43851, 43852, 43884, 43885, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011000000000000000000000000000000000000000|11100111111111111111111111111111111111111111|00011000000000000000000000000000000000000000|11100111111111111111111111111111111111111111|11100111111111111111111111111111111111111111|00011000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Po}$", "^\\P{Po}$", "^[\\p{Po}]$", "^[\\P{Po}]$", "^[^\\p{Po}]$",
                    "^[^\\P{Po}]$"
                ],
                points = [223, 631, 636, 11872, 11873, 42973, 42978, 43851, 43852, 43884, 43885, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011000000000000000000000000000000000000000|11100111111111111111111111111111111111111111|00011000000000000000000000000000000000000000|11100111111111111111111111111111111111111111|11100111111111111111111111111111111111111111|00011000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Po}$", "^\\P{Po}$", "^[\\p{Po}]$", "^[\\P{Po}]$", "^[^\\p{Po}]$",
                    "^[^\\P{Po}]$"
                ],
                points = [223, 631, 636, 11872, 11873, 42973, 42978, 43851, 43852, 43884, 43885, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00011000000000000000000000000000000000000000|11100111111111111111111111111111111111111111|00011000000000000000000000000000000000000000|11100111111111111111111111111111111111111111|11100111111111111111111111111111111111111111|00011000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Ps}$", "^\\P{Ps}$", "^[\\p{Ps}]$", "^[\\P{Ps}]$", "^[^\\p{Ps}]$",
                    "^[^\\P{Ps}]$"
                ],
                points = [223, 631, 636, 11874, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Ps}$", "^\\P{Ps}$", "^[\\p{Ps}]$", "^[\\P{Ps}]$", "^[^\\p{Ps}]$",
                    "^[^\\P{Ps}]$"
                ],
                points = [223, 631, 636, 11874, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Ps}$", "^\\P{Ps}$", "^[\\p{Ps}]$", "^[\\P{Ps}]$", "^[^\\p{Ps}]$",
                    "^[^\\P{Ps}]$"
                ],
                points = [223, 631, 636, 11874, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Ps}$", "^\\P{Ps}$", "^[\\p{Ps}]$", "^[\\P{Ps}]$", "^[^\\p{Ps}]$",
                    "^[^\\P{Ps}]$"
                ],
                points = [223, 631, 636, 11874, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689,
                    122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730,
                    122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743,
                    122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000|1110111111111111111111111111111111111111111|1110111111111111111111111111111111111111111|0001000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{S}$", "^\\P{S}$", "^[\\p{S}]$", "^[\\P{S}]$", "^[^\\p{S}]$",
                    "^[^\\P{S}]$"
                ],
                points = [223, 631, 636, 8386, 8387, 8388, 42973, 42978, 43851, 43852, 43884, 43885, 69321,
                    69322, 118482, 118483, 118484, 118493, 118494, 118495, 118513, 118519, 118525, 119275,
                    119285, 119295, 119379, 119382, 119386, 119389, 119390, 119392, 119407, 119423, 121600,
                    121614, 121628, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705,
                    122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739,
                    122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750,
                    122751, 122773, 127406, 128729, 128986, 128987, 129009, 129016, 129023, 129675, 129676,
                    129677, 129740, 129757, 129771, 129785, 129786
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001110000001111111111111111111111111000000000000000000000000000000000111111111111111|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|0001110000001111111111111111111111111000000000000000000000000000000000111111111111111|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|0001110000001111111111111111111111111000000000000000000000000000000000111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{S}$", "^\\P{S}$", "^[\\p{S}]$", "^[\\P{S}]$", "^[^\\p{S}]$",
                    "^[^\\P{S}]$"
                ],
                points = [223, 631, 636, 8386, 8387, 8388, 42973, 42978, 43851, 43852, 43884, 43885, 69321,
                    69322, 118482, 118483, 118484, 118493, 118494, 118495, 118513, 118519, 118525, 119275,
                    119285, 119295, 119379, 119382, 119386, 119389, 119390, 119392, 119407, 119423, 121600,
                    121614, 121628, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705,
                    122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739,
                    122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750,
                    122751, 122773, 127406, 128729, 128986, 128987, 129009, 129016, 129023, 129675, 129676,
                    129677, 129740, 129757, 129771, 129785, 129786
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001110000001111111111111111111111111000000000000000000000000000000000111111111111111|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|0001110000001111111111111111111111111000000000000000000000000000000000111111111111111|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|0001110000001111111111111111111111111000000000000000000000000000000000111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{S}$", "^\\P{S}$", "^[\\p{S}]$", "^[\\P{S}]$", "^[^\\p{S}]$",
                    "^[^\\P{S}]$"
                ],
                points = [223, 631, 636, 8386, 8387, 8388, 42973, 42978, 43851, 43852, 43884, 43885, 69321,
                    69322, 118482, 118483, 118484, 118493, 118494, 118495, 118513, 118519, 118525, 119275,
                    119285, 119295, 119379, 119382, 119386, 119389, 119390, 119392, 119407, 119423, 121600,
                    121614, 121628, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705,
                    122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739,
                    122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750,
                    122751, 122773, 127406, 128729, 128986, 128987, 129009, 129016, 129023, 129675, 129676,
                    129677, 129740, 129757, 129771, 129785, 129786
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001110000001111111111111111111111111000000000000000000000000000000000111111111111111|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|0001110000001111111111111111111111111000000000000000000000000000000000111111111111111|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|0001110000001111111111111111111111111000000000000000000000000000000000111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{S}$", "^\\P{S}$", "^[\\p{S}]$", "^[\\P{S}]$", "^[^\\p{S}]$",
                    "^[^\\P{S}]$"
                ],
                points = [223, 631, 636, 8386, 8387, 8388, 42973, 42978, 43851, 43852, 43884, 43885, 69321,
                    69322, 118482, 118483, 118484, 118493, 118494, 118495, 118513, 118519, 118525, 119275,
                    119285, 119295, 119379, 119382, 119386, 119389, 119390, 119392, 119407, 119423, 121600,
                    121614, 121628, 122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705,
                    122706, 122728, 122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739,
                    122740, 122741, 122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750,
                    122751, 122773, 127406, 128729, 128986, 128987, 129009, 129016, 129023, 129675, 129676,
                    129677, 129740, 129757, 129771, 129785, 129786
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "0001110000001111111111111111111111111000000000000000000000000000000000111111111111111|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|0001110000001111111111111111111111111000000000000000000000000000000000111111111111111|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|1110001111110000000000000000000000000111111111111111111111111111111111000000000000000|0001110000001111111111111111111111111000000000000000000000000000000000111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sc}$", "^\\P{Sc}$", "^[\\p{Sc}]$", "^[\\P{Sc}]$", "^[^\\p{Sc}]$",
                    "^[^\\P{Sc}]$"
                ],
                points = [223, 631, 636, 8386, 8387, 8388, 42973, 42978, 43851, 43852, 43884, 43885,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sc}$", "^\\P{Sc}$", "^[\\p{Sc}]$", "^[\\P{Sc}]$", "^[^\\p{Sc}]$",
                    "^[^\\P{Sc}]$"
                ],
                points = [223, 631, 636, 8386, 8387, 8388, 42973, 42978, 43851, 43852, 43884, 43885,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sc}$", "^\\P{Sc}$", "^[\\p{Sc}]$", "^[\\P{Sc}]$", "^[^\\p{Sc}]$",
                    "^[^\\P{Sc}]$"
                ],
                points = [223, 631, 636, 8386, 8387, 8388, 42973, 42978, 43851, 43852, 43884, 43885,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sc}$", "^\\P{Sc}$", "^[\\p{Sc}]$", "^[\\P{Sc}]$", "^[^\\p{Sc}]$",
                    "^[^\\P{Sc}]$"
                ],
                points = [223, 631, 636, 8386, 8387, 8388, 42973, 42978, 43851, 43852, 43884, 43885,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000|111000111111111111111111111111111111111111111|111000111111111111111111111111111111111111111|000111000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sk}$", "^\\P{Sk}$", "^[\\p{Sk}]$", "^[\\P{Sk}]$", "^[^\\p{Sk}]$",
                    "^[^\\P{Sk}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 69321, 69322, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011000000000000000000000000000000000|11111111100111111111111111111111111111111111|00000000011000000000000000000000000000000000|11111111100111111111111111111111111111111111|11111111100111111111111111111111111111111111|00000000011000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sk}$", "^\\P{Sk}$", "^[\\p{Sk}]$", "^[\\P{Sk}]$", "^[^\\p{Sk}]$",
                    "^[^\\P{Sk}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 69321, 69322, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011000000000000000000000000000000000|11111111100111111111111111111111111111111111|00000000011000000000000000000000000000000000|11111111100111111111111111111111111111111111|11111111100111111111111111111111111111111111|00000000011000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sk}$", "^\\P{Sk}$", "^[\\p{Sk}]$", "^[\\P{Sk}]$", "^[^\\p{Sk}]$",
                    "^[^\\P{Sk}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 69321, 69322, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011000000000000000000000000000000000|11111111100111111111111111111111111111111111|00000000011000000000000000000000000000000000|11111111100111111111111111111111111111111111|11111111100111111111111111111111111111111111|00000000011000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sk}$", "^\\P{Sk}$", "^[\\p{Sk}]$", "^[\\P{Sk}]$", "^[^\\p{Sk}]$",
                    "^[^\\P{Sk}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 69321, 69322, 122688,
                    122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729,
                    122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742,
                    122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011000000000000000000000000000000000|11111111100111111111111111111111111111111111|00000000011000000000000000000000000000000000|11111111100111111111111111111111111111111111|11111111100111111111111111111111111111111111|00000000011000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sm}$", "^\\P{Sm}$", "^[\\p{Sm}]$", "^[\\P{Sm}]$", "^[^\\p{Sm}]$",
                    "^[^\\P{Sm}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 118493, 118494, 118495,
                    118513, 118519, 118525, 121600, 121614, 121628, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000111111111000000000000000000000000000000000|111111111000000000111111111111111111111111111111111|000000000111111111000000000000000000000000000000000|111111111000000000111111111111111111111111111111111|111111111000000000111111111111111111111111111111111|000000000111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sm}$", "^\\P{Sm}$", "^[\\p{Sm}]$", "^[\\P{Sm}]$", "^[^\\p{Sm}]$",
                    "^[^\\P{Sm}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 118493, 118494, 118495,
                    118513, 118519, 118525, 121600, 121614, 121628, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000111111111000000000000000000000000000000000|111111111000000000111111111111111111111111111111111|000000000111111111000000000000000000000000000000000|111111111000000000111111111111111111111111111111111|111111111000000000111111111111111111111111111111111|000000000111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sm}$", "^\\P{Sm}$", "^[\\p{Sm}]$", "^[\\P{Sm}]$", "^[^\\p{Sm}]$",
                    "^[^\\P{Sm}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 118493, 118494, 118495,
                    118513, 118519, 118525, 121600, 121614, 121628, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000111111111000000000000000000000000000000000|111111111000000000111111111111111111111111111111111|000000000111111111000000000000000000000000000000000|111111111000000000111111111111111111111111111111111|111111111000000000111111111111111111111111111111111|000000000111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Sm}$", "^\\P{Sm}$", "^[\\p{Sm}]$", "^[\\P{Sm}]$", "^[^\\p{Sm}]$",
                    "^[^\\P{Sm}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 118493, 118494, 118495,
                    118513, 118519, 118525, 121600, 121614, 121628, 122688, 122689, 122696, 122697, 122698,
                    122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731, 122732, 122733,
                    122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744, 122745, 122746,
                    122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000111111111000000000000000000000000000000000|111111111000000000111111111111111111111111111111111|000000000111111111000000000000000000000000000000000|111111111000000000111111111111111111111111111111111|111111111000000000111111111111111111111111111111111|000000000111111111000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{So}$", "^\\P{So}$", "^[\\p{So}]$", "^[\\P{So}]$", "^[^\\p{So}]$",
                    "^[^\\P{So}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 118482, 118483, 118484,
                    119275, 119285, 119295, 119379, 119382, 119386, 119389, 119390, 119392, 119407, 119423,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773,
                    127406, 128729, 128986, 128987, 129009, 129016, 129023, 129675, 129676, 129677, 129740,
                    129757, 129771, 129785, 129786
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011111111111111000000000000000000000000000000000111111111111111|11111111100000000000000111111111111111111111111111111111000000000000000|00000000011111111111111000000000000000000000000000000000111111111111111|11111111100000000000000111111111111111111111111111111111000000000000000|11111111100000000000000111111111111111111111111111111111000000000000000|00000000011111111111111000000000000000000000000000000000111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{So}$", "^\\P{So}$", "^[\\p{So}]$", "^[\\P{So}]$", "^[^\\p{So}]$",
                    "^[^\\P{So}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 118482, 118483, 118484,
                    119275, 119285, 119295, 119379, 119382, 119386, 119389, 119390, 119392, 119407, 119423,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773,
                    127406, 128729, 128986, 128987, 129009, 129016, 129023, 129675, 129676, 129677, 129740,
                    129757, 129771, 129785, 129786
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011111111111111000000000000000000000000000000000111111111111111|11111111100000000000000111111111111111111111111111111111000000000000000|00000000011111111111111000000000000000000000000000000000111111111111111|11111111100000000000000111111111111111111111111111111111000000000000000|11111111100000000000000111111111111111111111111111111111000000000000000|00000000011111111111111000000000000000000000000000000000111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{So}$", "^\\P{So}$", "^[\\p{So}]$", "^[\\P{So}]$", "^[^\\p{So}]$",
                    "^[^\\P{So}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 118482, 118483, 118484,
                    119275, 119285, 119295, 119379, 119382, 119386, 119389, 119390, 119392, 119407, 119423,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773,
                    127406, 128729, 128986, 128987, 129009, 129016, 129023, 129675, 129676, 129677, 129740,
                    129757, 129771, 129785, 129786
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011111111111111000000000000000000000000000000000111111111111111|11111111100000000000000111111111111111111111111111111111000000000000000|00000000011111111111111000000000000000000000000000000000111111111111111|11111111100000000000000111111111111111111111111111111111000000000000000|11111111100000000000000111111111111111111111111111111111000000000000000|00000000011111111111111000000000000000000000000000000000111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{So}$", "^\\P{So}$", "^[\\p{So}]$", "^[\\P{So}]$", "^[^\\p{So}]$",
                    "^[^\\P{So}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 118482, 118483, 118484,
                    119275, 119285, 119295, 119379, 119382, 119386, 119389, 119390, 119392, 119407, 119423,
                    122688, 122689, 122696, 122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728,
                    122729, 122730, 122731, 122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741,
                    122742, 122743, 122744, 122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773,
                    127406, 128729, 128986, 128987, 129009, 129016, 129023, 129675, 129676, 129677, 129740,
                    129757, 129771, 129785, 129786
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "00000000011111111111111000000000000000000000000000000000111111111111111|11111111100000000000000111111111111111111111111111111111000000000000000|00000000011111111111111000000000000000000000000000000000111111111111111|11111111100000000000000111111111111111111111111111111111000000000000000|11111111100000000000000111111111111111111111111111111111000000000000000|00000000011111111111111000000000000000000000000000000000111111111111111"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Z}$", "^\\P{Z}$", "^[\\p{Z}]$", "^[\\P{Z}]$", "^[^\\p{Z}]$",
                    "^[^\\P{Z}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Z}$", "^\\P{Z}$", "^[\\p{Z}]$", "^[\\P{Z}]$", "^[^\\p{Z}]$",
                    "^[^\\P{Z}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Z}$", "^\\P{Z}$", "^[\\p{Z}]$", "^[\\P{Z}]$", "^[^\\p{Z}]$",
                    "^[^\\P{Z}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Z}$", "^\\P{Z}$", "^[\\p{Z}]$", "^[\\P{Z}]$", "^[^\\p{Z}]$",
                    "^[^\\P{Z}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zl}$", "^\\P{Zl}$", "^[\\p{Zl}]$", "^[\\P{Zl}]$", "^[^\\p{Zl}]$",
                    "^[^\\P{Zl}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zl}$", "^\\P{Zl}$", "^[\\p{Zl}]$", "^[\\P{Zl}]$", "^[^\\p{Zl}]$",
                    "^[^\\P{Zl}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zl}$", "^\\P{Zl}$", "^[\\p{Zl}]$", "^[\\P{Zl}]$", "^[^\\p{Zl}]$",
                    "^[^\\P{Zl}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zl}$", "^\\P{Zl}$", "^[\\p{Zl}]$", "^[\\P{Zl}]$", "^[^\\p{Zl}]$",
                    "^[^\\P{Zl}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zp}$", "^\\P{Zp}$", "^[\\p{Zp}]$", "^[\\P{Zp}]$", "^[^\\p{Zp}]$",
                    "^[^\\P{Zp}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zp}$", "^\\P{Zp}$", "^[\\p{Zp}]$", "^[\\P{Zp}]$", "^[^\\p{Zp}]$",
                    "^[^\\P{Zp}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zp}$", "^\\P{Zp}$", "^[\\p{Zp}]$", "^[\\P{Zp}]$", "^[^\\p{Zp}]$",
                    "^[^\\P{Zp}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zp}$", "^\\P{Zp}$", "^[\\p{Zp}]$", "^[\\P{Zp}]$", "^[^\\p{Zp}]$",
                    "^[^\\P{Zp}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zs}$", "^\\P{Zs}$", "^[\\p{Zs}]$", "^[\\P{Zs}]$", "^[^\\p{Zs}]$",
                    "^[^\\P{Zs}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "u";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zs}$", "^\\P{Zs}$", "^[\\p{Zs}]$", "^[\\P{Zs}]$", "^[^\\p{Zs}]$",
                    "^[^\\P{Zs}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "ui";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zs}$", "^\\P{Zs}$", "^[\\p{Zs}]$", "^[\\P{Zs}]$", "^[^\\p{Zs}]$",
                    "^[^\\P{Zs}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "v";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
    assert_true(
        r#"
            let patterns = ["^\\p{Zs}$", "^\\P{Zs}$", "^[\\p{Zs}]$", "^[\\P{Zs}]$", "^[^\\p{Zs}]$",
                    "^[^\\P{Zs}]$"
                ],
                points = [223, 631, 636, 42973, 42978, 43851, 43852, 43884, 43885, 122688, 122689, 122696,
                    122697, 122698, 122699, 122701, 122702, 122705, 122706, 122728, 122729, 122730, 122731,
                    122732, 122733, 122734, 122735, 122738, 122739, 122740, 122741, 122742, 122743, 122744,
                    122745, 122746, 122747, 122748, 122749, 122750, 122751, 122773
                ],
                flags = "vi";
            patterns.map(pattern => {
                    let r = new RegExp(pattern, flags);
                    return points.map(cp => r.test(String.fromCodePoint(cp)) ? '1' : '0').join('')
                }).join('|') ===
                "000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000|111111111111111111111111111111111111111111|111111111111111111111111111111111111111111|000000000000000000000000000000000000000000"
        "#,
    );
}

#[test]
fn unicode_general_category_results_snapshot() {
    let mut rows = String::new();
    for name in [
        "C", "Cc", "Cf", "Cn", "Co", "Cs", "L", "LC", "Ll", "Lm", "Lo", "Lt", "Lu", "M", "Mc",
        "Me", "Mn", "N", "Nd", "Nl", "No", "P", "Pc", "Pd", "Pe", "Pf", "Pi", "Po", "Ps", "S",
        "Sc", "Sk", "Sm", "So", "Z", "Zl", "Zp", "Zs",
    ] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{gc={name}}}"),
            format!(r"(?<x>[\p{{General_Category={name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["KaμΣς😀\u{378}\n", "X\r\nA\u{2028}😀", ""] {
                    let input = JsString::from(input);
                    for start in [0, 1, 3] {
                        let script = format!(
                            r#"
                                let r = new RegExp({pattern:?}, {flags:?});
                                r.lastIndex = {start};
                                let m = r.exec({input:?});
                                JSON.stringify({{
                                    m,
                                    index: m?.index,
                                    groups: m?.groups,
                                    indices: m?.indices,
                                    indexGroups: m?.indices?.groups,
                                    lastIndex: r.lastIndex,
                                    source: r.source
                                }})
                            "#
                        );
                        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                            panic!("{script}")
                        };
                        writeln!(
                            rows,
                            "{body:?} flags={flags:?} input={input:?} lastIndex={start} => {result}"
                        ,
result = result.to_utf8().expect("JSON output must be valid UTF-8"),
)
                        .unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_general_category_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{gc=Lu}}K]))$/dim{mode}g,
                    m = r.exec('\u0378\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\p{{gc=So}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀A');
                m.index === 0 && m[1] === '😀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{Cs}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('😀\uD800');
                m.index === 2 && m[0] === '\uD800' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{Cs}}/{mode}y;r.lastIndex=1;r.exec('😀\uD800')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_general_category_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'A😀\u0378B'.matchAll(/(?<x>\P{{L}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1
                ] === '\u0378' && a[1].index === 3
            "#
        ));
        assert_true(&format!(
            r"'AB\u0378'.replace(/(?<x>\p{{Letter}})/{mode}g,'<$1:$<x>>')==='<A:A><B:B>\u0378'"
        ));
        assert_true(&format!(
            r"'\u0378AB\u0378'.split(/(\p{{General_Category=Letter}})/{mode}).join('|')==='\u0378|A||B|\u0378'&&'\u0378😀A'.search(/\p{{L}}/{mode})===3&&'AB\u0378'.match(/\p{{L}}/{mode}g).join('')==='AB'"
        ));
        assert_true(&format!(
            r#"
                let t = '',
                    x = 'AB\u0378A'.replace(/(\p{{L}})/{mode}g, (m, c, p) => {{
                        t += p;
                        return '_'
                    }});
                x === '__\u0378_' && t === '013'
            "#
        ));
    }
}

#[test]
fn unicode_general_category_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{gc=Lu})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{gc=Lu})$' && c.unicodeSets && !c.unicode && !c.multiline && c.exec('a')
                .groups.x === 'a' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\p{So}/uy;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return '😀'
                }
            });
            t === 'si' && m[0] === '😀' && m.index === 0 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let r = /\p{Lu}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('A')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{Cs}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('😀\uD800')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_general_category_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Lu}/ug,marker=0,text='\\u0378'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert_true(
        r#"
            let m = /\p{Script_Extensions=Latin}/u.exec('A');
            JSON.stringify(m) === '["A"]' && m.index === 0 && m.input === 'A'
        "#,
    );
    assert_true(
        r"let m=/[\p{Lu}&&A]/v.exec('A');m!==null&&m.length===1&&m[0]==='A'&&m.index===0&&m.input==='A'",
    );
    assert_true(r"/[\p{Lu}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x4b),
                m = /[\p{Lu}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{Lu}+b/u.exec('AA')", r"/\p{Lu}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_general_category_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{So}})+\B/d{mode}g,
                    m = r.exec('\u0378😀😀😀\u0378');
                m.index === 1 && m[0] === '😀😀😀' && m.groups.x === '😀' && m.indices.groups.x[0] === 5 && m
                    .indices.groups.x[1] === 7 && r.lastIndex === 7
            "#
        ));
    }
}

#[test]
fn unicode_general_category_direct_eval_preserves_surrounding_execution() {
    assert_true(
        r#"
            let marker = 0,
                result;
            try {
                result = eval("marker=1; /\\p{General_Category=Letter}/u.test('a');")
            } catch {
                marker = 2
            } finally {
                marker = 3
            }
            marker === 3 && result === true
        "#,
    );
}
