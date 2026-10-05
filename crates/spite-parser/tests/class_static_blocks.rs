//! Static block statement grammar, function boundaries, and early errors (15.7).

use spite_core::DiagnosticKind;
use spite_parser::parse_script;

#[test]
fn static_block_syntax_and_function_boundaries_snapshot() {
    insta::assert_debug_snapshot!(
        parse_script("class C { static x=1; static { var x=2; this.y=x; } static {} }").unwrap()
    );
    for source in [
        "class C{static\n{}}",
        "class C{static{let x=1;function f(){return arguments;}()=>{return 1;};}}",
        "class C{static{for(let i=0;i<2;i++){if(i)break;continue;}}}",
        "class C{static{a:while(true){break a;}}}",
        "class C{static{new.target;super.x;()=>super.x;}}",
        "class C{static{()=>await;()=>{let await=1;return await;};function f(await){return await;}({m(await){return await;}});}}",
        "class C{static{({await:1}).await;let {await:x}={};}}",
        "class C{static{let f=function await(){};}}",
        "class C{static{let x=1;}static{let x=2;}}",
    ] {
        assert!(parse_script(source).is_ok(), "{source}");
    }
}

#[test]
fn static_block_early_errors_snapshot() {
    let errors: Vec<_> = [
        "class C{static{return;}}",
        "class C{static{arguments;}}",
        "class C{static{()=>arguments;}}",
        "class C extends B{static{super();}}",
        "class C{static{await;}}",
        "class C{static{aw\\u0061it;}}",
        "class C{static{var await;}}",
        "class C{static{let {await}={};}}",
        "class C{static{class await{}}}",
        "class C{static{function await(){}}}",
        "class C{static{(await)=>1;}}",
        "class C{static{await:;}}",
        "outer:while(true){class C{static{break outer;}}}",
        "while(true){class C{static{continue;}}}",
        "class C{static{let x;var x;}}",
        "class C{static{let x;let x;}}",
        "class C{static{with({}){}}}",
    ]
    .into_iter()
    .map(|source| {
        let error = parse_script(source).unwrap_err();
        assert_eq!(error.kind, DiagnosticKind::Syntax, "{source}");
        (source, error)
    })
    .collect();
    insta::assert_debug_snapshot!(errors);
}
