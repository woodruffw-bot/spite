//! Script_Extensions from independent pinned aliases, Script fallback and overrides.
use spite_core::JsString;
use spite_heap::Error as HeapError;
use spite_runtime::{Error, Limits, Realm, Value, object::Error as ObjectError};
use std::{collections::BTreeMap, fmt::Write, sync::OnceLock};

const LIMIT: usize = 0x110000;

struct ExtensionRange {
    first: usize,
    last: usize,
    values: Vec<usize>,
}

struct ExtensionReference {
    names: Vec<&'static str>,
    aliases: Vec<Vec<&'static str>>,
    owners: Vec<u16>,
    overrides: Vec<ExtensionRange>,
    members: Vec<Option<u32>>,
    exclusive: Vec<Option<u32>>,
    folds: Vec<(u32, u32)>,
}

impl ExtensionReference {
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
        let aliases: Vec<_> = names
            .iter()
            .map(|name| aliases.remove(name).unwrap())
            .collect();
        let mut by_alias = BTreeMap::new();
        for (index, list) in aliases.iter().enumerate() {
            for &alias in list {
                assert!(by_alias.insert(alias, index).is_none());
            }
        }
        let unknown = names.binary_search(&"Unknown").unwrap();
        let mut owners = vec![u16::try_from(unknown).unwrap(); LIMIT];
        let mut assigned = vec![false; LIMIT];
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
            let span: Vec<_> = fields[0]
                .split("..")
                .map(|cp| usize::from_str_radix(cp, 16).unwrap())
                .collect();
            let first = span[0];
            let last = *span.last().unwrap();
            assert!(first <= last && last < LIMIT);
            assert!(assigned[first..=last].iter().all(|&value| !value));
            let owner = names.binary_search(&fields[1]).unwrap();
            assigned[first..=last].fill(true);
            owners[first..=last].fill(u16::try_from(owner).unwrap());
        }
        let source = include_str!("../../../spite-core/src/testdata/ScriptExtensions-18.0.0.txt");
        assert!(source.contains("# @missing: 0000..10FFFF; <script>"));
        let mut overrides = Vec::new();
        for line in source.lines() {
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
            assert_eq!(fields.len(), 2);
            let span: Vec<_> = fields[0]
                .split("..")
                .map(|cp| usize::from_str_radix(cp, 16).unwrap())
                .collect();
            let first = span[0];
            let last = *span.last().unwrap();
            assert!(first <= last && last < LIMIT);
            let mut values: Vec<_> = fields[1]
                .split_whitespace()
                .map(|alias| by_alias[alias])
                .collect();
            assert!(!values.is_empty());
            values.sort_unstable();
            assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
            overrides.push(ExtensionRange {
                first,
                last,
                values,
            });
        }
        overrides.sort_unstable_by_key(|range| range.first);
        assert!(
            overrides
                .windows(2)
                .all(|pair| pair[0].last < pair[1].first)
        );
        assert_eq!(overrides.len(), 210);
        assert_eq!(
            overrides
                .iter()
                .map(|range| range.last - range.first + 1)
                .sum::<usize>(),
            741
        );
        let mut members = vec![None; names.len()];
        let mut exclusive = vec![None; names.len()];
        let mut current = 0;
        let mut memberships = 0;
        // One raw-data scan establishes witnesses for every value. No matcher is
        // run over the domain, and no per-alias/full-domain matrix is allocated.
        for (point, &owner) in owners.iter().enumerate() {
            while current < overrides.len() && overrides[current].last < point {
                current += 1;
            }
            if let Some(range) = overrides.get(current).filter(|range| range.first <= point) {
                for &value in &range.values {
                    members[value].get_or_insert(point as u32);
                }
                memberships += range.values.len();
            } else {
                let owner = usize::from(owner);
                members[owner].get_or_insert(point as u32);
                exclusive[owner].get_or_insert(point as u32);
                memberships += 1;
            }
        }
        assert_eq!(memberships, 1_115_175);
        assert_eq!(
            members[names.binary_search(&"Katakana_Or_Hiragana").unwrap()],
            None
        );
        assert_eq!(members[unknown], Some(0x378));
        let mut folds = Vec::new();
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
            let targets: Vec<_> = fields[2].split_whitespace().collect();
            assert_eq!(targets.len(), 1);
            folds.push((
                u32::from_str_radix(fields[0], 16).unwrap(),
                u32::from_str_radix(targets[0], 16).unwrap(),
            ));
        }
        assert_eq!(folds.len(), 1533);
        assert!(folds.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(folds.iter().all(|&(_, target)| {
            folds
                .binary_search_by_key(&target, |&(source, _)| source)
                .is_err()
        }));
        Self {
            names,
            aliases,
            owners,
            overrides,
            members,
            exclusive,
            folds,
        }
    }

    fn index(&self, name: &str) -> usize {
        self.names.binary_search(&name).unwrap()
    }

    fn contains(&self, value: usize, point: u32) -> bool {
        let point = point as usize;
        let next = self.overrides.partition_point(|range| range.first <= point);
        if let Some(range) = next
            .checked_sub(1)
            .and_then(|index| self.overrides.get(index))
            .filter(|range| point <= range.last)
        {
            range.values.binary_search(&value).is_ok()
        } else {
            usize::from(self.owners[point]) == value
        }
    }

    fn nonmember(&self, value: usize) -> u32 {
        [0, 0x41]
            .into_iter()
            .find(|&point| !self.contains(value, point))
            .unwrap()
    }

    fn folded(&self, value: usize, point: u32, negative: bool, sets: bool) -> bool {
        let canonical = self
            .folds
            .binary_search_by_key(&point, |&(source, _)| source)
            .map_or(point, |index| self.folds[index].1);
        let mut inside = self.contains(value, canonical);
        let mut outside = !inside;
        for &(source, target) in &self.folds {
            if target == canonical {
                inside |= self.contains(value, source);
                outside |= !self.contains(value, source);
            }
        }
        if negative {
            if sets { !inside } else { outside }
        } else {
            inside
        }
    }
}

fn reference() -> &'static ExtensionReference {
    static REFERENCE: OnceLock<ExtensionReference> = OnceLock::new();
    REFERENCE.get_or_init(ExtensionReference::load)
}

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn script_extensions_all_exact_aliases_positive_and_complement_modes() {
    let reference = reference();
    let mut keys = 0;
    for (index, aliases) in reference.aliases.iter().enumerate() {
        let names: Vec<_> = aliases
            .iter()
            .flat_map(|alias| [format!("Script_Extensions={alias}"), format!("scx={alias}")])
            .collect();
        keys += names.len();
        let mut points = vec![reference.nonmember(index)];
        let mut raw = vec![false];
        if let Some(member) = reference.members[index] {
            assert!(reference.contains(index, member));
            points.push(member);
            raw.push(true);
        }
        for flags in ["u", "v"] {
            check(&format!(
                r#"let names={names:?},points={points:?},raw={raw:?},flags={flags:?};names.every(name=>{{let forms=["\\p{{"+name+"}}","\\P{{"+name+"}}"];return forms.every((body,k)=>{{let r=new RegExp('^'+body+'$',flags);return points.every((cp,j)=>r.test(String.fromCodePoint(cp))===(raw[j]!==[false,true][k]))}})}})"#,
            ));
        }
    }
    assert_eq!(keys, 702);
    assert_eq!(keys * 2 * 2, 2_808);
}

#[test]
fn script_extensions_overrides_replace_script_fallback_and_overlap() {
    let reference = reference();
    for (point, yes, no) in [
        (0x30fc, "Hiragana", "Common"),
        (0x300, "Latin", "Inherited"),
        (0x345, "Greek", "Inherited"),
        (0x12550, "Cuneiform", "Common"),
    ] {
        assert!(reference.contains(reference.index(yes), point));
        assert!(!reference.contains(reference.index(no), point));
    }
    for mode in ["u", "v"] {
        check(&format!(
            r"/\p{{sc=Common}}/{mode}.test('ー')&&/\p{{scx=Hira}}/{mode}.test('ー')&&/\p{{scx=Kana}}/{mode}.test('ー')&&!/\p{{scx=Zyyy}}/{mode}.test('ー')&&/\P{{scx=Common}}/{mode}.test('ー')"
        ));
        check(&format!(
            r"/\p{{sc=Inherited}}/{mode}.test('\u0300')&&/\p{{scx=Latin}}/{mode}.test('\u0300')&&/\p{{scx=Greek}}/{mode}.test('\u0300')&&/\p{{scx=Coptic}}/{mode}.test('\u0300')&&!/\p{{scx=Inherited}}/{mode}.test('\u0300')&&/\p{{scx=Grek}}/{mode}.test('\u0345')&&!/\p{{scx=Zinh}}/{mode}.test('\u0345')"
        ));
        check(&format!(
            r"/\p{{scx=Proto_Cuneiform}}/{mode}.test('\u{{12550}}')&&/\p{{scx=Cuneiform}}/{mode}.test('\u{{12550}}')&&/\p{{sc=Cuneiform}}/{mode}.test('\u{{12550}}')&&!/\p{{sc=Proto_Cuneiform}}/{mode}.test('\u{{12550}}')&&/\p{{scx=Han}}/{mode}.test('一')&&/\p{{scx=Latin}}/{mode}.test('A')&&!/\p{{scx=Greek}}/{mode}.test('A')"
        ));
    }
}

#[test]
fn script_extensions_extra_aliases_follow_extension_membership() {
    for flags in ["u", "ui", "v", "vi"] {
        check(&format!(
            r"/\p{{scx=Qaac}}/{flags}.test('Ⲁ')&&/\p{{Script_Extensions=Qaac}}/{flags}.test('\u0300')&&!/\p{{sc=Qaac}}/{flags}.test('\u0300')&&/\p{{scx=Qaai}}/{flags}.test('\u034F')&&!/\p{{Script_Extensions=Qaai}}/{flags}.test('\u0300')&&!/\p{{scx=Qaai}}/{flags}.test('\u0345')&&/\P{{scx=Qaai}}/{flags}.test('\u0300')"
        ));
    }
}

#[test]
fn script_extensions_casefold_images_preimages_and_outer_complements() {
    let reference = reference();
    let points = [
        0xb5, 0x39c, 0x3bc, 0x345, 0x3b9, 0x300, 0x30fc, 0x41, 0x378, 0xd800,
    ];
    for name in ["Common", "Greek", "Inherited"] {
        let index = reference.index(name);
        for (flags, sets) in [("ui", false), ("vi", true)] {
            let positive: Vec<_> = points
                .iter()
                .map(|&point| reference.folded(index, point, false, sets))
                .collect();
            let negative: Vec<_> = points
                .iter()
                .map(|&point| reference.folded(index, point, true, sets))
                .collect();
            check(&format!(
                r#"let points={points:?},p={positive:?},n={negative:?},forms=["^\\p{{scx={name}}}$","^\\P{{scx={name}}}$","^[\\p{{scx={name}}}]$","^[\\P{{scx={name}}}]$","^[^\\p{{scx={name}}}]$","^[^\\P{{scx={name}}}]$"],want=[p,n,p,n,p.map(x=>!x),n.map(x=>!x)];forms.every((body,k)=>{{let r=new RegExp(body,{flags:?});return points.every((cp,j)=>r.test(String.fromCodePoint(cp))===want[k][j])}})"#,
            ));
        }
    }
    check(
        r"/\P{sc=Greek}/ui.test('ι')&&!/\P{scx=Greek}/ui.test('ι')&&!/\P{scx=Greek}/vi.test('ι')",
    );
    check(
        r"/[\P{scx=Greek}A]/ui.test('µ')&&!/[\P{scx=Greek}A]/vi.test('µ')&&/[\P{scx=Common}A]/ui.test('μ')&&!/[\P{scx=Common}A]/vi.test('μ')",
    );
    check(
        r"!/[\P{scx=Greek}\p{scx=Inherited}]/vi.test('ι')&&/[\P{scx=Greek}\p{sc=Inherited}]/vi.test('ι')&&/[\P{scx=Greek}\p{Lu}]/vi.test('μ')&&/[\P{scx=Common}\p{sc=Greek}]/vi.test('µ')",
    );
}

#[test]
fn script_extensions_unknown_keeps_missing_surrogates_private_use_and_noncharacters() {
    let reference = reference();
    let index = reference.index("Unknown");
    let points = [0x378, 0xd800, 0xdfff, 0xe000, 0xf0000, 0x10fffd, 0x10ffff];
    assert!(points.iter().all(|&point| reference.contains(index, point)));
    for flags in ["u", "ui", "v", "vi"] {
        check(&format!(
            r"let points={points:?};points.every(cp=>{{let s=String.fromCodePoint(cp);return /^\p{{scx=Zzzz}}$/{flags}.test(s)&&!/^\P{{Script_Extensions=Unknown}}$/{flags}.test(s)&&/^[^\P{{scx=Unknown}}]$/{flags}.test(s)}})&&!/^\p{{scx=Unknown}}$/{flags}.test('A')&&!/^\p{{scx=Unknown}}$/{flags}.test('😀')"
        ));
        check(&format!(
            r"/^\p{{scx=Unknown}}$/{flags}.test('\uE000')&&!/^\p{{Cn}}$/{flags}.test('\uE000')&&/^\p{{scx=Unknown}}$/{flags}.test('\uD800')&&!/^\p{{Cn}}$/{flags}.test('\uD800')"
        ));
    }
}

#[test]
fn script_extensions_empty_hrkt_is_not_the_hiragana_katakana_union() {
    for flags in ["u", "ui", "v", "vi"] {
        check(&format!(
            r"let r=/\p{{scx=Hrkt}}/{flags}g;r.lastIndex=1;r.exec('あアー😀\uD800')===null&&r.lastIndex===0&&'あアー😀\uD800'.replace(/\p{{Script_Extensions=Katakana_Or_Hiragana}}/{flags}g,'x')==='あアー😀\uD800'&&/\p{{scx=Hira}}/{flags}.test('ー')&&/\p{{scx=Kana}}/{flags}.test('ー')"
        ));
        check(&format!(
            r"let a=[...'あアー😀\uD800'.matchAll(/(?<x>\P{{scx=Hrkt}})/d{flags}g)];a.length===5&&a[2][1]==='ー'&&a[3][1]==='😀'&&a[3].index===3&&a[3].indices[1][1]===5&&a[4][1]==='\uD800'&&a[4].index===5&&a[4].indices[1][1]===6&&/^[^\p{{scx=Hrkt}}]$/{flags}.test('😀')&&!/^[^\P{{scx=Hrkt}}]$/{flags}.test('😀')"
        ));
    }
}

#[test]
fn script_extensions_cased_captures_repetitions_keep_original_text() {
    for (name, source, target) in [
        ("Coptic", 0x2c80, 0x2c81),
        ("Deseret", 0x10400, 0x10428),
        ("Georgian", 0x1c90, 0x10d0),
    ] {
        let reference = reference();
        let index = reference.index(name);
        assert!(reference.contains(index, source) && reference.contains(index, target));
        assert!(reference.folds.contains(&(source, target)));
        for mode in ["u", "v"] {
            check(&format!(
                r"let a=String.fromCodePoint({source}),b=String.fromCodePoint({target}),w=a.length,r=/\B(?<x>\p{{scx={name}}})+\B/di{mode}g,m=r.exec('µ'+a+b+'µ');m.index===1&&m[0]===a+b&&m.groups.x===b&&m.indices.groups.x[0]===1+w&&m.indices.groups.x[1]===1+2*w&&r.lastIndex===1+2*w"
            ));
        }
    }
}

#[test]
fn script_extensions_capture_indices_and_pair_starts_keep_utf16_widths() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<__proto__>\p{{scx=Dsrt}})/di{mode}y;r.lastIndex=1;let m=r.exec('\u{{10400}}A');m.index===0&&m[0]==='\u{{10400}}'&&m[1]==='\u{{10400}}'&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\P{{scx=Deseret}}/d{mode}g;r.lastIndex=1;let m=r.exec('\u{{10400}}\uD800');m.index===2&&m[0]==='\uD800'&&m.indices[0][1]===3&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\P{{scx=Deseret}}/{mode}y;r.lastIndex=1;r.exec('\u{{10400}}A')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/^(?<x>([\p{{scx=Hira}}K]))$/dim{mode}g,m=r.exec('A\r\nー\u2028');m.index===3&&m[1]==='ー'&&m[2]==='ー'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
    }
}

#[test]
fn script_extensions_consumers_preserve_shared_marks_and_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'あー😀アA'.matchAll(/(?<x>\p{{scx=Hira}})/d{mode}g)];a.length===2&&a[0][1]==='あ'&&a[1][1]==='ー'&&a[1].index===1&&a[1].indices[1][1]===2&&'あー😀アA'.replace(/(?<x>\p{{scx=Hira}})/{mode}g,'<$1:$<x>>')==='<あ:あ><ー:ー>😀アA'"
        ));
        check(&format!(
            r"'AあーA'.split(/(\p{{scx=Hira}})/{mode}).join('|')==='A|あ||ー|A'&&'A😀ー'.search(/\p{{scx=Hira}}/{mode})===3&&'あーア'.match(/\p{{scx=Hira}}/{mode}g).join('')==='あー'"
        ));
        check(&format!(
            r"let t='',x='あーAあ'.replace(/(\p{{scx=Hira}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__A_'&&t==='013'"
        ));
    }
}

#[test]
fn script_extensions_clones_source_coercions_and_readonly_last_index() {
    check(
        r"let r=/^(?<x>\p{Script_Extensions=Hira})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Script_Extensions=Hira})$'&&c.flags==='divy'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('ー').groups.x==='ー'&&c.lastIndex===1&&r.lastIndex===0",
    );
    check(
        r"let t='',r=/\P{scx=Hira}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{scx=Hira}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ー')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{scx=Deseret}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u{10400}A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn script_extensions_high_mask_words_and_negative_aliases_keep_every_set() {
    let reference = reference();
    for (left, right) in [
        ("Buhid", "Canadian_Aboriginal"),
        ("Gurung_Khema", "Han"),
        ("Lisu", "Lycian"),
        ("Old_Hungarian", "Old_Italic"),
        ("Soyombo", "Sundanese"),
        ("Yi", "Zanabazar_Square"),
    ] {
        let left_index = reference.index(left);
        let right_index = reference.index(right);
        let a = reference.exclusive[left_index].unwrap();
        let b = reference.exclusive[right_index].unwrap();
        assert!(reference.contains(left_index, a) && !reference.contains(right_index, a));
        assert!(reference.contains(right_index, b) && !reference.contains(left_index, b));
        for flags in ["u", "ui", "v", "vi"] {
            check(&format!(
                r"let points=[{a},{b},65,55296,128512],want=[true,true,true,true,false],r=/^[\p{{scx={left}}}\p{{Script_Extensions={right}}}\p{{sc=Latn}}\p{{Cs}}]$/{flags};points.every((cp,i)=>r.test(String.fromCodePoint(cp))===want[i])"
            ));
            for (name, member, other) in [(left, a, b), (right, b, a)] {
                check(&format!(
                    r"let points=[{member},{other},65,128512],want=[true,false,false,false],r=/^[^\P{{scx={name}}}]$/{flags};points.every((cp,i)=>r.test(String.fromCodePoint(cp))===want[i])"
                ));
            }
        }
    }
}

#[test]
fn script_extensions_repeated_aliases_collection_and_realms_keep_native_handles() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut first = Realm::default();
    let mut second = Realm::default();
    let value = first.eval(r"let p='^'+'('.repeat(128)+'['+'\\p{scx=Zanb}\\p{Script_Extensions=Zanabazar_Square}\\p{scx=Hira}\\p{Cs}'.repeat(128)+']'+')'.repeat(128)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='A'.repeat(10000)+'\nー';c").unwrap();
    let root = first.root_value(value.clone(), 100).unwrap();
    assert!(matches!(
        second.root_value(value, 100),
        Err(ObjectError::Heap(HeapError::ForeignHandle))
    ));
    first.collect(usize::MAX).unwrap();
    assert_eq!(first.eval(r"let m=c.exec(text);m.length===129&&m[1]==='ー'&&m[128]==='ー'&&m.indices[128][0]===10001&&m.indices[128][1]===10002&&c.lastIndex===10002&&c.source===p"), Ok(Value::Boolean(true)));
    assert_eq!(
        second.eval(r"let r=/\p{scx=Hira}/vy;r.lastIndex===0&&r.exec('A')===null&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    first.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    first.collect(usize::MAX).unwrap();
    drop(root);
}

#[test]
fn script_extensions_opted_work_aborts_without_catch_or_finally() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\p{scx=Greek}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
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
fn script_extensions_exact_names_and_unadmitted_compositions() {
    for mode in ["u", "v"] {
        for name in [
            "scx",
            "Script_Extensions",
            "Script_extensions=Latin",
            "Scx=Latin",
            "scx=latin",
            "scx=LATN",
            "scx=OldItalic",
            "scx=Latin ",
            "scx=Latín",
            "scx=Script=Latin",
        ] {
            let pattern = JsString::from(format!(r"\p{{{name}}}").as_str());
            check(&format!(
                "let caught=false;try{{new RegExp({pattern:?},{mode:?})}}catch(e){{caught=e instanceof SyntaxError}}caught"
            ));
        }
    }
    check(r"/[\p{scx=Hira}&&A]/v.exec('A')===null");
    check(r"/[\p{scx=Greek}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x300),m=/[\p{scx=Greek}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [
        r"/\p{scx=Latin}A/u.exec('AA')",
        r"/\p{scx=Latin}+b/u.exec('AA')",
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
fn script_extensions_selected_results_snapshot() {
    let mut rows = String::new();
    for (body, units, start) in [
        (
            r"(?<x>\P{scx=Common})",
            "µμA".encode_utf16().collect::<Vec<_>>(),
            0,
        ),
        (r"(?<x>[\P{scx=Greek}A])", "µιA".encode_utf16().collect(), 0),
        (
            r"(?<x>\p{scx=Inherited})",
            "\u{300}\u{345}\u{301}\u{34f}".encode_utf16().collect(),
            0,
        ),
        (
            r"(?<x>\p{scx=Hira})",
            "😀ーあア".encode_utf16().collect(),
            1,
        ),
        (
            r"(?<x>\p{scx=Zzzz})",
            vec![0xd83d, 0xde00, 0xd800, 0xe000, 0x378],
            1,
        ),
        (r"(?<x>\P{scx=Hrkt})", vec![0xd83d, 0xde00, 0xd800], 1),
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
