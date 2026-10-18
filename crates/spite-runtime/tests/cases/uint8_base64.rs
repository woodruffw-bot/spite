//! Uint8Array Base64 alphabets, options, bounded decoding, and encoding (23.3).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

// Independent standard-library/RFC 4648 golden encoding of bytes 0 through 255.
const ALL_BYTES: &str = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0+P0BBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWltcXV5fYGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn+AgYKDhIWGh4iJiouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq+wsbKztLW2t7i5uru8vb6/wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t/g4eLj5OXm5+jp6uvs7e7v8PHy8/T19vf4+fr7/P3+/w==";

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn exception(source: &str, expected: ExceptionKind) {
    let result = Realm::default().eval(source);
    assert!(
        matches!(&result,Err(Error::Exception {kind,..}) if *kind==expected),
        "{source}: {result:?}"
    );
}

#[test]
fn both_alphabets_encode_and_decode_every_byte_against_an_independent_golden() {
    let url = ALL_BYTES.replace('+', "-").replace('/', "_");
    check(&format!(
        "let standard='{ALL_BYTES}',url='{url}',a=new Uint8Array(256),b=Uint8Array.fromBase64(standard),c=Uint8Array.fromBase64(url,{{alphabet:'base64url',lastChunkHandling:'strict'}}),valid=b.length===256 && c.length===256;for(let i=0;i<256;i++){{a[i]=i;valid=valid && b[i]===i && c[i]===i;}}valid && a.toBase64()===standard && a.toBase64({{alphabet:'base64url'}})===url && a.toBase64({{omitPadding:true}})===standard.slice(0,-2) && a.toBase64({{alphabet:'base64url',omitPadding:true}})===url.slice(0,-2)"
    ));
    for (text, encoded) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        let bytes = text
            .bytes()
            .map(|byte| byte.to_string())
            .collect::<Vec<_>>()
            .join(",");
        check(&format!(
            "let a=new Uint8Array([{bytes}]),b=Uint8Array.fromBase64('{encoded}',{{lastChunkHandling:'strict'}});a.toBase64()==='{encoded}' && b.join()==='{bytes}'"
        ));
    }
}

#[test]
fn mode_padding_capacity_and_utf16_read_matrix_uses_hand_derived_original_input_boundaries() {
    let mut rows = Vec::new();
    for (label, input, mode, capacity) in [
        ("empty", "''", "loose", 2),
        ("white", r#"'\t\n\f\r '"#, "loose", 1),
        ("white", r#"'\t\n\f\r '"#, "loose", 0),
        ("Z", "'Z'", "loose", 2),
        ("Z", "'Z'", "strict", 2),
        ("Z", "'Z'", "stop-before-partial", 2),
        ("Zg", "'Zg'", "loose", 2),
        ("Zg", "'Zg'", "strict", 2),
        ("Zg", "'Zg'", "stop-before-partial", 2),
        ("Zm8", "'Zm8'", "loose", 2),
        ("Zm8", "'Zm8'", "strict", 2),
        ("Zm8", "'Zm8'", "stop-before-partial", 2),
        ("Zg==", "'Zg=='", "strict", 1),
        ("Zh==", "'Zh=='", "strict", 2),
        ("Zh==", "'Zh=='", "loose", 2),
        ("Zh==", "'Zh=='", "stop-before-partial", 2),
        ("Zm9=", "'Zm9='", "strict", 2),
        ("Zm9=", "'Zm9='", "loose", 2),
        ("Zg=", "'Zg='", "loose", 2),
        ("Zg=", "'Zg='", "stop-before-partial", 2),
        ("Zg==x", "'Zg==x'", "loose", 1),
        ("Zg==x", "'Zg==x'", "loose", 0),
        ("Zm9v!", "'Zm9v!'", "loose", 3),
        ("Zm9v!", "'Zm9v!'", "loose", 4),
        ("TWFu", "'TWFu'", "loose", 1),
        ("TWFu", "'TWFu'", "loose", 2),
        ("TQ==", "'TQ=='", "strict", 1),
        ("TW!", "'TW!'", "loose", 1),
        ("TWA", "'TWA'", "loose", 1),
        ("TWF!", "'TWF!'", "loose", 2),
        ("TWFA", "'TWFA'", "loose", 2),
        ("partial after full", "' Zm9v Zg'", "stop-before-partial", 5),
        ("partial after full", "' Zm9v Zg'", "loose", 5),
        ("partial after full", "' Zm9v Zg'", "strict", 5),
        ("space after full", "' Zm9v '", "loose", 3),
        ("space after full", "' Zm9v '", "loose", 4),
        ("spaced padding", "'Z g = ='", "strict", 1),
        ("ASCII white", r#"'\tZ\nm\f9\rv '"#, "strict", 3),
        ("ASCII white", r#"'\tZ\nm\f9\rv '"#, "strict", 4),
        ("surrogate suffix", r#"'Zm9v\uD800\uDC00'"#, "loose", 3),
        ("surrogate suffix", r#"'Zm9v\uD800\uDC00'"#, "loose", 4),
    ] {
        let value=Realm::default().eval(&format!(
            "let a=new Uint8Array({capacity}),status='ok',read='-',written='-';a.fill(170);try{{let r=a.setFromBase64({input},{{lastChunkHandling:'{mode}'}});read=r.read;written=r.written;}}catch(e){{if(!(e instanceof SyntaxError))throw e;status='syntax';}}status+' '+read+'/'+written+' ['+a.join()+']'"
        )).unwrap();
        let Value::String(output) = value else {
            panic!("{label}: {value:?}")
        };
        rows.push(format!(
            "{label}/{mode}/cap{capacity}: {}",
            output.to_utf8().unwrap()
        ));
    }
    insta::assert_snapshot!(rows.join("\n"), @r###"
    empty/loose/cap2: ok 0/0 [170,170]
    white/loose/cap1: ok 5/0 [170]
    white/loose/cap0: ok 0/0 []
    Z/loose/cap2: syntax -/- [170,170]
    Z/strict/cap2: syntax -/- [170,170]
    Z/stop-before-partial/cap2: ok 0/0 [170,170]
    Zg/loose/cap2: ok 2/1 [102,170]
    Zg/strict/cap2: syntax -/- [170,170]
    Zg/stop-before-partial/cap2: ok 0/0 [170,170]
    Zm8/loose/cap2: ok 3/2 [102,111]
    Zm8/strict/cap2: syntax -/- [170,170]
    Zm8/stop-before-partial/cap2: ok 0/0 [170,170]
    Zg==/strict/cap1: ok 4/1 [102]
    Zh==/strict/cap2: syntax -/- [170,170]
    Zh==/loose/cap2: ok 4/1 [102,170]
    Zh==/stop-before-partial/cap2: ok 4/1 [102,170]
    Zm9=/strict/cap2: syntax -/- [170,170]
    Zm9=/loose/cap2: ok 4/2 [102,111]
    Zg=/loose/cap2: syntax -/- [170,170]
    Zg=/stop-before-partial/cap2: ok 0/0 [170,170]
    Zg==x/loose/cap1: syntax -/- [170]
    Zg==x/loose/cap0: ok 0/0 []
    Zm9v!/loose/cap3: ok 4/3 [102,111,111]
    Zm9v!/loose/cap4: syntax -/- [102,111,111,170]
    TWFu/loose/cap1: ok 0/0 [170]
    TWFu/loose/cap2: ok 0/0 [170,170]
    TQ==/strict/cap1: ok 4/1 [77]
    TW!/loose/cap1: syntax -/- [170]
    TWA/loose/cap1: ok 0/0 [170]
    TWF!/loose/cap2: syntax -/- [170,170]
    TWFA/loose/cap2: ok 0/0 [170,170]
    partial after full/stop-before-partial/cap5: ok 5/3 [102,111,111,170,170]
    partial after full/loose/cap5: ok 8/4 [102,111,111,102,170]
    partial after full/strict/cap5: syntax -/- [102,111,111,170,170]
    space after full/loose/cap3: ok 5/3 [102,111,111]
    space after full/loose/cap4: ok 6/3 [102,111,111,170]
    spaced padding/strict/cap1: ok 7/1 [102]
    ASCII white/strict/cap3: ok 8/3 [102,111,111]
    ASCII white/strict/cap4: ok 9/3 [102,111,111,170]
    surrogate suffix/loose/cap3: ok 4/3 [102,111,111]
    surrogate suffix/loose/cap4: syntax -/- [102,111,111,170]
    "###);
}

#[test]
fn alphabets_are_exact_and_never_mixed_or_automatically_detected() {
    check(
        "Uint8Array.of(255).toBase64()==='/w==' && Uint8Array.of(255).toBase64({alphabet:'base64url'})==='_w==' && Uint8Array.of(251,255,255).toBase64()==='+///' && Uint8Array.of(251,255,255).toBase64({alphabet:'base64url'})==='-___'",
    );
    for (input, alphabet) in [
        ("'_w=='", "base64"),
        ("'-___'", "base64"),
        ("'/w=='", "base64url"),
        ("'+///'", "base64url"),
        ("'AA-/'", "base64"),
        ("'AA-/'", "base64url"),
    ] {
        exception(
            &format!("Uint8Array.fromBase64({input},{{alphabet:'{alphabet}'}})"),
            ExceptionKind::SyntaxError,
        );
    }
    check(
        "Uint8Array.fromBase64('_w',{alphabet:'base64url'}).join()==='255' && Uint8Array.fromBase64('/w').join()==='255'",
    );
    check(
        "Uint8Array.of(255,255).toBase64()==='//8=' && Uint8Array.of(255,255).toBase64({omitPadding:true})==='//8' && Uint8Array.of(255,255).toBase64({alphabet:'base64url',omitPadding:true})==='__8' && new Uint8Array(0).toBase64({omitPadding:true})===''",
    );
}

#[test]
fn only_five_ascii_whitespace_units_are_skipped_and_padding_suffixes_are_fully_checked() {
    check(
        r#"Uint8Array.fromBase64('\tZ\nm\f9\rv ').join()==='102,111,111' && Uint8Array.fromBase64('Z g = = \t\r\n',{lastChunkHandling:'strict'})[0]===102"#,
    );
    for invalid in [
        r#"'\v'"#,
        r#"'\u00A0'"#,
        r#"'\uFEFF'"#,
        r#"'\u2028'"#,
        r#"'\u2029'"#,
        r#"'\u0000'"#,
        r#"'\uD800'"#,
        r#"'\uDC00'"#,
        r#"'\uD800\uDC00'"#,
        r#"'\uFF21'"#,
    ] {
        exception(
            &format!("Uint8Array.fromBase64('Zg=='+{invalid})"),
            ExceptionKind::SyntaxError,
        );
        exception(
            &format!("Uint8Array.fromBase64({invalid}+'Zg==')"),
            ExceptionKind::SyntaxError,
        );
    }
    for input in [
        "'='",
        "'Z='",
        "'Zg==='",
        "'Zm8=='",
        "'Zg=x'",
        "'Zg==Zg=='",
        "'Zg==x'",
    ] {
        for mode in ["loose", "strict", "stop-before-partial"] {
            exception(
                &format!("Uint8Array.fromBase64({input},{{lastChunkHandling:'{mode}'}})"),
                ExceptionKind::SyntaxError,
            );
        }
    }
}

#[test]
fn string_type_checks_precede_options_and_never_invoke_coercion() {
    for input in [
        "undefined",
        "null",
        "true",
        "7",
        "1n",
        "Symbol()",
        "Object('Zg==')",
        "new String('Zg==')",
        "[]",
        "{}",
    ] {
        for call in ["Uint8Array.fromBase64", "a.setFromBase64"] {
            check(&format!(
                "let a=new Uint8Array(1),hits=0,caught=false;try{{{call}({input},{{get alphabet(){{hits++;throw 7;}},get lastChunkHandling(){{hits++;throw 8;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0 && a[0]===0"
            ));
        }
    }
    check(
        "let hits=0,caught=false,input={get [Symbol.toPrimitive](){hits++;throw 7;},valueOf(){hits++;throw 8;},toString(){hits++;throw 9;}};try{Uint8Array.fromBase64(input);}catch(e){caught=e instanceof TypeError;}caught && hits===0",
    );
}

#[test]
fn exact_uint8_brand_precedes_options_and_subclasses_keep_the_private_kind() {
    for receiver in [
        "null",
        "{}",
        "Uint8Array.prototype",
        "Object.create(new Uint8Array(1))",
        "new Int8Array(1)",
        "new Uint8ClampedArray(1)",
        "new Uint16Array(1)",
        "new BigUint64Array(1)",
        "new DataView(new ArrayBuffer(1))",
    ] {
        for method in ["setFromBase64", "toBase64"] {
            let args = if method == "setFromBase64" {
                "'Zg==',options"
            } else {
                "options"
            };
            check(&format!(
                "let hits=0,caught=false,options={{get alphabet(){{hits++;throw 7;}}}};try{{Uint8Array.prototype.{method}.call({receiver},{args});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
            ));
        }
    }
    check(
        "class S extends Uint8Array{}let a=new S(1),r=a.setFromBase64('_w==',{alphabet:'base64url'});r.read===4 && r.written===1 && a instanceof S && a.toBase64({alphabet:'base64url'})==='_w=='",
    );
}

#[test]
fn static_receiver_subclasses_global_bindings_and_species_are_ignored() {
    check(
        "let A=Uint8Array,from=A.fromBase64,prototype=A.prototype;class S extends A{constructor(){throw 7;}}Object.defineProperty(S,Symbol.species,{get(){throw 8;}});let inherited=S.fromBase64('Zg=='),receiver={get prototype(){throw 9;},get [Symbol.species](){throw 10;}};Uint8Array=function(){throw 11;};let a=from.call(receiver,'/w=='),b=from.call(null,''),c=from.call(Symbol(),'_w==',{alphabet:'base64url'});Object.getPrototypeOf(inherited)===prototype && !(inherited instanceof S) && inherited[0]===102 && Object.getPrototypeOf(a)===prototype && a[0]===255 && a.byteOffset===0 && a.buffer.byteLength===1 && !a.buffer.resizable && b.length===0 && b.buffer!==a.buffer && c[0]===255 && Object.getPrototypeOf(c)===prototype",
    );
}

#[test]
fn absent_or_undefined_options_use_a_null_prototype_object_while_explicit_options_inherit() {
    check(
        "Object.defineProperty(Object.prototype,'alphabet',{get(){throw 7;},configurable:true});Object.defineProperty(Object.prototype,'lastChunkHandling',{get(){throw 8;},configurable:true});Object.defineProperty(Object.prototype,'omitPadding',{get(){throw 9;},configurable:true});let a=new Uint8Array(1),r=a.setFromBase64('Zg=='),s=a.setFromBase64('Zg==',undefined);r.read===4 && s.written===1 && a.toBase64()==='Zg==' && a.toBase64(undefined)==='Zg==' && Uint8Array.fromBase64('Zg==')[0]===102 && Uint8Array.fromBase64('Zg==',undefined)[0]===102",
    );
    check(
        "let log='',prototype={get alphabet(){log+='a';return 'base64url';},get lastChunkHandling(){log+='l';return 'strict';},get omitPadding(){throw 7;}},options=Object.create(prototype);let a=Uint8Array.fromBase64('_w==',options),b=new Uint8Array(1),r=b.setFromBase64('_w==',options);log==='alal' && a[0]===255 && b[0]===255 && r.read===4",
    );
    check(
        "let log='',options=Object.create({get alphabet(){log+='a';return 'base64url';},get omitPadding(){log+='o';return true;},get lastChunkHandling(){throw 7;}});Uint8Array.of(255).toBase64(options)==='_w' && log==='ao'",
    );
}

#[test]
fn options_must_be_objects_and_keyword_values_are_exact_primitive_strings() {
    for options in ["null", "true", "0", "'base64'", "1n", "Symbol()"] {
        exception(
            &format!("Uint8Array.fromBase64('Zg==',{options})"),
            ExceptionKind::TypeError,
        );
        exception(
            &format!("new Uint8Array(1).setFromBase64('Zg==',{options})"),
            ExceptionKind::TypeError,
        );
        exception(
            &format!("new Uint8Array(1).toBase64({options})"),
            ExceptionKind::TypeError,
        );
    }
    for value in [
        "null",
        "7",
        "true",
        "'BASE64'",
        "Object('base64')",
        "Symbol()",
        "{toString(){throw 7;},valueOf(){throw 8;}}",
    ] {
        for call in [
            "Uint8Array.fromBase64('Zg==',options)",
            "a.setFromBase64('Zg==',options)",
            "a.toBase64(options)",
        ] {
            check(&format!(
                "let a=new Uint8Array(1),later=0,caught=false,options={{alphabet:{value},get lastChunkHandling(){{later++;throw 9;}},get omitPadding(){{later++;throw 10;}}}};try{{{call};}}catch(e){{caught=e instanceof TypeError;}}caught && later===0"
            ));
        }
    }
    for value in [
        "null",
        "7",
        "true",
        "'LOOSE'",
        "Object('loose')",
        "Symbol()",
        "{toString(){throw 7;},valueOf(){throw 8;}}",
    ] {
        exception(
            &format!("Uint8Array.fromBase64('Zg==',{{lastChunkHandling:{value}}})"),
            ExceptionKind::TypeError,
        );
    }
    check(
        "function options(){}options.alphabet='base64url';options.lastChunkHandling='strict';Uint8Array.fromBase64('_w==',options)[0]===255 && Uint8Array.fromBase64('Zg==',[])[0]===102 && Uint8Array.fromBase64('Zg==',Object('options'))[0]===102",
    );
}

#[test]
fn option_getters_run_in_order_before_live_witnesses_even_after_detachment() {
    check(
        "let a=new Uint8Array(0),log='',options={get alphabet(){log+='a';return 'base64';},get lastChunkHandling(){log+='l';return 'strict';},get omitPadding(){log+='o';return true;}},s=Uint8Array.fromBase64('',options),r=a.setFromBase64('!',options),encoded=a.toBase64(options);log==='alalao' && s.length===0 && r.read===0 && r.written===0 && encoded===''",
    );
    exception(
        "Uint8Array.fromBase64('',{alphabet:'invalid'})",
        ExceptionKind::TypeError,
    );
    exception(
        "new Uint8Array(0).setFromBase64('!',{lastChunkHandling:'invalid'})",
        ExceptionKind::TypeError,
    );
    for method in ["setFromBase64", "toBase64"] {
        let call = if method == "setFromBase64" {
            "a.setFromBase64('Zg==',options)"
        } else {
            "a.toBase64(options)"
        };
        let later = if method == "setFromBase64" {
            "lastChunkHandling"
        } else {
            "omitPadding"
        };
        let result = if method == "setFromBase64" {
            "'loose'"
        } else {
            "false"
        };
        check(&format!(
            "let a=new Uint8Array(1),log='',caught=false;a.buffer.transfer();let options={{get alphabet(){{log+='a';return 'base64';}},get {later}(){{log+='l';return {result};}}}};try{{{call};}}catch(e){{caught=e instanceof TypeError;}}caught && log==='al'"
        ));
        assert_eq!(Realm::default().eval(&format!(
            "let a=new Uint8Array(1);a.buffer.transfer();let options={{get alphabet(){{return 'base64';}},get {later}(){{throw 7;}}}};{call}"
        )),Err(Error::Thrown(Value::Number(7.0))));
        check(&format!(
            "let a=new Uint8Array(1),log='',caught=false,options={{get alphabet(){{log+='a';a.buffer.transfer();return 'base64';}},get {later}(){{log+='l';return {result};}}}};try{{{call};}}catch(e){{caught=e instanceof TypeError;}}caught && log==='al'"
        ));
    }
}

#[test]
fn getters_can_restore_fixed_views_or_change_auto_capacity_before_the_witness() {
    check(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(buffer,0,4),log='';buffer.resize(1);let r=a.setFromBase64('AQIDBA==',{get alphabet(){log+='a';return 'base64';},get lastChunkHandling(){log+='l';buffer.resize(4);return 'strict';}});log==='al' && r.read===8 && r.written===4 && a.join()==='1,2,3,4'",
    );
    check(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(buffer,0,4);a[0]=1;buffer.resize(1);let result=a.toBase64({get alphabet(){return 'base64';},get omitPadding(){buffer.resize(4);a[1]=2;a[2]=3;a[3]=4;return false;}});result==='AQIDBA==' && a.length===4",
    );
    check(
        "let buffer=new ArrayBuffer(1,{maxByteLength:4}),a=new Uint8Array(buffer);let r=a.setFromBase64('TWFu',{get lastChunkHandling(){buffer.resize(3);return 'loose';}});r.read===4 && r.written===3 && a.join()==='77,97,110'",
    );
    check(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(buffer);a[0]=170;a[1]=171;let r=a.setFromBase64('TWFu',{get lastChunkHandling(){buffer.resize(2);return 'loose';}});r.read===0 && r.written===0 && a.join()==='170,171'",
    );
    exception(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(buffer,0,4);a.setFromBase64('',{get lastChunkHandling(){buffer.resize(1);return 'loose';}})",
        ExceptionKind::TypeError,
    );
}

#[test]
fn omit_padding_uses_boolean_conversion_without_coercion_and_ignores_decoder_only_fields() {
    for (value, expected) in [
        ("undefined", "/w=="),
        ("null", "/w=="),
        ("false", "/w=="),
        ("0", "/w=="),
        ("''", "/w=="),
        ("true", "/w"),
        ("1", "/w"),
        ("'false'", "/w"),
        ("Symbol()", "/w"),
        ("1n", "/w"),
        ("Object(false)", "/w"),
    ] {
        check(&format!(
            "Uint8Array.of(255).toBase64({{omitPadding:{value},get lastChunkHandling(){{throw 7;}}}})==='{expected}'"
        ));
    }
    check(
        "let hits=0,value={get [Symbol.toPrimitive](){hits++;throw 7;},toString(){hits++;throw 8;},valueOf(){hits++;throw 9;}},options={alphabet:'base64url',omitPadding:value};Uint8Array.of(255).toBase64(options)==='_w' && hits===0",
    );
    check(
        "let options={get omitPadding(){throw 7;}};Uint8Array.fromBase64('Zg==',options)[0]===102 && new Uint8Array(1).setFromBase64('Zg==',options).written===1",
    );
}

#[test]
fn options_and_encoding_use_internal_state_and_never_public_index_or_string_helpers() {
    check(
        "let a=Uint8Array.of(255,0,1);for(let key of ['length','buffer','byteLength','byteOffset','BYTES_PER_ELEMENT','constructor',Symbol.iterator,Symbol.toStringTag])Object.defineProperty(a,key,{get(){throw 7;}});Number.prototype.toString=function(){throw 8;};String.prototype.trim=function(){throw 9;};Object.defineProperty(String.prototype,'charCodeAt',{get(){throw 10;}});let r=a.setFromBase64('AP8B');r.read===4 && r.written===3 && a[0]===0 && a[1]===255 && a[2]===1 && a.toBase64()==='AP8B' && Uint8Array.fromBase64(' /w== ')[0]===255",
    );
}

#[test]
fn syntax_errors_commit_only_complete_prefixes_and_capacity_can_ignore_a_malformed_suffix() {
    check(
        "let bytes=Uint8Array.of(90,91,170,170,170,170,170,170,92),a=new Uint8Array(bytes.buffer,2,6),caught=false;try{a.setFromBase64('Zm9vZg==x');}catch(e){caught=e instanceof SyntaxError;}caught && bytes.join()==='90,91,102,111,111,170,170,170,92'",
    );
    check(
        "let a=new Uint8Array(3),r=a.setFromBase64('Zm9vZg==x');r.read===4 && r.written===3 && a.join()==='102,111,111'",
    );
    check(
        "let a=Uint8Array.of(170,171),caught=false;try{a.setFromBase64('Zg==x');}catch(e){caught=e instanceof SyntaxError;}caught && a.join()==='170,171'",
    );
}

#[test]
fn setter_results_have_the_ordered_plain_data_shape_and_ignore_inherited_setters() {
    check(
        "let hits=0;Object.defineProperty(Object.prototype,'read',{set(){hits++;throw 7;},configurable:true});Object.defineProperty(Object.prototype,'written',{set(){hits++;throw 8;},configurable:true});let a=new Uint8Array(2),r=a.setFromBase64('Zg=='),read=Object.getOwnPropertyDescriptor(r,'read'),written=Object.getOwnPropertyDescriptor(r,'written');hits===0 && Object.getPrototypeOf(r)===Object.prototype && Reflect.ownKeys(r).join()==='read,written' && read.value===4 && read.writable && read.enumerable && read.configurable && !('get' in read) && !('set' in read) && written.value===1 && written.writable && written.enumerable && written.configurable && !('get' in written) && !('set' in written) && a[0]===102 && a[1]===0",
    );
}

#[test]
fn offset_rab_and_empty_view_state_is_checked_after_options_and_preserves_guards() {
    check(
        "let buffer=new ArrayBuffer(8,{maxByteLength:12}),bytes=new Uint8Array(buffer),a=new Uint8Array(buffer,2,3),auto=new Uint8Array(buffer,2);bytes[0]=90;bytes[1]=91;bytes[5]=92;let r=a.setFromBase64('AP8B');let initial=r.read===4 && r.written===3 && a.toBase64()==='AP8B' && auto.toBase64()==='AP8BXAAA';buffer.resize(4);let current=auto.toBase64()==='AP8=';buffer.resize(8);initial && current && a.toBase64()==='AP8A' && bytes[0]===90 && bytes[1]===91 && bytes[5]===0",
    );
    for method in ["setFromBase64", "toBase64"] {
        let arguments = if method == "setFromBase64" { "''" } else { "" };
        exception(
            &format!("let a=new Uint8Array(0);a.buffer.transfer();a.{method}({arguments})"),
            ExceptionKind::TypeError,
        );
        exception(
            &format!(
                "let buffer=new ArrayBuffer(4,{{maxByteLength:8}}),a=new Uint8Array(buffer,1,3);buffer.resize(2);a.{method}({arguments})"
            ),
            ExceptionKind::TypeError,
        );
    }
    check(
        "let a=new Uint8Array(0),r=a.setFromBase64('!');r.read===0 && r.written===0 && a.toBase64()===''",
    );
}

#[test]
fn sparse_page_and_safe_integer_offsets_remain_bounded_and_default_outputs_are_unlimited() {
    check(
        "let buffer=new ArrayBuffer(4102),a=new Uint8Array(buffer,4094,6),bytes=new Uint8Array(buffer);bytes[4093]=90;bytes[4100]=91;let r=a.setFromBase64('AAECAwQF');r.read===8 && r.written===6 && a.toBase64()==='AAECAwQF' && bytes[4093]===90 && bytes[4100]===91",
    );
    check(
        "let maximum=9007199254740991,buffer=new ArrayBuffer(maximum),a=new Uint8Array(buffer,maximum-3,3),guard=new DataView(buffer,maximum-4,1),r=a.setFromBase64('//8B');r.read===4 && r.written===3 && a.toBase64()==='//8B' && a.toBase64({alphabet:'base64url'})==='__8B' && a.byteOffset===maximum-3 && guard.getUint8(0)===0",
    );
    check(
        "let a=new Uint8Array(10000),encoded=a.toBase64(),copy=Uint8Array.fromBase64(encoded);encoded.length===13336 && copy.length===10000 && copy[0]===0 && copy[9999]===0",
    );
}

#[test]
fn decoded_intrinsic_results_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let result=(()=>{let input='AP8B',from=Uint8Array.fromBase64;return from.call({},input);})()").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("result.toBase64()==='AP8B' && result.length===3 && !result.buffer.resizable"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn decoded_byte_lists_are_not_call_arguments() {
    let mut realm = Realm::new(Limits {
        max_arguments: Some(1),
        ..Limits::default()
    });
    assert_eq!(realm.eval(&format!("let text='{ALL_BYTES}',a=Uint8Array.fromBase64(text),b=new Uint8Array(256),r=b.setFromBase64(text);r.read===344 && r.written===256 && a.toBase64()===text && b.toBase64()===text")),Ok(Value::Boolean(true)));
}

#[test]
fn explicit_work_abort_preserves_source_and_skips_language_cleanup_without_setup_allowances() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5_000),
        ..Limits::default()
    });
    realm.eval("let a=new Uint8Array(100000),flag=0").unwrap();
    realm.eval("a[0]=7").unwrap();
    realm.eval("a[99999]=9").unwrap();
    assert!(matches!(
        realm.eval("try{a.toBase64();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && a.length===100000 && a[0]===7 && a[99999]===9"),
        Ok(Value::Boolean(true))
    );
}
