//! Source code points and exact Function.prototype.toString (11.1, 20.2.3.5).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn string_and_identity_escape_literals_preserve_surrogates_and_replacement_characters() {
    check(
        r#"let text='\uD800\uD800\uDC00\uDFFF\uFFFD',body="return '"+text+"';",f=Function(body);f()===text && f.toString()==='function anonymous(\n) {\n'+body+'\n}'"#,
    );
    check(
        r#"let high='\uD800',low='\uDFFF';Function("return '\\"+high+"';")()===high && Function("return '\\"+low+"';")()===low && Function("return '\uFFFD';")()==='\uFFFD'"#,
    );
    check(
        r#"let high='\uD800',low='\uDFFF';Function("a='"+high+"'","b='"+low+"'",'return a+b')()===high+low"#,
    );
}

#[test]
fn templates_preserve_cooked_raw_invalid_escapes_and_original_line_endings() {
    check(
        r#"let text='\uD800\r\n\uDFFF',body='return `'+text+'`;',f=Function(body);f()==='\uD800\n\uDFFF' && f.toString()==='function anonymous(\n) {\n'+body+'\n}'"#,
    );
    check(
        r#"let text='\uD800\r\n\uDFFF';Function('tag','return tag`'+text+'${7}\uFFFD`;')(function(s,x){return s[0]==='\uD800\n\uDFFF' && s.raw[0]===s[0] && s[1]==='\uFFFD' && s.raw[1]===s[1] && x===7;})"#,
    );
    check(
        r#"let text='\\xZ\uD800';Function('tag','return tag`'+text+'`;')(function(s){return s[0]===undefined && s.raw[0]===text;})"#,
    );
    check(
        r#"let text='\\\uD800';Function('tag','return tag`'+text+'`;')(function(s){return s[0]==='\uD800' && s.raw[0]===text;})"#,
    );
}

#[test]
fn comments_and_all_nested_function_sources_retain_their_own_ranges() {
    check(
        r#"let high='\uD800',low='\uDFFF',p='a/*'+high+'*/',body='//'+low+'\nreturn a;';let f=Function(p,body);f(7)===7 && f.toString()==='function anonymous('+p+'\n) {\n'+body+'\n}'"#,
    );
    check(
        r#"let text='\uD800',body="/*"+text+"*/return [function inner(){return '"+text+"';},()=> '"+text+"', {m(){return '"+text+"';},get x(){return '"+text+"';},set x(v){this.value=v+'"+text+"';}}];",items=Function(body)(),obj=items[2];obj.x='a';items[0]()===text && items[1]()===text && obj.m()===text && obj.x===text && obj.value==='a'+text && items[0].toString()==="function inner(){return '"+text+"';}" && items[1].toString()==="()=> '"+text+"'" && obj.m.toString()==="m(){return '"+text+"';}" && Object.getOwnPropertyDescriptor(obj,'x').get.toString()==="get x(){return '"+text+"';}""#,
    );
    check(
        r#"Function('/*\uD800*/return function clean(){return 7;}/*\uDFFF*/')().toString()==='function clean(){return 7;}'"#,
    );
}

#[test]
fn invalid_identifiers_throw_after_all_conversions_and_before_prototype_lookup() {
    check(
        r#"let log=[],target=(function(){}).bind(null);Object.defineProperty(target,'prototype',{get(){log.push('prototype');return {};}});let caught=false;try{Reflect.construct(Function,[{toString(){log.push('parameter');return '\uD800';}},{toString(){log.push('body');return 'return 7';}}],target);}catch(e){caught=e instanceof SyntaxError;}caught && log.join(',')==='parameter,body'"#,
    );
    for input in [
        r#"Function('return \uDFFF;')"#,
        r#"Function('x\uD800','return x;')"#,
        r#"Function('return 1\uD800;')"#,
        r#"Function('/*\uD800','*/){')"#,
        r#"Function('"use strict";return "\uD800" + 010;')"#,
    ] {
        check(&format!(
            "let caught=false,finalized=false;try{{{input};}}catch(e){{caught=e instanceof SyntaxError;}}finally{{finalized=true;}}caught && finalized"
        ));
    }
}

#[test]
fn retained_sources_and_literals_survive_collection_and_public_intrinsic_deletion() {
    let mut realm = Realm::default();
    realm.eval(r#"let text='\uD800',body="return function inner(){return '"+text+"';};",f=Function(body)();delete globalThis.Function;"#).unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r#"f()===text && f.toString()==="function inner(){return '"+text+"';}""#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn opt_in_source_bytes_count_lone_surrogates_and_pairs_exactly() {
    for (text, bytes) in [
        (r#"'\uD800'"#, 156),
        (r#"'\uD800\uDC00'"#, 196),
        (r#"'\uFFFD'"#, 156),
    ] {
        let source = format!(
            "try{{Function(\"return '\"+{text}.repeat(40)+\"';\");}}catch{{marker=1;}}finally{{marker=2;}}"
        );
        assert!(source.len() < bytes - 1);
        let mut limited = Realm::new(Limits {
            max_source_bytes: Some(bytes - 1),
            ..Limits::default()
        });
        limited.eval("var marker=0;").unwrap();
        assert!(
            matches!(limited.eval(&source), Err(Error::Limit { .. })),
            "{text}"
        );
        assert_eq!(limited.eval("marker"), Ok(Value::Number(0.0)));
        let mut exact = Realm::new(Limits {
            max_source_bytes: Some(bytes),
            ..Limits::default()
        });
        exact.eval("var marker=0;").unwrap();
        assert!(exact.eval(&source).is_ok(), "{text}");
        assert_eq!(exact.eval("marker"), Ok(Value::Number(2.0)));
    }
}
