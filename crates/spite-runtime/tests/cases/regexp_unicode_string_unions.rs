//! Adjacent ClassStringDisjunction operands in one positive class (22.2.2).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn string_unions_share_longest_member_priority_across_operand_boundaries() {
    check(
        r"let a=/[\q{a}\q{ab|abc}]/dv.exec('xabc!'),b=/[\q{abc}\q{a|ab}]/v.exec('ab!');a.index===1 && a[0]==='abc' && a.indices[0][1]===4 && b[0]==='ab'",
    );
    check(
        r"let a=/[\q{}\q{ab|a}]/v.exec('ab'),b=/[\q{ab|a}\q{}]/v.exec('xab');a[0]==='ab' && b.index===0 && b[0]==='' && /[\q{}\q{}]/v.exec('')[0]==='' && !/[\q{ab}\q{cd}]/v.test('ac')",
    );
    check(
        r"let a=/[\q{ab|a!}\q{a|}]\b/dv.exec('a!'),b=/[\q{a!}\q{}]\b/dv.exec('a!');a[0]==='a' && a.indices[0][1]===1 && b[0]==='' && b.indices[0][1]===0 && /[\q{a!}\q{a}]\B/v.exec('a!')[0]==='a!'",
    );
    check(
        r"/[\q{a\}}\q{\{b}]/v.exec('a}')[0]==='a}' && /[\q{a\}}\q{\{b}]/v.exec('{b')[0]==='{b' && /[\q{ab}\q{a\|b}]/v.exec('a|b')[0]==='a|b'",
    );
}

#[test]
fn union_folding_assertions_surrogates_and_captures_use_original_context() {
    check(
        r"let r=/(?<whole>([\q{😀}\q{😀x|🚀}]))/dvy;r.lastIndex=2;let m=r.exec('A😀x!');m.index===1 && m[0]==='😀x' && m[1]==='😀x' && m[2]==='😀x' && m.groups.whole==='😀x' && m.indices[0][0]===1 && m.indices[2][1]===4 && m.indices.groups.whole===m.indices[1] && r.lastIndex===4",
    );
    check(
        r"/[\q{sK}\q{k|ßa}]/iv.exec('ſK')[0]==='ſK' && /[\q{sK}\q{k|ßa}]/iv.exec('K')[0]==='K' && /[\q{sK}\q{k|ßa}]/iv.exec('ẞA')[0]==='ẞA' && !/[\q{ßa}\q{x}]/iv.test('SSa') && !/[\q{\u{D800}\u{DC00}}\q{ab}]/v.test('𐀀')",
    );
    check(
        r"let m=/^\b[\q{a!}\q{a|}]\b$/mv.exec('x\r\na\u2028z');m.index===3 && m[0]==='a' && !/^\b[\q{a!}\q{a|}]\b$/v.test('x\r\na\u2028z')",
    );
    check(
        r"let r=/(?<whole>([\q{}\q{}]))/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m[1]==='' && m[2]==='' && m.groups.whole==='' && m.indices[2][0]===0 && m.indices[2][1]===0 && r.lastIndex===0",
    );
}

#[test]
fn union_results_equal_one_disjunction_and_string_consumers_advance_empty_matches() {
    check(
        r"let pairs=[['[\\q{a}\\q{ab|abc}]','[\\q{a|ab|abc}]'],['[\\q{}\\q{ab|a}]','[\\q{|ab|a}]'],['[\\q{sK}\\q{k|ßa}]','[\\q{sK|k|ßa}]'],['[\\q{😀}\\q{😀x|🚀}]','[\\q{😀|😀x|🚀}]']],ok=true;for(let p of pairs)for(let flags of ['dvg','divg','dvgy','divgmy'])for(let text of ['','xabc!','A!','ſK','ẞA','A😀x🚀'])for(let start=0;start<=text.length+1;start++){let a=new RegExp('(?<whole>('+p[0]+'))',flags),b=new RegExp('(?<whole>('+p[1]+'))',flags);a.lastIndex=start;b.lastIndex=start;let x=a.exec(text),y=b.exec(text);ok=ok && JSON.stringify(x)===JSON.stringify(y) && (x===null || (x.index===y.index && JSON.stringify(x.indices)===JSON.stringify(y.indices) && x.groups.whole===y.groups.whole)) && a.lastIndex===b.lastIndex;}ok",
    );
    check(
        r"let r=/(?<whole>([\q{}\q{ab|a}]))/dvg,a=[...'ab😀'.matchAll(r)];a.length===3 && a[0][0]==='ab' && a[1].index===2 && a[1][0]==='' && a[2].index===4 && a[2].indices[2][1]===4 && r.lastIndex===0 && 'ab😀'.replace(r,'#')==='##😀#' && 'ab😀'.split(/[\q{}\q{ab|a}]/v).join('|')==='|😀'",
    );
    check(
        r"let r=/[\q{ab}\q{a}]/vg;r.lastIndex=9;'xab!'.search(r)===1 && r.lastIndex===9 && 'ab-a'.match(r).join('|')==='ab|a' && r.lastIndex===0 && new RegExp(r,'ivy').test('AB')",
    );
}

#[test]
fn union_proof_keeps_other_operands_and_repetition_pending_and_cancels_without_mutation() {
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[\\q{ab}\\q{cd}]+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4],[true,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for source in [
        r"/[[\q{ab}\q{cd}]&&\p{RGI_Emoji}]+/v.test('ab')",
        r"/[\q{ab}\q{cd}]x/v.test('abx')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([\q{}\q{ab|a}]))/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('zab').groups.whole==='ab' && r.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let source = format!(
        r"try{{r=/[\q{{}}{}]/vg;}}catch{{marker=1;}}finally{{marker=2;}}",
        r"\q{ab}".repeat(4000)
    );
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("r===7 && marker===0"), Ok(Value::Boolean(true)));
}
