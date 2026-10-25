//! A validated whole group repeated zero times never executes its body.

use spite_runtime::{Realm, Value};

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
fn mandatory_enclosing_captures_select_empty_spans_and_inner_names_stay_undefined() {
    check(
        r#"
        var ok=true,body='(?<root>(?=(?<x>a))b|(?<=(?<y>c))d)';
        for(var mode of ['','u','v']) for(var count of ['{0}','{000,000}?']) {
            for(var wrapper of [['(?<outside>',')',1],['(?<outside>(?:(?<second>',')))',2],['(?:',')',0]]) {
                var pattern=wrapper[0]+body+count+wrapper[1],r=new RegExp(pattern,'dgy'+mode),point=mode===''?2:1;
                r.lastIndex=2;var m=r.exec('A😀!');
                ok=ok&&r.source===pattern&&m.index===point&&m[0]===''&&r.lastIndex===point&&m.length===wrapper[2]+4;
                for(var i=1;i<m.length;i++)ok=ok&&(i<=wrapper[2]?m[i]===''&&m.indices[i][0]===point&&m.indices[i][1]===point:m[i]===undefined&&m.indices[i]===undefined);
                ok=ok&&m.groups.root===undefined&&m.groups.x===undefined&&m.groups.y===undefined&&m.indices.groups.root===undefined&&m.indices.groups.x===undefined&&m.indices.groups.y===undefined;
                if(wrapper[2]>0)ok=ok&&m.groups.outside===''&&m.indices.groups.outside===m.indices[1];
                if(wrapper[2]>1)ok=ok&&m.groups.second===''&&m.indices.groups.second===m.indices[2];
            }
        }
        var sets=new RegExp('(?<outside>(?<root>[[a-z]--[b]](?<x>[\\p{RGI_Emoji}])){0})','dgvy');
        sets.lastIndex=2;var m=sets.exec('A😀!');
        ok=ok&&m.length===4&&m.index===1&&m.groups.outside===''&&m.groups.root===undefined&&m.groups.x===undefined&&m.indices.groups.outside===m.indices[1]&&m.indices[1][0]===1&&m.indices[1][1]===1;
        var references=new RegExp('(?<outside>(?<second>(?<third>(?<root>(?=\\4)b|\\k<outside>){0})))','dgu'),m=references.exec('x');
        ok&&m.length===5&&m[1]===''&&m[2]===''&&m[3]===''&&m[4]===undefined&&m.groups.root===undefined&&m.indices.groups.outside===m.indices[1]&&m.indices.groups.second===m.indices[2]&&m.indices.groups.third===m.indices[3]
    "#,
    );
}

#[test]
fn enclosing_empty_captures_flow_through_matchall_replace_search_and_split() {
    check(
        r#"
        var pattern='(?<outside>(?<root>(?=(?<x>a))b){0})',r=new RegExp(pattern,'dgu');
        var ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m.length===4&&m[1]===''&&m[2]===undefined&&m[3]===undefined&&m.groups.outside===''&&m.indices.groups.outside===m.indices[1];
        var calls=[];var value='ab'.replace(r,function(m,outside,root,x,at,input,groups){calls.push(at);return m===''&&outside===''&&root===undefined&&x===undefined&&input==='ab'&&groups.outside===''&&groups.root===undefined?'!':'?'});
        ok=ok&&value==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.search(r)===0&&'ab'.split(r).join(',')==='a,,,,b';
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');
        ok&&m.index===1&&copy.lastIndex===1&&m.groups.outside===''&&m.groups.root===undefined&&m.indices.groups.outside===m.indices[1]
    "#,
    );
}
