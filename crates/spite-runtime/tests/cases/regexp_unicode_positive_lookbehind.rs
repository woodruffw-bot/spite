//! Positive Unicode lookbehind captures the complete predecessor character.

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn named_predecessor_and_outside_captures_keep_original_source_and_ranges() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var row of [['a','a'],['[a-z]','a'],['\\d','2'],['.','😀'],['\\u{1F600}','😀'],['\\uD83D\\uDE00','😀'],['\\uD800','\ud800']]){
            var source='(?<outside>(?<=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+mode);r.lastIndex=row[1].length;var m=r.exec(row[1]);
            ok=ok&&m.index===row[1].length&&m[0]===''&&m.groups.outside===''&&m.groups.run===row[1]&&m.indices.groups.outside[0]===row[1].length&&m.indices.groups.outside[1]===row[1].length&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===row[1].length&&r.source===source&&r.lastIndex===row[1].length;
            var copy=new RegExp(r);copy.lastIndex=row[1].length;ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===row[1];
        }ok
    "#,
    );
}

#[test]
fn predecessor_search_sticky_end_and_initial_pair_candidates_are_complete() {
    check(
        r#"
        var ok=true,input='😀a';
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp('(?<=(?<run>.))','d'+state+mode);r.lastIndex=start;var m=r.exec(input),active=r.global||r.sticky,point=active?start:0;
            if(point===1)point=0;
            if(point>input.length||r.sticky&&point<2){ok=ok&&m===null&&r.lastIndex===0;continue}
            if(point<2)point=2;
            ok=ok&&m.index===point&&m[0]===''&&m.groups.run===(point===2?'😀':'a')&&m.indices.groups.run[0]===(point===2?0:2)&&m.indices.groups.run[1]===point&&r.lastIndex===(active?point:start);
        }ok
    "#,
    );
}

#[test]
fn predecessor_line_dot_property_and_lone_surrogate_contexts_remain_original() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var m=new RegExp('(?<=(?<run>^a$))','dim'+mode).exec('\nA\n');ok=ok&&m.index===2&&m.groups.run==='A'&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2;
            var m=new RegExp('(?<=(?<run>^\\$$))','d'+mode).exec('$');ok=ok&&m.index===1&&m.groups.run==='$'&&m.indices.groups.run[0]===0;
            var m=new RegExp('(?<=(?<run>\\p{ASCII}))','di'+mode).exec('ſa');ok=ok&&m.index===1&&m.groups.run==='ſ'&&m.indices.groups.run[0]===0;
            var m=new RegExp('(?<=(?<run>.))','ds'+mode).exec('\na');ok=ok&&m.index===1&&m.groups.run==='\n';
            var r=new RegExp('(?<=(?<run>\\uD800))','dg'+mode);ok=ok&&r.exec('\ud800\udc00')===null&&r.lastIndex===0;
            var m=r.exec('\ud800x\udc00');ok=ok&&m.index===1&&m.groups.run==='\ud800'&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===1;
        }ok
    "#,
    );
}

#[test]
fn predecessor_string_consumers_copies_and_collection_keep_captures_before_match() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='(?<=(?<run>.))',r=new RegExp(source,'dg'+mode),all=Array.from('😀a'.matchAll(r));
            ok=ok&&all.length===2&&all[0].index===2&&all[0].groups.run==='😀'&&all[1].index===3&&all[1].groups.run==='a'&&r.lastIndex===0;
            ok=ok&&'😀a'.replace(r,'<$<run>>')==='😀<😀>a<a>'&&'😀a'.replaceAll(r,'<$<run>>')==='😀<😀>a<a>'&&JSON.stringify('😀a'.split(new RegExp('(?<=(.))',mode)))==='["😀","😀","a"]';
            var offsets=[];ok=ok&&'😀a'.replace(r,function(match,run,offset,input,groups){offsets.push(offset);ok=ok&&match===''&&run===(offset===2?'😀':'a')&&groups.run===run;return '!';})==='😀!a!'&&JSON.stringify(offsets)==='[2,3]';
            r.lastIndex=1;ok=ok&&'😀a'.search(r)===2&&r.lastIndex===1;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('😀').groups.run==='😀';
        }ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('b');m.index===1&&m.groups.run==='b'&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===1&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn predecessor_deep_wrappers_retain_all_empty_and_consuming_capture_slots() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='('.repeat(1000)+'(?<='+'('.repeat(1000)+'[a-z]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'d'+mode),m=r.exec('!a');
            ok=ok&&m.index===2&&m.length===2001&&m[0]==='';
            for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':'a')&&m.indices[i][0]===(i<=1000?2:1)&&m.indices[i][1]===2;
        }ok
    "#,
    );
}

#[test]
fn word_predecessor_captures_keep_boundary_polarity_source_and_outside_ranges() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var row of [['\\ba\\B','aaa',1],['\\Ba\\B','aaaa',2],['\\b[a-z]\\b','!a!',2]]){
            var source='(?<outside>(?<=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+mode);r.lastIndex=row[2];var m=r.exec(row[1]);
            ok=ok&&m.index===row[2]&&m[0]===''&&m.groups.outside===''&&m.groups.run==='a'&&m.indices.groups.outside[0]===row[2]&&m.indices.groups.outside[1]===row[2]&&m.indices.groups.run[0]===row[2]-1&&m.indices.groups.run[1]===row[2]&&r.lastIndex===row[2]&&r.source===source;
        }ok
    "#,
    );
}

#[test]
fn word_predecessor_line_fold_property_and_backspace_contexts_remain_original() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var m=new RegExp('(?<=(?<run>^\\ba\\b$))','dim'+mode).exec('\nA\n');ok=ok&&m.index===2&&m.groups.run==='A'&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2;
            var m=new RegExp('(?<=(?<run>\\bſ\\b))','di'+mode).exec('!ſ!');ok=ok&&m.index===2&&m.groups.run==='ſ';
            var m=new RegExp('(?<=(?<run>\\b\\p{ASCII}\\b))','di'+mode).exec('!ſ!');ok=ok&&m.index===2&&m.groups.run==='ſ';
            var m=new RegExp('(?<=(?<run>\\B[\\b]\\B))','d'+mode).exec('\b');ok=ok&&m.index===1&&m.groups.run==='\b'&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===1;
        }ok
    "#,
    );
}

#[test]
fn word_predecessor_string_consumers_copies_and_collection_keep_preceding_captures() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='(?<=(?<run>\\ba\\b))',r=new RegExp(source,'dg'+mode),all=Array.from('!!a! a'.matchAll(r));
            ok=ok&&all.length===2&&all[0].index===3&&all[1].index===6&&all[0].groups.run==='a'&&r.lastIndex===0;
            ok=ok&&'!!a! a'.replace(r,'<$<run>>')==='!!a<a>! a<a>'&&JSON.stringify('!!a! a'.split(new RegExp('(?<=(\\ba\\b))',mode)))==='["!!a","a","! a"]';
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!a!').groups.run==='a';
        }ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('!a!');m.index===2&&m.groups.run==='a'&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2&&copy.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn word_predecessor_search_retains_complete_character_and_initial_pair_boundaries() {
    check(
        "var ok=true;for(var mode of ['u','v']){var r=new RegExp('(?<=(?<run>\\\\B.\\\\b))','dg'+mode),m=r.exec('a😀ab!');ok=ok&&m.index===5&&m.groups.run==='b'&&m.indices.groups.run[0]===4&&m.indices.groups.run[1]===5&&r.lastIndex===5;r=new RegExp('(?<=(?<run>\\\\B.\\\\b))','dgy'+mode);r.lastIndex=2;ok=ok&&r.exec('a😀ab!')===null&&r.lastIndex===0}ok",
    );
}

#[test]
fn q_class_predecessors_keep_original_source_names_and_independent_capture_ranges() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var atom of ['[\\q{a|😀}]','[[\\q{a|😀}]--[b]]','[[\\q{a|😀}]&&[a😀]]','[x\\q{a|😀}]']){
            var source='(?<outside>(?<=(?<run>'+atom+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=2;var m=r.exec('😀a');
            ok=ok&&m.index===2&&m[0]===''&&m.groups.outside===''&&m.groups.run==='😀'&&m.indices.groups.outside[0]===2&&m.indices.groups.outside[1]===2&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===2&&r.lastIndex===2&&r.source===source;
            var copy=new RegExp(r);copy.lastIndex=1;ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('a').groups.run==='a';
        }ok
    "#,
    );
}

#[test]
fn q_class_predecessors_retain_original_line_fold_property_and_surrogate_context() {
    check(
        r#"
        var m=/(?<=(?<run>^[\q{a|😀}]$))/dimv.exec('\nA\n'),ok=m.index===2&&m.groups.run==='A'&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2;
        for(var text of ['S','ſ','K','K']){var m=/(?<=(?<run>[\q{ſ|k}]))/div.exec(text);ok=ok&&m.index===1&&m.groups.run===text&&m.indices.groups.run[0]===0}
        var m=/(?<=(?<run>[\p{ASCII}\q{😀}]))/div.exec('ſ');ok=ok&&m.index===1&&m.groups.run==='ſ';
        var r=/(?<=(?<run>[\q{\uD800|x}]))/dgv;ok=ok&&r.exec('😀')===null&&r.lastIndex===0;var m=r.exec('\ud800x');ok=ok&&m.index===1&&m.groups.run==='\ud800'&&m.indices.groups.run[0]===0;
        ok
    "#,
    );
}

#[test]
fn q_class_predecessor_string_consumers_copies_and_collection_keep_preceding_extents() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<=(?<run>[\\q{a|😀}])))',r=new RegExp(source,'dgv'),all=Array.from('😀a'.matchAll(r));
        var ok=all.length===2&&all[0].index===2&&all[0].groups.run==='😀'&&all[1].index===3&&all[1].groups.run==='a'&&r.lastIndex===0;
        ok=ok&&'😀a'.replace(r,'<$<run>>')==='😀<😀>a<a>'&&'😀a'.replaceAll(r,'<$<run>>')==='😀<😀>a<a>'&&JSON.stringify('😀a'.split(new RegExp('(?<=([\\q{a|😀}]))','v')))==='["😀","😀","a"]';
        var offsets=[];ok=ok&&'😀a'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&outside===''&&run===groups.run;return '!';})==='😀!a!'&&JSON.stringify(offsets)==='[2,3]';
        r.lastIndex=1;ok=ok&&'😀a'.search(r)===2&&r.lastIndex===1;
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('😀').groups.run==='😀';ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('a');m.index===1&&m.groups.outside===''&&m.groups.run==='a'&&m.indices.groups.run[0]===0&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn q_class_predecessor_deep_wrappers_and_pair_offsets_keep_exact_original_slots() {
    check(
        r#"
        var source='('.repeat(1000)+'(?<='+'('.repeat(1000)+'[\\q{a|😀}]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'dgyv');r.lastIndex=3;var m=r.exec('!😀a');
        var ok=m.index===3&&m.length===2001&&m[0]===''&&r.lastIndex===3;
        for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':'😀')&&m.indices[i][0]===(i<=1000?3:1)&&m.indices[i][1]===3;
        r.lastIndex=2;ok=ok&&r.exec('!😀a')===null&&r.lastIndex===0;r.lastIndex=5;ok=ok&&r.exec('!😀a')===null&&r.lastIndex===0;ok
    "#,
    );
}

#[test]
fn word_q_class_predecessors_keep_original_polarity_source_and_named_ranges() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['\\b[\\q{a|b}]\\b','ab a!',4,'a'],['\\b[\\q{a|b}]\\B','ab!',1,'a'],['\\B[\\q{a|😀}]\\B','😀😀',2,'😀']]){
            var source='(?<outside>(?<=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=row[2];var m=r.exec(row[1]);
            ok=ok&&m.index===row[2]&&m[0]===''&&m.groups.outside===''&&m.groups.run===row[3]&&m.indices.groups.outside[0]===row[2]&&m.indices.groups.outside[1]===row[2]&&m.indices.groups.run[0]===row[2]-row[3].length&&m.indices.groups.run[1]===row[2]&&r.lastIndex===row[2]&&r.source===source;
            var copy=new RegExp(r);copy.lastIndex=row[2];ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===row[3];
        }ok
    "#,
    );
}

#[test]
fn word_q_class_predecessor_line_fold_property_and_backspace_context_stay_original() {
    check(
        r#"
        var m=/(?<=(?<run>^\b[\q{a|😀}]\b$))/dimv.exec('\nA\n'),ok=m.index===2&&m.groups.run==='A'&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2;
        for(var text of ['S','ſ','K','K']){var m=/(?<=(?<run>\b[\q{ſ|k}]\b))/div.exec('!'+text+'!');ok=ok&&m.index===2&&m.groups.run===text&&m.indices.groups.run[0]===1}
        var m=/(?<=(?<run>\b[\p{ASCII}\q{😀}]\b))/div.exec('!ſ!');ok=ok&&m.index===2&&m.groups.run==='ſ';
        var m=/(?<=(?<run>\B[\q{\u0008|a}]\B))/dv.exec('\b');ok=ok&&m.index===1&&m.groups.run==='\b'&&m.indices.groups.run[0]===0;ok
    "#,
    );
}

#[test]
fn word_q_class_predecessor_string_consumers_copies_and_collection_keep_preceding_spans() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<=(?<run>\\b[\\q{a|b}]\\b)))',r=new RegExp(source,'dgv'),all=Array.from('!a! b'.matchAll(r));
        var ok=all.length===2&&all[0].index===2&&all[0].groups.run==='a'&&all[1].index===5&&all[1].groups.run==='b'&&r.lastIndex===0;
        ok=ok&&'!a! b'.replace(r,'<$<run>>')==='!a<a>! b<b>'&&JSON.stringify('!a! b'.split(new RegExp('(?<=(\\b[\\q{a|b}]\\b))','v')))==='["!a","a","! b"]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!a!').groups.run==='a';ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('!b!');m.index===2&&m.groups.outside===''&&m.groups.run==='b'&&m.indices.groups.run[0]===1&&copy.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn word_q_class_predecessor_search_preserves_complete_candidates_and_initial_pair_rewind() {
    check(
        "var r=new RegExp('(?<=(?<run>\\\\B[\\\\q{😀|a}]\\\\B))','dgyv');r.lastIndex=3;var m=r.exec('😀😀');m.index===2&&m.groups.run==='😀'&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===2&&r.lastIndex===2",
    );
}

#[test]
fn scalar_v_set_predecessors_keep_original_source_and_named_independent_ranges() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['[[a-z]--[aeiou]]','!ab',3,'b'],['[[a-z]&&[ab]]','!ab',2,'a'],['[[a][b]]','!ab',2,'a'],['[[😀a]--[a]]','!😀a',3,'😀'],['[^[a-z]]','😀abc',2,'😀']]){
            var source='(?<outside>(?<=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=row[2];var m=r.exec(row[1]);
            ok=ok&&m.index===row[2]&&m[0]===''&&m.groups.outside===''&&m.groups.run===row[3]&&m.indices.groups.outside[0]===row[2]&&m.indices.groups.outside[1]===row[2]&&m.indices.groups.run[0]===row[2]-row[3].length&&m.indices.groups.run[1]===row[2]&&r.lastIndex===row[2]&&r.source===source;
            var copy=new RegExp(r);copy.lastIndex=row[2];ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===row[3];
        }ok
    "#,
    );
}

#[test]
fn scalar_v_set_predecessors_keep_original_line_fold_property_and_surrogate_context() {
    check(
        r#"
        var m=/(?<=(?<run>^[[a-z]--[aeiou]]$))/dimv.exec('\nB\n'),ok=m.index===2&&m.groups.run==='B'&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2;
        var m=/(?<=(?<run>[[\p{ASCII}]--[x]]))/div.exec('ſK');ok=ok&&m.index===1&&m.groups.run==='ſ';
        var m=/(?<=(?<run>[^[\p{ASCII}]]))/div.exec('ſK😀');ok=ok&&m.index===4&&m.groups.run==='😀'&&m.indices.groups.run[0]===2&&m.indices.groups.run[1]===4;
        var m=/(?<=(?<run>[\p{Script=Greek}&&[Α-Ωα-ω]]))/div.exec('!Σ');ok=ok&&m.index===2&&m.groups.run==='Σ';
        ok=ok&&/(?<=(?<run>[[a-z]--[a-z]]))/dv.exec('abc')===null;
        var r=/(?<=(?<run>[[\uD800]--[x]]))/dgv;ok=ok&&r.exec('😀')===null&&r.lastIndex===0;var m=r.exec('\ud800x');ok=ok&&m.index===1&&m.groups.run==='\ud800';ok
    "#,
    );
}

#[test]
fn scalar_v_set_predecessor_consumers_copies_and_collection_keep_preceding_extents() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<=(?<run>[[😀a]--[x]])))',r=new RegExp(source,'dgv'),all=Array.from('😀a'.matchAll(r));
        var ok=all.length===2&&all[0].index===2&&all[0].groups.run==='😀'&&all[1].index===3&&all[1].groups.run==='a'&&r.lastIndex===0;
        ok=ok&&'😀a'.replace(r,'<$<run>>')==='😀<😀>a<a>'&&'😀a'.replaceAll(r,'<$<run>>')==='😀<😀>a<a>'&&JSON.stringify('😀a'.split(new RegExp('(?<=([[😀a]--[x]]))','v')))==='["😀","😀","a"]';
        var offsets=[];ok=ok&&'😀a'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&outside===''&&run===groups.run;return '!';})==='😀!a!'&&JSON.stringify(offsets)==='[2,3]';
        r.lastIndex=1;ok=ok&&'😀a'.search(r)===2&&r.lastIndex===1;
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('😀').groups.run==='😀';ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('a');m.index===1&&m.groups.outside===''&&m.groups.run==='a'&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===1&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn scalar_v_set_predecessor_deep_wrappers_and_pair_offsets_keep_original_slots() {
    check(
        r#"
        var source='('.repeat(1000)+'(?<='+'('.repeat(1000)+'[[😀a]--[x]]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'dgyv');r.lastIndex=3;var m=r.exec('!😀a');
        var ok=m.index===3&&m.length===2001&&m[0]===''&&r.lastIndex===3;
        for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':'😀')&&m.indices[i][0]===(i<=1000?3:1)&&m.indices[i][1]===3;
        r.lastIndex=2;ok=ok&&r.exec('!😀a')===null&&r.lastIndex===0;r.lastIndex=5;ok=ok&&r.exec('!😀a')===null&&r.lastIndex===0;
        var nested='['.repeat(1000)+'a'+']'.repeat(1000),m=new RegExp('(?<=(?<run>'+nested+'))','dv').exec('a');ok=ok&&m.index===1&&m.groups.run==='a'&&m.indices.groups.run[0]===0;ok
    "#,
    );
}

#[test]
fn word_scalar_v_set_predecessors_keep_original_polarity_source_and_named_ranges() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['\\b[[a-z]--[aeiou]]\\b','ab b!',4,'b'],['\\B[[a-z]--[aeiou]]\\b','ab!',2,'b'],['\\b[[a-z]--[aeiou]]\\B','bb!',1,'b']]){
            var source='(?<outside>(?<=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=row[2];var m=r.exec(row[1]);
            ok=ok&&m.index===row[2]&&m[0]===''&&m.groups.outside===''&&m.groups.run===row[3]&&m.indices.groups.outside[0]===row[2]&&m.indices.groups.outside[1]===row[2]&&m.indices.groups.run[0]===row[2]-1&&m.indices.groups.run[1]===row[2]&&r.lastIndex===row[2]&&r.source===source;
            var copy=new RegExp(r);copy.lastIndex=row[2];ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===row[3];
        }ok
    "#,
    );
}

#[test]
fn word_scalar_v_set_predecessors_keep_original_line_fold_property_and_backspace_context() {
    check(
        r#"
        var m=/(?<=(?<run>^\b[[a-z]--[aeiou]]\b$))/dimv.exec('\nB\n'),ok=m.index===2&&m.groups.run==='B'&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2;
        for(var text of ['S','ſ','K','K']){var m=/(?<=(?<run>\b[[\p{ASCII}]--[x]]\b))/div.exec('!'+text+'!');ok=ok&&m.index===2&&m.groups.run===text&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2}
        var m=/(?<=(?<run>\B[[\b]--[a]]\B))/dv.exec('\b');ok=ok&&m.index===1&&m.groups.run==='\b'&&m.indices.groups.run[0]===0;ok
    "#,
    );
}

#[test]
fn word_scalar_v_set_predecessor_consumers_copies_and_collection_keep_preceding_extents() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<=(?<run>\\b[[a-z]--[aeiou]]\\b)))',r=new RegExp(source,'dgv'),all=Array.from('!b! c'.matchAll(r));
        var ok=all.length===2&&all[0].index===2&&all[0].groups.run==='b'&&all[1].index===5&&all[1].groups.run==='c'&&r.lastIndex===0;
        ok=ok&&'!b! c'.replace(r,'<$<run>>')==='!b<b>! c<c>'&&JSON.stringify('!b! c'.split(new RegExp('(?<=(\\b[[a-z]--[aeiou]]\\b))','v')))==='["!b","b","! c"]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!b!').groups.run==='b';ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('!c!');m.index===2&&m.groups.outside===''&&m.groups.run==='c'&&m.indices.groups.run[0]===1&&copy.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn word_scalar_v_set_predecessor_search_keeps_complete_candidates_and_initial_pair_rewind() {
    check(
        "var r=new RegExp('(?<=(?<run>\\\\B[[😀a]--[a]]\\\\B))','dgyv');r.lastIndex=3;var m=r.exec('😀😀');m.index===2&&m.groups.run==='😀'&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===2&&r.lastIndex===2",
    );
}

#[test]
fn character_result_predecessors_keep_original_source_and_named_independent_capture_ranges() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['[\\q{a|ab|}&&\\p{Any}]','!a',2,'a'],['[[\\q{ab}&&a]b]','!b',2,'b'],['[^\\q{ab}&&a]','😀',2,'😀'],['[[\\q{😀x|😀}&&\\p{Any}]a]','!😀a',3,'😀']]){
            var source='(?<outside>(?<=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=row[2];var m=r.exec(row[1]);
            ok=ok&&m.index===row[2]&&m[0]===''&&m.groups.outside===''&&m.groups.run===row[3]&&m.indices.groups.outside[0]===row[2]&&m.indices.groups.outside[1]===row[2]&&m.indices.groups.run[0]===row[2]-row[3].length&&m.indices.groups.run[1]===row[2]&&r.lastIndex===row[2]&&r.source===source;
            var copy=new RegExp(r);copy.lastIndex=row[2];ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===row[3];
        }ok
    "#,
    );
}

#[test]
fn character_result_predecessors_keep_original_line_fold_empty_set_and_surrogate_context() {
    check(
        r#"
        var m=/(?<=(?<run>^[\q{a|ab|}&&\p{Any}]$))/dimv.exec('\nA\n'),ok=m.index===2&&m.groups.run==='A'&&m.indices.groups.run[1]===2;
        var m=/(?<=(?<run>[^\q{k|sK|}&&\p{ASCII}]))/div.exec('KKſ');ok=ok&&m.index===3&&m.groups.run==='ſ'&&m.indices.groups.run[1]===3;
        ok=ok&&/(?<=(?<run>[\q{ab}&&a]))/dv.exec('ab')===null;
        var r=/(?<=(?<run>[\q{\uD800|ab}&&\p{Any}]))/dgv;ok=ok&&r.exec('😀')===null&&r.lastIndex===0;var m=r.exec('\ud800x');ok=ok&&m.index===1&&m.groups.run==='\ud800';
        var m=/(?<=(?<run>[\q{\u{10428}|ab}&&\p{Any}]))/div.exec('𐐀');ok&&m.index===2&&m.groups.run==='𐐀'&&m.indices.groups.run[1]===2
    "#,
    );
}

#[test]
fn character_result_predecessor_consumers_copies_and_collection_keep_original_extents() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<=(?<run>[[\\q{😀x|😀}&&\\p{Any}]a])))',r=new RegExp(source,'dgv'),all=Array.from('😀a'.matchAll(r));
        var ok=all.length===2&&all[0].index===2&&all[0].groups.run==='😀'&&all[1].index===3&&all[1].groups.run==='a'&&r.lastIndex===0;
        ok=ok&&'😀a'.replace(r,'<$<run>>')==='😀<😀>a<a>'&&'😀a'.replaceAll(r,'<$<run>>')==='😀<😀>a<a>'&&JSON.stringify('😀a'.split(new RegExp('(?<=([[\\q{😀x|😀}&&\\p{Any}]a]))','v')))==='["😀","😀","a"]';
        var offsets=[];ok=ok&&'😀a'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&outside===''&&run===groups.run;return '!';})==='😀!a!'&&JSON.stringify(offsets)==='[2,3]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('😀').groups.run==='😀';ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('a');m.index===1&&m.groups.outside===''&&m.groups.run==='a'&&m.indices.groups.run[1]===1&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn character_result_predecessor_deep_wrappers_and_pair_offsets_keep_exact_slots() {
    check(
        r#"
        var source='('.repeat(1000)+'(?<='+'('.repeat(1000)+'[\\q{a|ab|}&&\\p{Any}]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'dgyv');r.lastIndex=2;var m=r.exec('!a');
        var ok=m.index===2&&m.length===2001&&m[0]===''&&r.lastIndex===2;
        for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':'a')&&m.indices[i][0]===(i<=1000?2:1)&&m.indices[i][1]===2;ok
    "#,
    );
    check(
        "var r=new RegExp('(?<=(?<run>[[\\\\q{😀x|😀}&&\\\\p{Any}]a]))','dgyv');r.lastIndex=3;var m=r.exec('😀😀');m.index===2&&m.groups.run==='😀'&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===2&&r.lastIndex===2",
    );
}

#[test]
fn word_character_result_predecessors_keep_original_polarity_source_and_named_independent_ranges() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['\\b[\\q{b|c|bc|}&&\\p{Any}]\\b','ab b!',4,'b'],['\\B[\\q{b|c|bc|}&&\\p{Any}]\\b','ab!',2,'b'],['\\b[\\q{b|c|bc|}&&\\p{Any}]\\B','bb!',1,'b']]){
            var source='(?<outside>(?<=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=row[2];var m=r.exec(row[1]);
            ok=ok&&m.index===row[2]&&m[0]===''&&m.groups.outside===''&&m.groups.run===row[3]&&m.indices.groups.outside[0]===row[2]&&m.indices.groups.outside[1]===row[2]&&m.indices.groups.run[0]===row[2]-1&&m.indices.groups.run[1]===row[2]&&r.lastIndex===row[2]&&r.source===source;
            var copy=new RegExp(r);copy.lastIndex=row[2];ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===row[3];
        }ok
    "#,
    );
}

#[test]
fn word_character_result_predecessor_lines_folds_properties_and_backspace_keep_original_context() {
    check(
        r#"
        var m=/(?<=(?<run>^\b[\q{b|c|bc|}&&\p{Any}]\b$))/dimv.exec('\nB\n'),ok=m.index===2&&m.groups.run==='B'&&m.indices.groups.run[1]===2;
        for(var text of ['S','ſ']){var m=/(?<=(?<run>\b[\q{S|sK|}&&\p{ASCII}]\b))/div.exec('!'+text+'!');ok=ok&&m.index===2&&m.groups.run===text&&m.indices.groups.run[1]===2}
        var m=/(?<=(?<run>\B[^\q{ab}&&a]\B))/dv.exec('\b');ok=ok&&m.index===1&&m.groups.run==='\b'&&m.indices.groups.run[1]===1;ok
    "#,
    );
}

#[test]
fn word_character_result_predecessor_string_consumers_copies_and_collection_keep_original_extents()
{
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<=(?<run>\\b[\\q{b|c|bc|}&&\\p{Any}]\\b)))',r=new RegExp(source,'dgv'),all=Array.from('!b! c'.matchAll(r));
        var ok=all.length===2&&all[0].index===2&&all[0].groups.run==='b'&&all[1].index===5&&all[1].groups.run==='c'&&r.lastIndex===0;
        ok=ok&&'!b! c'.replace(r,'<$<run>>')==='!b<b>! c<c>'&&JSON.stringify('!b! c'.split(new RegExp('(?<=(\\b[\\q{b|c|bc|}&&\\p{Any}]\\b))','v')))==='["!b","b","! c"]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!b!').groups.run==='b';ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('!c!');m.index===2&&m.groups.outside===''&&m.groups.run==='c'&&m.indices.groups.run[1]===2&&copy.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn word_character_result_predecessor_search_preserves_complete_candidates_and_initial_pair_rewind()
{
    check(
        "var r=new RegExp('(?<=(?<run>\\\\B[\\\\q{😀|😀x|}&&\\\\p{Any}]\\\\B))','dgyv');r.lastIndex=3;var m=r.exec('😀😀');m.index===2&&m.groups.run==='😀'&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===2&&r.lastIndex===2",
    );
}
