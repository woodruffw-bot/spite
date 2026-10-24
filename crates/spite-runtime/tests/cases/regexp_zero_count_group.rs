//! A validated whole group repeated zero times never executes its body.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn arbitrary_validated_bodies_keep_every_capture_and_name_undefined() {
    check(
        r#"
        var bodies = [
            '(?<root>(?=(?<ahead>a))(?<=(?<behind>b))\\k<ahead>|(?<tail>c))',
            '(?<root>(?<x>a+?)(?:b|c){2,3}\\k<x>)',
            '(?<root>(?i:(?<x>a))(?-i:b)[\\p{Script=Greek}])',
            '(?<root>[\\p{RGI_Emoji}](?<x>[\\q{ab|cd}&&\\q{cd|ef}]))',
            '(?<root>[[a-z]--[b]](?<x>[\\q{\\(\\)|ab}]))'
        ], ok = true;
        for (var i=0;i<bodies.length;i++) {
            for (var count of ['{0}','{0,0}','{0000}?','{000,000}?']) {
                var flags = i>=3 ? 'dgvi' : 'dguim', r = new RegExp(bodies[i]+count,flags);
                r.lastIndex=2;
                var m=r.exec('a😀b');
                ok=ok&&m.index===1&&m[0]===''&&r.lastIndex===1&&m.indices[0][0]===1&&m.indices[0][1]===1;
                for(var n=1;n<m.length;n++) ok=ok&&m[n]===undefined&&m.indices[n]===undefined;
                for(var name of Object.keys(m.groups)) ok=ok&&Object.hasOwn(m.indices.groups,name)&&m.groups[name]===undefined&&m.indices.groups[name]===undefined;
                ok=ok&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null;
            }
        }
        ok
    "#,
    );
}

#[test]
fn modes_lastindex_and_string_consumers_use_the_empty_match() {
    check(
        r#"
        var ok=true, body='(?<root>(?=(?<x>a))b|(?<=c)d){0}';
        for(var mode of ['','u','v']) for(var sticky of ['','y']) {
            var r=new RegExp(body,'dg'+mode+sticky), input='A😀!';
            for(var start=0;start<=input.length+1;start++) {
                r.lastIndex=start; var m=r.exec(input), point=mode!==''&&start===2?1:start;
                ok=ok&&(start>input.length ? m===null&&r.lastIndex===0 : m.index===point&&m[0]===''&&m[1]===undefined&&m[2]===undefined&&r.lastIndex===point);
            }
        }
        var r=new RegExp(body,'dgu'), matches=Array.from('a😀'.matchAll(r));
        ok=ok&&matches.length===3&&matches[0].index===0&&matches[1].index===1&&matches[2].index===3;
        var seen=[];var replaced='ab'.replace(r,function(m,root,x,p,input,groups){seen.push(p);return m===''&&root===undefined&&x===undefined&&input==='ab'&&groups.root===undefined?'!':'?'});
        ok=ok&&replaced==='!a!b!'&&seen.join(',')==='0,1,2'&&'ab'.search(r)===0&&'ab'.split(r).join(',')==='a,,,b';
        ok
    "#,
    );
}

#[test]
fn full_syntax_validation_precedes_erasure() {
    check(
        r#"
        var ok=true, invalid=[
            ['(?:(?<x>a)\\k<missing>){0}','u'],
            ['(?:(a)\\2){0}','u'],
            ['(?:[z-a]){0}','u'],
            ['(?:[\\p{NotAProperty}]){0}','v'],
            ['(?:[\\q{ab}&&]){0}','v'],
            ['(?:(?<x>a)(?<x>b)){0}','u'],
            ['(?:(?ii:a)){0}','u'],
            ['(?:(a){2,1}){0}','u'],
            ['(a){0}','uv'],
            ['(a){0}','gg']
        ];
        for(var pair of invalid){var caught=false;try{new RegExp(pair[0],pair[1])}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}
        ok
    "#,
    );
}

#[test]
fn escaped_and_duplicate_names_keep_undefined_aliases() {
    check(
        r#"
        var r=new RegExp('(?:(?<x>a)|(?<x>b)(?<\\u0079>c)\\k<y>){0}','dgu'),m=r.exec('x');
        m.length===4&&m[1]===undefined&&m[2]===undefined&&m[3]===undefined&&
        Object.keys(m.groups).join(',')==='x,y'&&m.groups.x===undefined&&m.groups.y===undefined&&
        Object.keys(m.indices.groups).join(',')==='x,y'&&m.indices.groups.x===undefined&&m.indices.groups.y===undefined
    "#,
    );
}

#[test]
fn opt_in_constructor_work_limits_remain_host_aborts() {
    let pattern = JsString::from(format!("(?:(?=({}))[a]){{0}}", "a".repeat(20_000)).as_str());
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=new RegExp({pattern:?},'u')}}catch{{marker=1}}finally{{marker=2}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7&&marker===0"), Ok(Value::Boolean(true)));
}

#[test]
fn deep_erased_captures_clone_and_collect_without_executing_body() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var depth=1000,pattern='('.repeat(depth)+'(?=(a))b'+')'.repeat(depth)+'{0}',r=new RegExp(pattern,'dgu'),copy=new RegExp(r);
        var m=copy.exec('x'),ok=m.length===depth+2&&m.index===0&&m[0]==='';
        for(var i=1;i<m.length;i++)ok=ok&&m[i]===undefined&&m.indices[i]===undefined;
        ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("copy.exec('x')[1]===undefined && copy.source===r.source"),
        Ok(Value::Boolean(true))
    );
}
