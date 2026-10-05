//! Assignment cover refinement, member/rest targets, and strict early errors (13.15.5).

use spite_core::DiagnosticKind;
use spite_parser::{MAX_DEPTH, parse_script};

#[test]
fn assignment_pattern_syntax_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("({[key]:[obj.x=1,...xs],x=2,...obj.rest}=source);for([x] of rows) ;")
            .unwrap()
    );
}

#[test]
fn assignment_pattern_diagnostics_snapshot() {
    insta::assert_debug_snapshot!(
        [
            "[...x,]=source",
            "({...{x}}=source)",
            "[x+y]=source",
            "'use strict';({x:arguments}=source)",
            "([x])=source",
            "({x=1})",
            "for([x]=[] of source) ;",
        ]
        .map(|source| parse_script(source).unwrap_err())
    );
}

#[test]
fn patterns_accept_repeated_names_nested_defaults_references_and_every_iteration_header() {
    for pattern in [
        "{}",
        "[]",
        "[x,x]",
        "{x,a:x,...obj.rest}",
        "[obj.x,obj[key],(x),...{length:n}]",
        "{[key in source]:[x=1,...xs]}",
        "{__proto__:x,__proto__:y}",
        "[...[x]]",
        "[{x}.y,[x].length]",
    ] {
        for source in [
            format!("({pattern}=source);"),
            format!("for({pattern} of rows) ;"),
            format!("for({pattern} in rows) ;"),
            format!("for({pattern}=source;false;) ;"),
        ] {
            assert!(
                parse_script(&source).is_ok(),
                "{source}: {:?}",
                parse_script(&source)
            );
        }
    }
    assert!(parse_script("[a]=[b]=source;").is_ok());
    assert!(parse_script("for(([]).length in source) ;").is_ok());
    assert!(parse_script("for(({x}).y of source) ;").is_ok());
}

#[test]
fn ordinary_literals_keep_their_grammar_and_parenthesized_patterns_are_not_targets() {
    for source in [
        "({x=1})",
        "({__proto__:1,__proto__:2})",
        "({...x,...y}=source)",
        "({...x,}=source)",
        "[...x=1]=source",
        "({...x=1}=source)",
        "({...[]}=source)",
        "[f()]=source",
        "[a?.b]=source",
        "({a:b+c}=source)",
        "({m(){}}=source)",
        "({get x(){}}=source)",
        "([x])=source",
        "({x})=source",
        "[([x])]=source",
        "[]+=source",
        "for(([x]) of source) ;",
        "for([x],y in source) ;",
        "'use strict';[eval]=source",
        "'use strict';({...arguments}=source)",
        "'use strict';({[delete x]:y}=source)",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
    for source in [
        "let x=[1,2]",
        "({x:1,__proto__:null})",
        "[x] in source",
        "([x=1])",
        "[x]\nof;",
        "({x}).y",
        "[x].length=1",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn defaults_and_keys_preserve_in_new_target_and_unavailable_feature_categories() {
    assert!(
        parse_script("function f(){for({[new.target]:x=x in source}=source;false;) ;}").is_ok()
    );
    for source in [
        "[x=class{}]=source",
        "({x=async()=>0}=source)",
        "[x=function*(){}]=source",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Unsupported,
            "{source}"
        );
    }
    assert_eq!(
        parse_script("({[new.target]:x}=source)").unwrap_err().kind,
        DiagnosticKind::Syntax
    );
}

#[test]
fn nested_patterns_and_reentrant_initializer_syntax_use_the_existing_depth_guard() {
    let source = format!("{}x{}=source", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
    let source = format!(
        "{}0{}",
        "[x=".repeat(MAX_DEPTH),
        "]=source".repeat(MAX_DEPTH)
    );
    assert_eq!(
        parse_script(&source).unwrap_err().kind,
        DiagnosticKind::Limit
    );
}
