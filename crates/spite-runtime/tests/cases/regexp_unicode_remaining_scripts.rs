//! Grouped remaining-Script coverage against independently parsed pinned UCD files.
use spite_core::JsString;
use spite_heap::Error as HeapError;
use spite_runtime::{Error, Limits, Realm, Value, object::Error as ObjectError};
use std::{collections::BTreeMap, fmt::Write, sync::OnceLock};

const LIMIT: usize = 0x110000;
const LEGACY: [&str; 12] = [
    "Han",
    "Latin",
    "Greek",
    "Cyrillic",
    "Arabic",
    "Hebrew",
    "Common",
    "Inherited",
    "Devanagari",
    "Bengali",
    "Gurmukhi",
    "Gujarati",
];
const CASED: [&str; 7] = [
    "Adlam",
    "Cherokee",
    "Coptic",
    "Deseret",
    "Georgian",
    "Osage",
    "Warang_Citi",
];

struct ScriptReference {
    names: Vec<&'static str>,
    aliases: Vec<Vec<&'static str>>,
    owners: Vec<u16>,
    members: Vec<Option<u32>>,
}

impl ScriptReference {
    fn load() -> Self {
        let mut aliases = BTreeMap::new();
        for line in
            include_str!("../../../spite-core/src/testdata/PropertyValueAliases-18.0.0.txt").lines()
        {
            let fields: Vec<_> = line
                .split('#')
                .next()
                .unwrap()
                .split(';')
                .map(str::trim)
                .collect();
            if fields.first() != Some(&"sc") {
                continue;
            }
            assert!(fields.len() >= 3);
            let mut names = fields[1..].to_vec();
            names.sort_unstable();
            names.dedup();
            assert!(aliases.insert(fields[2], names).is_none());
        }
        let names: Vec<_> = aliases.keys().copied().collect();
        assert_eq!(names.len(), 179);
        assert_eq!(aliases.values().map(Vec::len).sum::<usize>(), 351);
        let unknown = names.binary_search(&"Unknown").unwrap();
        let mut owners = vec![u16::try_from(unknown).unwrap(); LIMIT];
        let mut explicit = vec![false; LIMIT];
        let scripts = include_str!("../../../spite-core/src/testdata/Scripts-18.0.0.txt");
        assert!(scripts.contains("# @missing: 0000..10FFFF; Unknown"));
        for line in scripts.lines() {
            let fields: Vec<_> = line
                .split('#')
                .next()
                .unwrap()
                .split(';')
                .map(str::trim)
                .collect();
            if fields.len() < 2 {
                continue;
            }
            let owner = names.binary_search(&fields[1]).unwrap();
            let span: Vec<_> = fields[0]
                .split("..")
                .map(|point| usize::from_str_radix(point, 16).unwrap())
                .collect();
            let start = span[0];
            let end = *span.last().unwrap();
            assert!(start <= end && end < LIMIT);
            assert!(explicit[start..=end].iter().all(|&used| !used));
            explicit[start..=end].fill(true);
            owners[start..=end].fill(u16::try_from(owner).unwrap());
        }
        let mut members = vec![None; names.len()];
        for (point, &owner) in owners.iter().enumerate() {
            members[usize::from(owner)].get_or_insert(point as u32);
        }
        let aliases = names
            .iter()
            .map(|name| aliases.remove(name).unwrap())
            .collect();
        let result = Self {
            names,
            aliases,
            owners,
            members,
        };
        assert_eq!(result.members[result.index("Katakana_Or_Hiragana")], None);
        assert_eq!(result.members[unknown], Some(0x378));
        result
    }

    fn index(&self, name: &str) -> usize {
        self.names.binary_search(&name).unwrap()
    }

    fn nonmember(&self, index: usize) -> u32 {
        self.owners
            .iter()
            .position(|&owner| usize::from(owner) != index)
            .unwrap() as u32
    }

    fn fold_pair(&self, name: &str) -> (u32, u32) {
        let index = self.index(name);
        for line in include_str!("../../../spite-core/src/testdata/CaseFolding-18.0.0.txt").lines()
        {
            let fields: Vec<_> = line
                .split('#')
                .next()
                .unwrap()
                .split(';')
                .map(str::trim)
                .collect();
            if fields.len() < 3 || !matches!(fields[1], "C" | "S") {
                continue;
            }
            let source = u32::from_str_radix(fields[0], 16).unwrap();
            let targets: Vec<_> = fields[2].split_whitespace().collect();
            assert_eq!(targets.len(), 1);
            let target = u32::from_str_radix(targets[0], 16).unwrap();
            if usize::from(self.owners[source as usize]) == index {
                assert_eq!(usize::from(self.owners[target as usize]), index);
                return (source, target);
            }
        }
        panic!("no pinned simple/common fold for {name}")
    }
}

fn reference() -> &'static ScriptReference {
    static REFERENCE: OnceLock<ScriptReference> = OnceLock::new();
    REFERENCE.get_or_init(ScriptReference::load)
}

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn remaining_scripts_all_exact_aliases_positive_and_complement_modes() {
    let reference = reference();
    let mut scripts = 0;
    let mut keys = 0;
    for (index, &name) in reference.names.iter().enumerate() {
        if LEGACY.contains(&name) {
            continue;
        }
        scripts += 1;
        let names: Vec<_> = reference.aliases[index]
            .iter()
            .flat_map(|alias| [format!("Script={alias}"), format!("sc={alias}")])
            .collect();
        keys += names.len();
        let mut points = vec![reference.nonmember(index)];
        let mut raw = vec![false];
        if let Some(member) = reference.members[index] {
            points.push(member);
            raw.push(true);
        }
        for flags in ["u", "v"] {
            check(&format!(
                r#"let names={names:?},points={points:?},raw={raw:?},flags={flags:?};names.every(name=>{{let forms=["\\p{{"+name+"}}","\\P{{"+name+"}}"];return forms.every((body,k)=>{{let r=new RegExp('^'+body+'$',flags);return points.every((cp,j)=>r.test(String.fromCodePoint(cp))===(raw[j]!==[false,true][k]))}})}})"#,
            ));
        }
    }
    assert_eq!(scripts, 167);
    assert_eq!(keys, 652);
    assert_eq!(keys * 2 * 2, 2_608);
}

#[test]
fn remaining_scripts_unknown_includes_default_surrogates_private_use_and_noncharacters() {
    let reference = reference();
    let index = reference.index("Unknown");
    let points = [0x378, 0xd800, 0xdfff, 0xe000, 0xf0000, 0x10fffd, 0x10ffff];
    assert!(
        points
            .iter()
            .all(|&point| usize::from(reference.owners[point]) == index)
    );
    for flags in ["u", "ui", "v", "vi"] {
        check(&format!(
            r"let points={points:?};points.every(cp=>{{let s=String.fromCodePoint(cp);return /^\p{{sc=Zzzz}}$/{flags}.test(s)&&!/^\P{{Script=Unknown}}$/{flags}.test(s)&&/^[^\P{{sc=Unknown}}]$/{flags}.test(s)&&!/^[^\p{{Script=Zzzz}}]$/{flags}.test(s)}})&&!/^\p{{sc=Unknown}}$/{flags}.test('A')&&!/^\p{{sc=Unknown}}$/{flags}.test('😀')"
        ));
        check(&format!(
            r"/^\p{{sc=Unknown}}$/{flags}.test('\uE000')&&!/^\p{{Cn}}$/{flags}.test('\uE000')&&/^\p{{sc=Unknown}}$/{flags}.test('\uD800')&&!/^\p{{Cn}}$/{flags}.test('\uD800')"
        ));
    }
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{sc=Zzzz}})/d{mode}g,m=r.exec('😀\uD800\uE000\u0378');m.index===2&&m[0]==='\uD800'&&m.indices.groups.x[0]===2&&m.indices.groups.x[1]===3&&r.lastIndex===3&&r.exec('😀\uD800\uE000\u0378')[0]==='\uE000'"
        ));
    }
}

#[test]
fn remaining_scripts_empty_hrkt_is_an_empty_character_set() {
    for mode in ["u", "v", "ui", "vi"] {
        check(&format!(
            r"let r=/\p{{sc=Hrkt}}/{mode}g;r.lastIndex=1;r.exec('あア😀\uD800')===null&&r.lastIndex===0&&'あア😀\uD800'.match(/\p{{Script=Katakana_Or_Hiragana}}/{mode}g)===null&&'あア😀\uD800'.replace(/\p{{sc=Hrkt}}/{mode}g,'x')==='あア😀\uD800'"
        ));
        check(&format!(
            r"let a=[...'あア😀\uD800'.matchAll(/(?<x>\P{{sc=Hrkt}})/d{mode}g)];a.length===4&&a[0][1]==='あ'&&a[1][1]==='ア'&&a[2][1]==='😀'&&a[2].index===2&&a[2].indices[1][1]===4&&a[3][1]==='\uD800'&&a[3].index===4&&a[3].indices[1][1]===5&&/^[^\p{{sc=Hrkt}}]$/{mode}.test('😀')&&!/^[^\P{{sc=Hrkt}}]$/{mode}.test('😀')"
        ));
    }
}

#[test]
fn remaining_scripts_cased_captures_and_positive_consumers_keep_original_text() {
    for name in CASED {
        let (source, target) = reference().fold_pair(name);
        for flags in ["u", "ui", "v", "vi"] {
            check(&format!(
                r#"let a=String.fromCodePoint({source}),b=String.fromCodePoint({target}),w=a.length,text=a+b+'😀A',r=/(?<x>(\p{{sc={name}}}))/d{flags}g,m=r.exec(text),n=r.exec(text);m[0]===a&&m[1]===a&&m[2]===a&&m.index===0&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===w&&n[0]===b&&n[1]===b&&n.index===w&&n.indices[1][0]===w&&n.indices[1][1]===2*w&&r.lastIndex===2*w&&r.exec(text)===null&&r.lastIndex===0&&text.replace(/(?<x>\p{{Script={name}}})/{flags}g,'<$1:$<x>>')==='<'+a+':'+a+'><'+b+':'+b+'>😀A'&&('A'+a+b+'A').split(/(\p{{sc={name}}})/{flags}).join('|')==='A|'+a+'||'+b+'|A'&&('😀'+a).search(/\p{{sc={name}}}/{flags})===2&&text.match(/\p{{sc={name}}}/{flags}g).join('')===a+b"#,
            ));
        }
    }
}

#[test]
fn remaining_scripts_complement_and_outer_inversion_consumers_keep_widths() {
    for name in CASED {
        let (source, target) = reference().fold_pair(name);
        for flags in ["u", "ui", "v", "vi"] {
            check(&format!(
                r#"let a=String.fromCodePoint({source}),b=String.fromCodePoint({target}),w=a.length,text=a+'😀'+b+'A',p=[...text.matchAll(/(?<x>[^\P{{sc={name}}}])/d{flags}g)],n=[...text.matchAll(/(?<x>[^\p{{sc={name}}}])/d{flags}g)];p.length===2&&p[0][1]===a&&p[0].index===0&&p[1][1]===b&&p[1].index===w+2&&p[1].indices[1][1]===2*w+2&&n.length===2&&n[0][1]==='😀'&&n[0].index===w&&n[0].indices[1][1]===w+2&&n[1][1]==='A'&&n[1].index===2*w+2&&text.replace(/(\P{{sc={name}}})/{flags}g,'<$1>')===a+'<😀>'+b+'<A>'&&text.split(/(\P{{sc={name}}})/{flags}).join('|')===a+'|😀|'+b+'|A|'"#,
            ));
            check(&format!(
                r#"let a=String.fromCodePoint({source}),b=String.fromCodePoint({target}),t='',x=(a+'😀'+b).replace(/(?<x>\p{{sc={name}}})/{flags}g,(m,c,p)=>{{t+=p+':'+c+';';return '_'}});x==='_😀_'&&t==='0:'+a+';'+(a.length+2)+':'+b+';'"#,
            ));
        }
    }
}

#[test]
fn remaining_scripts_high_mask_words_mix_with_old_properties() {
    for (left, right) in [
        ("Adlam", "Cuneiform"),
        ("Cuneiform", "Cypriot"),
        ("Kawi", "Kayah_Li"),
        ("Multani", "Myanmar"),
        ("Rejang", "Runic"),
        ("Tifinagh", "Tirhuta"),
        ("Tirhuta", "Zanabazar_Square"),
    ] {
        let reference = reference();
        let a = reference.members[reference.index(left)].unwrap();
        let b = reference.members[reference.index(right)].unwrap();
        for flags in ["u", "ui", "v", "vi"] {
            check(&format!(
                r"let points=[65,55296,{a},{b},128512],want=[true,true,true,true,false],r=/^[\p{{ASCII}}\p{{Cs}}\p{{sc=Latn}}\p{{sc={left}}}\p{{sc={right}}}]$/{flags};points.every((cp,i)=>r.test(String.fromCodePoint(cp))===want[i])"
            ));
            for (name, member, other) in [(left, a, b), (right, b, a)] {
                check(&format!(
                    r"let points=[{member},{other},65,128512],want=[true,false,false,false],r=/^[^\P{{sc={name}}}]$/{flags};points.every((cp,i)=>r.test(String.fromCodePoint(cp))===want[i])"
                ));
            }
        }
    }
}

#[test]
fn remaining_scripts_pair_starts_context_and_original_capture_indices() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<__proto__>\p{{Script=Adlam}})/di{mode}y;r.lastIndex=1;let m=r.exec('\u{{1E900}}A');m.index===0&&m[0]==='\u{{1E900}}'&&m[1]==='\u{{1E900}}'&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\P{{sc=Adlam}}/d{mode}g;r.lastIndex=1;let m=r.exec('\u{{1E900}}\uD800');m.index===2&&m[0]==='\uD800'&&m.indices[0][1]===3&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\P{{sc=Adlam}}/{mode}y;r.lastIndex=1;r.exec('\u{{1E900}}A')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Coptic}}K]))$/dim{mode}g,m=r.exec('A\r\nⲀ\u2028');m.index===3&&m[1]==='Ⲁ'&&m[2]==='Ⲁ'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
    }
}

#[test]
fn remaining_scripts_clones_reflection_coercions_and_strict_last_index() {
    check(
        r"let r=/^(?<x>\p{sc=Qaac})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{sc=Qaac})$'&&c.flags==='divy'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('Ⲁ').groups.x==='Ⲁ'&&c.lastIndex===1&&r.lastIndex===0",
    );
    check(
        r"let t='',r=/\P{Script=Katakana_Or_Hiragana}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Cypriot}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u{10800}')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{sc=Unknown}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn remaining_scripts_qaac_and_qaai_retain_distinct_coptic_and_inherited_sets() {
    for flags in ["u", "ui", "v", "vi"] {
        check(&format!(
            r"/^\p{{sc=Qaac}}$/{flags}.test('Ⲁ')&&/^\p{{Script=Qaac}}$/{flags}.test('ⲁ')&&!/^\p{{sc=Qaac}}$/{flags}.test('\u0300')&&/^\p{{sc=Qaai}}$/{flags}.test('\u0300')&&/^\p{{Script=Qaai}}$/{flags}.test('\u0300')&&!/^\p{{sc=Qaai}}$/{flags}.test('Ⲁ')"
        ));
    }
}

#[test]
fn remaining_scripts_repetitions_keep_last_capture_and_outside_words() {
    for name in CASED {
        let (source, target) = reference().fold_pair(name);
        for mode in ["u", "v"] {
            check(&format!(
                r#"let a=String.fromCodePoint({source}),b=String.fromCodePoint({target}),w=a.length,r=/\B(?<x>\p{{sc={name}}})+\B/di{mode}g,m=r.exec('µ'+a+b+'µ');m.index===1&&m[0]===a+b&&m.groups.x===b&&m.indices.groups.x[0]===1+w&&m.indices.groups.x[1]===1+2*w&&r.lastIndex===1+2*w"#,
            ));
        }
    }
}

#[test]
fn remaining_scripts_repeated_high_aliases_and_clones_survive_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval(r"let p='^'+'('.repeat(128)+'['+'\\p{sc=Cypriot}\\p{Script=Cprt}\\p{sc=Zanb}\\p{Cs}'.repeat(256)+']'+')'.repeat(128)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='A'.repeat(10000)+'\n\u{10800}'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"let m=c.exec(text);m.length===129&&m[1]==='\u{10800}'&&m[128]==='\u{10800}'&&m.indices[128][0]===10001&&m.indices[128][1]===10003&&c.lastIndex===10003&&c.source===p"),
        Ok(Value::Boolean(true))
    );
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn remaining_scripts_regexp_handles_and_execution_stay_in_their_realm() {
    let mut first = Realm::default();
    let mut second = Realm::default();
    let value = first
        .eval(r"let r=/\p{sc=Cypriot}/dug;r.lastIndex=1;r")
        .unwrap();
    let root = first.root_value(value.clone(), 100).unwrap();
    assert!(matches!(
        second.root_value(value, 100),
        Err(ObjectError::Heap(HeapError::ForeignHandle))
    ));
    first.collect(usize::MAX).unwrap();
    assert_eq!(
        first.eval(r"let m=r.exec('\u{10800}A');m.index===0&&m[0]==='\u{10800}'&&r.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        second
            .eval(r"let r=/\p{sc=Cypriot}/vy;r.lastIndex===0&&r.exec('A')===null&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    drop(root);
    check(
        r#"let marker=0,result;try{result=eval("marker=1; /\\p{Script=Coptic}/u.test('Ⲁ');")}catch{marker=2}finally{marker=3}marker===3&&result===true"#,
    );
}

#[test]
fn remaining_scripts_opted_work_propagates_without_javascript_catch_or_finally() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\p{sc=Coptic}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn remaining_scripts_exact_names_and_unadmitted_set_operations() {
    for mode in ["u", "v"] {
        for name in [
            "Coptic",
            "Qaac",
            "Hrkt",
            "Script=coptic",
            "SCRIPT=Coptic",
            "sc=COPT",
            "gc=Coptic",
            "Script=OldItalic",
            "Script=Coptic ",
            "Script=Cóptic",
            "sc=Script=Coptic",
        ] {
            let pattern = JsString::from(format!(r"\p{{{name}}}").as_str());
            check(&format!(
                "let caught=false;try{{new RegExp({pattern:?},{mode:?})}}catch(e){{caught=e instanceof SyntaxError}}caught"
            ));
        }
        check(&format!(
            r"let m=/\p{{Script_Extensions=Coptic}}/{mode}.exec('Ⲁ');m.length===1&&m[0]==='Ⲁ'&&m.index===0&&m.input==='Ⲁ'"
        ));
        check(&format!(
            r"let m=/\p{{scx=Unknown}}/{mode}.exec('\u0378');m.length===1&&m[0]==='\u0378'&&m.index===0&&m.input==='\u0378'"
        ));
    }
    check(r"/[\p{sc=Coptic}&&A]/v.exec('A')===null");
    check(r"/[\p{sc=Unknown}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0xe000),m=/[\p{sc=Unknown}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
}

#[test]
fn remaining_scripts_selected_results_snapshot() {
    let mut rows = String::new();
    for (body, units, start) in [
        (
            r"(?<x>\p{sc=Qaac})",
            "AⲀ😀".encode_utf16().collect::<Vec<_>>(),
            0,
        ),
        (
            r"(?<x>[^\P{Script=Adlam}])",
            "😀\u{1e900}".encode_utf16().collect(),
            1,
        ),
        (
            r"(?<x>\p{sc=Cprt})",
            "\u{10800}A".encode_utf16().collect(),
            1,
        ),
        (r"(?<x>\p{sc=Zzzz})", vec![0xd83d, 0xde00, 0xd800, 0x378], 1),
        (r"(?<x>\p{sc=Hrkt})", vec![0x41], 0),
        (r"(?<x>\P{sc=Hrkt})", vec![0xd83d, 0xde00, 0xd800], 1),
    ] {
        let pattern = JsString::from(body);
        let input = JsString::from_code_units(units);
        for flags in ["dug", "diug", "divg", "divy"] {
            let script = format!(
                "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify({{m,index:m?.index,groups:m?.groups,indices:m?.indices,indexGroups:m?.indices?.groups,lastIndex:r.lastIndex,source:r.source}})"
            );
            let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                panic!("{script}")
            };
            writeln!(
                rows,
                "{body:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
            )
            .unwrap();
        }
    }
    insta::assert_snapshot!(rows);
}
