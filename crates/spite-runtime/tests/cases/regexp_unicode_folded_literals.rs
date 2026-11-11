//! Unicode folded literal concatenations with original capture bounds.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn unicode_folded_literal_results_snapshot() {
    let mut rows = String::new();
    let inputs = [
        JsString::from("aaabſK😀ßS"),
        JsString::from("\u{10428}\u{10400}"),
        JsString::from_code_units(vec![0xd800, 0x41, 0xdc00, 0x61]),
        JsString::from(""),
    ];
    for body in [
        "(a)A(b)",
        "(())a(a)",
        "(ß)(s)()",
        r"(\u{10400})(\u{10428})",
        r"(\uD800)(a)",
        "((ſ)(K))",
        "(a)(A)",
        "(a)a(a)",
        "(a)()",
        "ab",
        "(?<x>a)(?<y>A)(?<z>b)",
    ] {
        let pattern = JsString::from(body);
        for flags in ["diug", "diumy", "divg", "divmy"] {
            for input in &inputs {
                for start in [0, 1, 2, input.len(), input.len() + 1] {
                    let script = format!(
                        "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_folded_literal_unicode_18_capture_widths_snapshot() {
    let mut rows = String::new();
    for body in ["(?<x>ß)(s)()", r"(?<x>\u{1df95})(s)()", "((ß)())s"] {
        for mode in ["u", "v"] {
            let pattern = JsString::from(body);
            for input in [JsString::from("ßS"), JsString::from("\u{1df95}ſ")] {
                for start in [0, 1] {
                    let script = format!(
                        "let r=new RegExp({pattern:?},'di{mode}g');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex}})"
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} mode={mode:?} input={input:?} start={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows, @r#"
        "(?<x>\u00df)(s)()" mode="u" input="\u00dfS" start=0 "{\u0022matches\u0022:[\u0022\u00dfS\u0022,\u0022\u00df\u0022,\u0022S\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00df\u0022},\u0022indices\u0022:[[0,2],[0,1],[1,2],[2,2]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:2}"
        "(?<x>\u00df)(s)()" mode="u" input="\u00dfS" start=1 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<x>\u00df)(s)()" mode="u" input="\ud837\udf95\u017f" start=0 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\u017f\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\ud837\udf95\u0022},\u0022indices\u0022:[[0,3],[0,2],[2,3],[3,3]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:3}"
        "(?<x>\u00df)(s)()" mode="u" input="\ud837\udf95\u017f" start=1 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\u017f\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\ud837\udf95\u0022},\u0022indices\u0022:[[0,3],[0,2],[2,3],[3,3]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:3}"
        "(?<x>\u00df)(s)()" mode="v" input="\u00dfS" start=0 "{\u0022matches\u0022:[\u0022\u00dfS\u0022,\u0022\u00df\u0022,\u0022S\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00df\u0022},\u0022indices\u0022:[[0,2],[0,1],[1,2],[2,2]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:2}"
        "(?<x>\u00df)(s)()" mode="v" input="\u00dfS" start=1 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<x>\u00df)(s)()" mode="v" input="\ud837\udf95\u017f" start=0 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\u017f\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\ud837\udf95\u0022},\u0022indices\u0022:[[0,3],[0,2],[2,3],[3,3]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:3}"
        "(?<x>\u00df)(s)()" mode="v" input="\ud837\udf95\u017f" start=1 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\u017f\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\ud837\udf95\u0022},\u0022indices\u0022:[[0,3],[0,2],[2,3],[3,3]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:3}"
        "(?<x>\u005cu{1df95})(s)()" mode="u" input="\u00dfS" start=0 "{\u0022matches\u0022:[\u0022\u00dfS\u0022,\u0022\u00df\u0022,\u0022S\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00df\u0022},\u0022indices\u0022:[[0,2],[0,1],[1,2],[2,2]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:2}"
        "(?<x>\u005cu{1df95})(s)()" mode="u" input="\u00dfS" start=1 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<x>\u005cu{1df95})(s)()" mode="u" input="\ud837\udf95\u017f" start=0 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\u017f\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\ud837\udf95\u0022},\u0022indices\u0022:[[0,3],[0,2],[2,3],[3,3]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:3}"
        "(?<x>\u005cu{1df95})(s)()" mode="u" input="\ud837\udf95\u017f" start=1 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\u017f\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\ud837\udf95\u0022},\u0022indices\u0022:[[0,3],[0,2],[2,3],[3,3]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:3}"
        "(?<x>\u005cu{1df95})(s)()" mode="v" input="\u00dfS" start=0 "{\u0022matches\u0022:[\u0022\u00dfS\u0022,\u0022\u00df\u0022,\u0022S\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\u00df\u0022},\u0022indices\u0022:[[0,2],[0,1],[1,2],[2,2]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,1]},\u0022lastIndex\u0022:2}"
        "(?<x>\u005cu{1df95})(s)()" mode="v" input="\u00dfS" start=1 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<x>\u005cu{1df95})(s)()" mode="v" input="\ud837\udf95\u017f" start=0 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\u017f\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\ud837\udf95\u0022},\u0022indices\u0022:[[0,3],[0,2],[2,3],[3,3]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:3}"
        "(?<x>\u005cu{1df95})(s)()" mode="v" input="\ud837\udf95\u017f" start=1 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\u017f\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022groups\u0022:{\u0022x\u0022:\u0022\ud837\udf95\u0022},\u0022indices\u0022:[[0,3],[0,2],[2,3],[3,3]],\u0022indexGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:3}"
        "((\u00df)())s" mode="u" input="\u00dfS" start=0 "{\u0022matches\u0022:[\u0022\u00dfS\u0022,\u0022\u00df\u0022,\u0022\u00df\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022indices\u0022:[[0,2],[0,1],[0,1],[1,1]],\u0022lastIndex\u0022:2}"
        "((\u00df)())s" mode="u" input="\u00dfS" start=1 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "((\u00df)())s" mode="u" input="\ud837\udf95\u017f" start=0 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\ud837\udf95\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022indices\u0022:[[0,3],[0,2],[0,2],[2,2]],\u0022lastIndex\u0022:3}"
        "((\u00df)())s" mode="u" input="\ud837\udf95\u017f" start=1 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\ud837\udf95\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022indices\u0022:[[0,3],[0,2],[0,2],[2,2]],\u0022lastIndex\u0022:3}"
        "((\u00df)())s" mode="v" input="\u00dfS" start=0 "{\u0022matches\u0022:[\u0022\u00dfS\u0022,\u0022\u00df\u0022,\u0022\u00df\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022indices\u0022:[[0,2],[0,1],[0,1],[1,1]],\u0022lastIndex\u0022:2}"
        "((\u00df)())s" mode="v" input="\u00dfS" start=1 "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "((\u00df)())s" mode="v" input="\ud837\udf95\u017f" start=0 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\ud837\udf95\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022indices\u0022:[[0,3],[0,2],[0,2],[2,2]],\u0022lastIndex\u0022:3}"
        "((\u00df)())s" mode="v" input="\ud837\udf95\u017f" start=1 "{\u0022matches\u0022:[\u0022\ud837\udf95\u017f\u0022,\u0022\ud837\udf95\u0022,\u0022\ud837\udf95\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022indices\u0022:[[0,3],[0,2],[0,2],[2,2]],\u0022lastIndex\u0022:3}"
        "#);
}

#[test]
fn unicode_folded_literal_prefix_retries_empty_captures_and_fold_distinctions() {
    assert_true(
        r"let m=/(a)A(b)/diu.exec('aaab');m[0]==='aab'&&m.index===1&&m[1]==='a'&&m[2]==='b'&&m.indices[1][0]===1&&m.indices[2][0]===3",
    );
    assert_true(
        r"let m=/(a())(A)/div.exec('aA');m.length===4&&m[1]==='a'&&m[2]===''&&m[3]==='A'&&m.indices[2][0]===1&&m.indices[2][1]===1",
    );
    assert_true(
        r"let m=/(())a(A)/diu.exec('aA');m[1]===''&&m[2]===''&&m[3]==='A'&&m.indices[1][0]===0&&m.indices[2][1]===0",
    );
    assert_true(r"let r=/(a)A(b)/iuy;r.lastIndex=0;r.exec('aaab')===null&&r.lastIndex===0");
    assert_true(
        r"let m=/(?<x>ſ)(?<y>K)/div.exec('SK');m.groups.x==='S'&&m.groups.y==='K'&&m.indices.groups.x===m.indices[1]&&m.indices.groups.y===m.indices[2]",
    );
    assert_true(
        r"/(ß)s/iu.test('ẞS')&&!/(ß)s/iv.test('sss')&&!/(İ)i/iu.test('ii')&&!/(ı)i/iv.test('Ii')",
    );
    assert_true(r"/(\u1fd3)(\ufb05)/iu.test('\u0390\ufb06')");
    assert_true(
        r"let m=/(\uD800)(a)/div.exec('\uD800A');m[1]==='\uD800'&&m[2]==='A'&&m.indices[2][0]===1&&!/(\uD800)(a)/iu.test('\uD800\uDC00a')",
    );
}

#[test]
fn unicode_folded_literal_consumers_named_groups_and_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r"let r=/(?<x>\u{{10400}})(?<y>s)()/di{mode}y;r.lastIndex=1;let m=r.exec('\u{{10428}}ſ');m[1]==='\u{{10428}}'&&m[2]==='ſ'&&m[3]===''&&m.index===0&&m.indices[1][1]===2&&m.indices[2][0]===2&&m.indices[3][0]===3&&m.indices.groups.x===m.indices[1]&&r.lastIndex===3"
        ));
        assert_true(&format!(
            r"let a=[...'ſKSK'.matchAll(/(?<x>s)(k)/di{mode}g)];a.length===2&&a[0][1]==='ſ'&&a[0][2]==='K'&&a[1][1]==='S'&&a[1].indices[2][1]===4"
        ));
        assert_true(&format!(
            r"'ſKSK'.replace(/(?<x>s)(k)/i{mode}g,'<$1:$2:$<x>>')==='<ſ:K:ſ><S:K:S>'"
        ));
        assert_true(&format!(
            r"'ſKSK'.split(/(s)(k)/i{mode}).join('|')==='|ſ|K||S|K|'"
        ));
        assert_true(&format!(
            r"let r=/(s)(k)/i{mode};r.lastIndex=8;'😀ſK'.search(r)===2&&r.lastIndex===8"
        ));
    }
}

#[test]
fn unicode_folded_literal_metadata_coercions_and_strict_writes() {
    assert_true(
        r"let r=/(?<x>s)(k)/dimv,c=new RegExp(r,'diuy');c.source==='(?<x>s)(k)'&&c.ignoreCase&&c.unicode&&!c.multiline&&c.exec('ſK').groups.x==='ſ'&&c.lastIndex===2",
    );
    assert_true(
        r"let t='',r=/(s)(k)/iuy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return 'ſK'}});t==='si'&&m[1]==='ſ'&&m[2]==='K'&&r.lastIndex===2",
    );
    assert_true(
        r"let r=/(s)(k)/iuy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ſK')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    assert_true(
        r"let r=/(s)(k)/ivy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_folded_literal_opted_work_and_additional_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(s)(k)/iug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/ab+/iu.exec('AB')",
        r"/(ab)+/iv.exec('AB')",
        r"/a|b/iu.exec('A')",
        r"/^(ab)+$/iv.exec('AB')",
        r"/[a]b/iu.exec('AB')",
        r"/(a)\1/iv.exec('AA')",
        r"/(?=ab)/iu.exec('AB')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn original_unicode_folded_literal_complete_programs() {
    let mut rows = String::new();
    for source in [r"/ab/iu.exec('AB')", r"/(a)b/iu.exec('AB')"] {
        let program = format!("JSON.stringify(eval({:?}))", JsString::from(source));
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{program}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "/ab/iu.exec('AB')" "[\u0022AB\u0022]"
        "/(a)b/iu.exec('AB')" "[\u0022AB\u0022,\u0022A\u0022]"
        "#);
}
