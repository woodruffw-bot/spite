//! Required outside-reference lookbehind counts beyond native integer width.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn unrepresentable_outside_lookbehind_result_snapshot() {
    let huge = "9".repeat(100);
    let mut rows = String::new();
    for (pattern, text) in [
        (r"()(?<=(\1)COUNT)b", "qb"),
        (r"()(?<=(\1)COUNTa)b", "ab"),
        (r"()()(?<=(\1\2\b)COUNT)b", " b"),
        (r"(a)(?<=(\1a)COUNT)b", "ab"),
        (r"(a)(?<!(\1a)COUNT)b", "ab"),
        (r"()(?<!(\1)COUNTq)b", "xb"),
        (r"(?<x>)(?<=(?<y>\k<x>)COUNT)b", "qb"),
        (r"(a(?<=\1COUNT))b", "ab"),
        (r"(?<=\1COUNT)(a)", "qa"),
    ] {
        let source = JsString::from(pattern.replace("COUNT", &format!("{{{huge}}}")).as_str());
        let input = JsString::from(text);
        let program = format!(
            "let r=new RegExp({source:?},'d'),a=r.exec({input:?});JSON.stringify(a===null?{{match:null}}:{{matches:[...a],index:a.index,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex}})"
        );
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{pattern:?} input={text:?} {result:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn empty_required_exact_lazy_bounded_and_unbounded_counts_keep_capture_identity() {
    check(
        r"let h='9'.repeat(100),upper='1'+'0'.repeat(101),ok=true;for(let bounds of ['{'+h+'}','{'+h+'}?','{'+h+',}','{'+h+',}?','{'+h+','+upper+'}']){let r=new RegExp('(?<x>)(?<=(?<y>\\k<x>)'+bounds+')b','d'),a=r.exec('qb');ok=ok&&a.index===1&&a.groups.y===''&&a.indices.groups.y===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===1;}ok",
    );
    check(
        r"let h='9'.repeat(100),a=new RegExp('()(?<=(\\1){'+h+'}a)b','d').exec('ab');a.index===1&&a.indices[1][0]===1&&a.indices[2][0]===0&&a.indices[2][1]===0",
    );
    check(
        r"let h='9'.repeat(100),a=new RegExp('(a(?<=\\1{'+h+'}))b','d').exec('ab'),b=new RegExp('(?<=\\1{'+h+'})(a)','d').exec('qa');a[1]==='a'&&a.index===0&&b[1]==='a'&&b.index===1",
    );
    check(
        r"let h='9'.repeat(100),r=new RegExp('()()(?<=(\\1\\2\\b){'+h+'})b','d'),a=r.exec(' b');a.index===1&&a[3]===''&&a.indices[3][0]===1&&new RegExp('()(?<=(\\1\\B){'+h+'})b').exec(' b')===null",
    );
}

#[test]
fn nonempty_unavailable_widths_and_completed_negative_effects_restore_owned_captures() {
    check(
        r"let h='9'.repeat(100),ok=true;for(let body of ['(\\1)','(\\1a)','(ab)']){let tail=body==='(ab)'?'\\1':'',p='(a)(?<='+body+'{'+h+'}'+tail+')b',n='(a)(?<!'+body+'{'+h+'}'+tail+')b',a=new RegExp(n,'d').exec('ab');ok=ok&&new RegExp(p).exec('ab')===null&&a[1]==='a'&&a[2]===undefined&&a.indices[2]===undefined;}ok",
    );
    check(
        r"let h='9'.repeat(100),a=new RegExp('(?<x>)(?<!(?<y>\\k<x>){'+h+'}q)b','d').exec('xb');a.index===1&&a.groups.x===''&&a.groups.y===undefined&&a.indices.groups.y===undefined&&Object.hasOwn(a.groups,'y')",
    );
}

#[test]
fn global_sticky_empty_advancement_and_callbacks_preserve_positions() {
    check(
        r"let h='9'.repeat(100),r=new RegExp('()(?<=(\\1){'+h+'})b','dy');r.lastIndex=1;let a=r.exec('qb');a.index===1&&a[2]===''&&r.lastIndex===2&&r.exec('qb')===null&&r.lastIndex===0",
    );
    check(
        r"let h='9'.repeat(100),r=new RegExp('()(?<=(\\1){'+h+'})','dg'),a=[...'ab'.matchAll(r)];a.length===3&&a[2].index===2&&a[2].indices[2][0]===2&&r.lastIndex===0",
    );
    check(
        r"let h='9'.repeat(100),r=new RegExp('()(?<=(\\1){'+h+'})b','g'),seen=[];let text='qb qb'.replace(r,(m,x,y,i)=>{seen.push(x,y,i);return '_'});text==='q_ q_'&&seen.join('|')==='||1|||4'",
    );
}

#[test]
fn deep_captures_clones_collection_and_long_decimal_counts_have_no_default_limits() {
    let mut realm = Realm::default();
    realm.eval(r"let n=100000,h='9'.repeat(10000),r=new RegExp('()(?<='+'('.repeat(n)+'\\1'+')'.repeat(n)+'{'+h+'})b','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('qb');a.index===1&&a.length===n+2&&a[1]===''&&a[n+1]===''&&a.indices[n+1][0]===1&&a.indices[n+1][1]===1&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval(r"let negative=new RegExp('()(?<!'+'('.repeat(n)+'\\1'+')'.repeat(n)+'{'+h+'}q)b','d'),restored=negative.exec('xb');restored.index===1&&restored[1]===''&&restored[2]===undefined&&restored[n+1]===undefined"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn invalid_decimal_bounds_still_throw_and_opted_in_work_aborts_bypass_handlers() {
    check(
        r"let h='9'.repeat(100),bad='1'+'0'.repeat(100),ok=false;try{new RegExp('()(?<=\\1{'+bad+','+h+'})b')}catch(e){ok=e instanceof SyntaxError}ok",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,h='9'.repeat(100),r=new RegExp('()(?<=(\\1){'+h+'})b','g');r.lastIndex=1;let text='a'.repeat(5000)").unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}
