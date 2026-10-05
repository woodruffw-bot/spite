//! Unicode property expression grammar and exact edition-17 alias early errors.

use spite_core::{DiagnosticKind, JsString};
use spite_parser::{
    EvalContext, parse_dynamic_function, parse_eval_utf16, parse_script, parse_script_utf16,
};

fn validates(pattern: &str, flags: &str) {
    let source = format!("/{pattern}/{flags}");
    let error = parse_script(&source).unwrap_err();
    assert_eq!(error.kind, DiagnosticKind::Unsupported, "{source}");
    assert_eq!(
        error.message, "regular expression matching is not implemented",
        "{source}"
    );
}

#[test]
fn general_category_and_script_aliases_are_exact_and_unicode_version_is_pinned() {
    for flags in ["u", "v"] {
        for name in ["General_Category", "gc"] {
            for value in [
                "L",
                "Letter",
                "LC",
                "Cased_Letter",
                "Lu",
                "Uppercase_Letter",
                "Nd",
                "Decimal_Number",
                "digit",
                "Cn",
                "Unassigned",
            ] {
                validates(&format!(r"\p{{{name}={value}}}"), flags);
                validates(&format!(r"[\P{{{name}={value}}}]"), flags);
            }
        }
        for name in ["Script", "sc", "Script_Extensions", "scx"] {
            for value in [
                "Latn",
                "Latin",
                "Copt",
                "Coptic",
                "Qaac",
                "Zinh",
                "Inherited",
                "Qaai",
                "Zyyy",
                "Common",
                "Zzzz",
                "Unknown",
                "Jurc",
                "Jurchen",
                "Pcun",
                "Proto_Cuneiform",
                "Seal",
                "Thai",
            ] {
                validates(&format!(r"\P{{{name}={value}}}"), flags);
                validates(&format!(r"[\p{{{name}={value}}}]"), flags);
            }
        }
        for value in ["L", "Letter", "LC", "Cased_Letter", "digit", "Surrogate"] {
            validates(&format!(r"\p{{{value}}}"), flags);
        }
    }
}

#[test]
fn binary_properties_use_only_ecmascript_whitelisted_aliases() {
    for flags in ["u", "v"] {
        for value in [
            "ASCII",
            "Any",
            "Assigned",
            "ASCII_Hex_Digit",
            "AHex",
            "Alphabetic",
            "Alpha",
            "Changes_When_NFKC_Casefolded",
            "CWKCF",
            "Emoji",
            "EBase",
            "EComp",
            "EMod",
            "EPres",
            "Extended_Pictographic",
            "ExtPict",
            "White_Space",
            "space",
            "Sentence_Terminal",
            "STerm",
            "ID_Start",
            "IDS",
            "XID_Continue",
            "XIDC",
        ] {
            validates(&format!(r"\p{{{value}}}"), flags);
            validates(&format!(r"[^\P{{{value}}}]"), flags);
        }
        // UCD loose matching, Is-prefixes, and additional aliases/properties
        // are deliberately excluded from the edition-17 tables.
        for value in [
            "WSpace",
            "Hyphen",
            "Other_Alphabetic",
            "OAlpha",
            "Bidi_Class",
            "IsAlphabetic",
            "alphabetic",
            "ALPHABETIC",
            "WhiteSpace",
            "Latin",
            "Latn",
            "Yes",
            "Y",
            "letter",
            "ll",
            "RGI_Emoji_ZWJ_Sequences",
        ] {
            let source = format!(r"/\p{{{value}}}/{flags}");
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{source}"
            );
        }
    }
}

#[test]
fn string_properties_require_v_and_obey_negation_and_containment_early_errors() {
    for value in [
        "Basic_Emoji",
        "Emoji_Keycap_Sequence",
        "RGI_Emoji",
        "RGI_Emoji_Flag_Sequence",
        "RGI_Emoji_Modifier_Sequence",
        "RGI_Emoji_Tag_Sequence",
        "RGI_Emoji_ZWJ_Sequence",
    ] {
        for pattern in [
            format!(r"\p{{{value}}}"),
            format!(r"[\p{{{value}}}]"),
            format!(r"[^\p{{{value}}}&&\p{{Letter}}]"),
            format!(r"[^\p{{Letter}}--\p{{{value}}}]"),
            format!(r"[\p{{{value}}}--\p{{{value}}}]"),
            format!(r"[^\p{{{value}}}&&[a]]"),
        ] {
            validates(&pattern, "v");
        }
        for (pattern, flags) in [
            (format!(r"\p{{{value}}}"), "u"),
            (format!(r"\P{{{value}}}"), "v"),
            (format!(r"[\P{{{value}}}]"), "v"),
            (format!(r"[^\p{{{value}}}]"), "v"),
            (format!(r"[^\p{{{value}}}--\p{{{value}}}]"), "v"),
            (format!(r"[[^\p{{{value}}}]&&a]"), "v"),
            (format!(r"[^\p{{{value}}}&&\q{{ab}}]"), "v"),
        ] {
            let source = format!("/{pattern}/{flags}");
            assert_eq!(
                parse_script(&source).unwrap_err().kind,
                DiagnosticKind::Syntax,
                "{source}"
            );
        }
    }
    for source in [
        r"/[a-\p{Letter}]/u",
        r"/[\p{Letter}-z]/u",
        r"/[a-\p{Letter}]/v",
        r"/[\p{Letter}-z]/v",
        r"/[\q{\p{Letter}}]/v",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn property_grammar_errors_share_exact_diagnostics_across_source_entry_points() {
    for pattern in [
        r"\p",
        r"\p{}",
        r"\p{=L}",
        r"\p{gc=}",
        r"\p{gc=L=L}",
        r"\p{gc=Not_A_Category}",
        r"\p{gc=Latin}",
        r"\p{sc=Letter}",
        r"\p{GC=L}",
        r"\p{GeneralCategory=L}",
        r"\p{scx=latin}",
        r"\p{gc2=L}",
        r"\p{Alphabetic=Yes}",
        r"\p{Basic_Emoji=Yes}",
        r"\p{gc = L}",
        r"\p{White-Space}",
        r"\p{White Space}",
        r"\p{\u004C}",
        r"\p{sc=\u004Catin}",
        r"\p{L",
        r"\p{Ä}",
        r"\p{Invalid}",
    ] {
        for flags in ["u", "v"] {
            let source = format!("/{pattern}/{flags}");
            let expected = parse_script(&source).unwrap_err();
            assert_eq!(expected.kind, DiagnosticKind::Syntax, "{source}");
            let utf16 = JsString::from(source.as_str());
            assert_eq!(parse_script_utf16(&utf16).unwrap_err(), expected);
            assert_eq!(
                parse_eval_utf16(&utf16, EvalContext::default()).unwrap_err(),
                expected
            );
            assert_eq!(
                parse_dynamic_function(&format!("x = {source}"), "return x;")
                    .unwrap_err()
                    .kind,
                DiagnosticKind::Syntax
            );
            assert_eq!(
                parse_dynamic_function("x", &format!("return {source};"))
                    .unwrap_err()
                    .kind,
                DiagnosticKind::Syntax
            );
        }
    }
    let errors: Vec<_> = [
        r"/\p/u",
        r"/\p{}/u",
        r"/\p{gc=L=L}/v",
        r"/\p{gc2=L}/u",
        r"/\p{Alphabetic=Yes}/v",
        r"/\p{sc=Letter}/u",
        r"/\p{WSpace}/v",
        r"/\P{RGI_Emoji}/v",
        r"/[^\p{Basic_Emoji}]/v",
    ]
    .into_iter()
    .map(|source| parse_script(source).unwrap_err())
    .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn property_scans_do_not_need_default_limits_and_non_unicode_escapes_remain_distinct() {
    validates(&r"\p{Letter}".repeat(20_000), "u");
    let source = format!(r"/\p{{{}}}/v", "A".repeat(100_000));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Syntax
    );
    for source in [r"/\p{Letter}/", r"/[\p{Letter}]/", r"/\P{Letter}/"] {
        // Non-Unicode IdentityEscape does not admit ID_Continue characters;
        // Annex B's legacy property-shaped identity escapes are excluded.
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax
        );
    }
}
