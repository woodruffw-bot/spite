//! AwaitExpression grammar, context, operand precedence and early errors.

use spite_core::DiagnosticKind;
use spite_parser::{
    ast::{BinaryOp, ExprKind, StatementKind},
    parse_script,
};

#[test]
fn await_operand_precedence_and_spans_are_retained() {
    let script = parse_script("async function f(){return await x * 2}").unwrap();
    let statement = &script.function_declarations()[0].body.statements()[0];
    let StatementKind::Return(Some(expression)) = &statement.kind else {
        panic!("return");
    };
    let ExprKind::Binary(BinaryOp::Multiply, left, _) = &expression.kind else {
        panic!("multiplication");
    };
    assert!(matches!(left.kind, ExprKind::Await(_)));
    assert_eq!(
        &"async function f(){return await x * 2}"[left.span.start..left.span.end],
        "await x"
    );
    insta::assert_debug_snapshot!(script);
}

#[test]
fn await_accepts_unary_operands_linebreaks_and_nested_async_contexts() {
    for source in [
        "async function f(){return await -x}",
        "async()=>await\nx",
        "async()=>await await value",
        "async()=>await /x/.test('x')",
        "async()=>await (x=>x)",
        "async()=>await (x=1)",
        "async function f(){(x=await /r/.exec('r'))}",
        "async function f(){(x=async()=>await y)=>x}",
        "async function f(){(x={async m(){await y}})=>x}",
        "async()=>2 ** await x",
        "async()=>(await x) ** 2",
        "async function f(){return class extends (await Base){[await name](){return 1}}}",
        "({async m(){return await this.x}})",
        "class C{async #m(){return await this.x}static async m(){return await super.x}}",
        "async function f(){function g(await){return await}return ()=>await}",
        "async function f(){using resource=await acquire()}",
        "async()=>await using",
    ] {
        assert!(
            parse_script(source).is_ok(),
            "{source}: {:?}",
            parse_script(source)
        );
    }
}

#[test]
fn await_is_not_an_assignment_target_or_an_unparenthesized_exponent_base() {
    for body in [
        "await;",
        "(await x)=1",
        "(await x)++",
        "++await x",
        "await x ** 2",
        "await x => x",
        "new await C()",
        "class C extends await Base{}",
        "let \\u0061wait=1",
        "\\u0061wait x",
        "await\nusing x=null",
    ] {
        let source = format!("async function f(){{{body}}}");
        assert_eq!(
            parse_script(&source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}

#[test]
fn strict_and_private_validation_reaches_the_await_operand() {
    for source in [
        "async function f(){'use strict';await delete x}",
        "async function f(){'use strict';await (eval=1)}",
        "async function f(){await object.#missing}",
        "class C{#x;async m(){await delete this.#x}}",
        "async function f(x=await value){}",
        "async(x=await value)=>x",
        "async function f(){(x=await value)=>x}",
        "async function f(){({[await key]:x})=>x}",
        "async function f(){(x=await /r/)=>x}",
        "class C{static{await value}}",
        "class C{x=await value}",
    ] {
        assert_eq!(
            parse_script(source).unwrap_err().kind,
            DiagnosticKind::Syntax,
            "{source}"
        );
    }
}
