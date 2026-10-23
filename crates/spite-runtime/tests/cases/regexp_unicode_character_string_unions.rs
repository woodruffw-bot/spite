//! Ordinary characters beside variable ClassStringDisjunction operands.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn ordinary_characters_and_strings_share_priority_in_every_operand_order() {
    check(
        r"let a=/[a\q{ab|abc}]/dv.exec('xabc!'),b=/[\q{abc|ab}a]/v.exec('a!');a.index===1 && a[0]==='abc' && a.indices[0][1]===4 && b[0]==='a' && /[x\q{ab}a\q{abc}\q{}z]/v.exec('abc')[0]==='abc' && /[x\q{ab}a\q{abc}\q{}z]/v.exec('!')[0]===''",
    );
    check(
        r"let a=/[\q{a!}a]\b/dv.exec('a!'),b=/[\q{a!}a]\B/v.exec('a!');a[0]==='a' && a.indices[0][1]===1 && b[0]==='a!' && /[a\q{}]/v.exec('a')[0]==='a' && /[a\q{}]/v.exec('!')[0]==='' && !/[a\q{ab}]/v.test('b')",
    );
    check(
        r"let m=/^\b[k\q{sK}\q{ßa}]\b$/imv.exec('x\r\nſK\u2028z');m.index===3 && m[0]==='ſK' && /[k\q{sK}]/iv.exec('K')[0]==='K' && /[\q{ßa}x]/iv.exec('ẞA')[0]==='ẞA' && !/[\q{ßa}x]/iv.test('SSa')",
    );
    check(
        r"let m=/(?<whole>([a\q{ab|abc}]))/dvg.exec('xabc!');m.length===3 && m[0]==='abc' && m[1]==='abc' && m[2]==='abc' && m.groups.whole==='abc' && m.indices[0][0]===1 && m.indices[2][1]===4 && m.indices.groups.whole===m.indices[1]",
    );
}

#[test]
fn literal_escapes_surrogates_and_constructor_lines_keep_original_ranges() {
    check(
        r"let r=/[😀\q{😀x}🚀]/dvy;r.lastIndex=2;let m=r.exec('A😀x!');m.index===1 && m[0]==='😀x' && m.indices[0][0]===1 && m.indices[0][1]===4 && r.lastIndex===4 && /[\q{ab}\uD83D\uDE00]/v.exec('😀')[0]==='😀'",
    );
    check(
        r"let a=/[\q{\u{D800}\u{DC00}}\u{D800}\u{DC00}]/v.exec('\uD800a');a[0].length===1 && a[0].charCodeAt(0)===0xd800 && !/[\q{\u{D800}\u{DC00}}\u{D800}\u{DC00}]/v.test('𐀀')",
    );
    check(
        r"/[\q{a\|b}\|\}\]]/v.exec('a|b')[0]==='a|b' && /[\q{a\|b}\|\}\]]/v.exec('|')[0]==='|' && /[\q{a\|b}\|\}\]]/v.exec('}')[0]==='}' && /[\q{ab}\b]/v.exec('\b')[0]==='\b' && /[\q{ab}\0]/v.exec('\0')[0]==='\0'",
    );
    check(
        r"new RegExp('[x\\q{a\n}]','v').exec('a\n')[0]==='a\n' && new RegExp('[\\q{a\n}x]','v').exec('x')[0]==='x'",
    );
}

#[test]
fn string_consumers_and_merged_q_members_keep_names_indices_and_empty_advancement() {
    check(
        r"let r=/(?<whole>([\q{}a\q{ab}]))/dvg,a=[...'ab😀'.matchAll(r)];a.length===3 && a[0][0]==='ab' && a[1].index===2 && a[1][0]==='' && a[2].index===4 && a[2].groups.whole==='' && a[2].indices[2][1]===4 && r.lastIndex===0 && 'ab😀'.replace(r,'#')==='##😀#' && 'ab😀'.split(/[\q{}a\q{ab}]/v).join('|')==='|😀'",
    );
    check(
        r"let r=/[a\q{ab}]/vg;r.lastIndex=9;'xab!'.search(r)===1 && r.lastIndex===9 && 'ab-a'.match(r).join('|')==='ab|a' && r.lastIndex===0 && new RegExp(r,'ivy').test('AB')",
    );
    check(
        r"let pairs=[['[a\\q{ab|abc}]','[\\q{a|ab|abc}]'],['[\\q{a!}a]\\b','[\\q{a!|a}]\\b'],['[\\q{}a\\q{ab}]','[\\q{|a|ab}]'],['[k\\q{sK}\\q{ßa}]','[\\q{k|sK|ßa}]'],['[😀\\q{😀x}🚀]','[\\q{😀|😀x|🚀}]']],ok=true;for(let p of pairs)for(let flags of ['dvg','divg','dvgy','divgmy'])for(let text of ['','xabc!','A!','ſK','ẞA','A😀x🚀'])for(let start=0;start<=text.length+1;start++){let a=new RegExp('(?<whole>('+p[0]+'))',flags),b=new RegExp('(?<whole>('+p[1]+'))',flags);a.lastIndex=start;b.lastIndex=start;let x=a.exec(text),y=b.exec(text);ok=ok && JSON.stringify(x)===JSON.stringify(y) && (x===null || (x.index===y.index && JSON.stringify(x.indices)===JSON.stringify(y.indices) && x.groups.whole===y.groups.whole)) && a.lastIndex===b.lastIndex;}ok",
    );
}

#[test]
fn former_controls_complete_finalizers_and_unproved_algebra_remains_unsupported() {
    for source in [r"/[\q{ab}a]/v.test('ab')", r"/[\q{ab}a\q{cd}]/v.test('ab')"] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2;}}finally{{marker=3;}}"
            )),
            Ok(Value::Boolean(true))
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)));
    }
    for source in [
        r"/[a-z\q{ab}]/v.test('ab')",
        r"/[\q{ab}\d]/v.test('ab')",
        r"/[a&&\q{ab}]/v.test('ab')",
        r"/[\q{ab}&&a]/v.test('ab')",
        r"/[\q{ab}--a]/v.test('ab')",
        r"/[\q{ab}[a]]/v.test('ab')",
        r"/[a\q{ab}]+/v.test('ab')",
        r"/[a\q{ab}]x/v.test('abx')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn mixed_compiled_storage_survives_collection_and_opt_in_cancellation_is_atomic() {
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([\q{}a\q{ab}]))/dvg;r.lastIndex=1")
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
        r"try{{r=/[\q{{}}{}\q{{ab}}]/vg;}}catch{{marker=1;}}finally{{marker=2;}}",
        "a".repeat(20_000)
    );
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("r===7 && marker===0"), Ok(Value::Boolean(true)));
}
