//! Positive Unicode lookahead keeps operand captures beyond its empty match.

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
fn single_character_q_classes_keep_original_source_names_and_independent_capture_ranges() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var atom of ['[\\q{a|😀}]','[[\\q{a|😀}]--[b]]','[[\\q{a|😀}]&&[a😀]]','[x\\q{a|😀}]']){
            var source='(?<outside>(?=(?<run>'+atom+')))',r=new RegExp(source,'d'+state+'v'),m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.groups.outside===''&&m.groups.run==='😀'&&m.indices.groups.outside[0]===0&&m.indices.groups.outside[1]===0&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===2&&r.lastIndex===0&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('a').groups.run==='a';
        }ok
    "#,
    );
}

#[test]
fn single_character_q_lookahead_retains_original_line_fold_and_surrogate_context() {
    check(
        r#"
        var ok=true;
        var m=/(?=(?<run>^[\q{a|😀}]$))/dimv.exec('\nA\n');ok=ok&&m.index===1&&m.groups.run==='A'&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2;
        for(var text of ['S','ſ','K','K']){var m=/(?=(?<run>[\q{ſ|k}]))/div.exec(text);ok=ok&&m.index===0&&m.groups.run===text&&m.indices.groups.run[1]===1}
        var m=/(?=(?<run>[\p{ASCII}\q{😀}]))/div.exec('ſ');ok=ok&&m.index===0&&m.groups.run==='ſ';
        var r=/(?=(?<run>[\q{\uD800|x}]))/dgv;ok=ok&&r.exec('😀')===null&&r.lastIndex===0;var m=r.exec('\ud800x');ok=ok&&m.index===0&&m.groups.run==='\ud800'&&m.indices.groups.run[1]===1;
        ok
    "#,
    );
}

#[test]
fn single_character_q_lookahead_string_consumers_copies_and_collection_keep_original_captures() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?=(?<run>[\\q{a|😀}])))',r=new RegExp(source,'dgv'),all=Array.from('😀a'.matchAll(r));
        var ok=all.length===2&&all[0].index===0&&all[0].groups.run==='😀'&&all[1].index===2&&all[1].groups.run==='a'&&r.lastIndex===0;
        ok=ok&&'😀a'.replace(r,'<$<run>>')==='<😀>😀<a>a'&&'😀a'.replaceAll(r,'<$<run>>')==='<😀>😀<a>a'&&JSON.stringify('😀a'.split(new RegExp('(?=([\\q{a|😀}]))','v')))==='["😀","a","a"]';
        var offsets=[];ok=ok&&'😀a'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&outside===''&&run===groups.run;return '!';})==='!😀!a'&&JSON.stringify(offsets)==='[0,2]';
        r.lastIndex=1;ok=ok&&'😀a'.search(r)===0&&r.lastIndex===1;
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('😀').groups.run==='😀';ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('a');m.index===0&&m.groups.outside===''&&m.groups.run==='a'&&m.indices.groups.run[1]===1&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn single_character_q_lookahead_deep_wrappers_and_stateful_pair_offsets_keep_exact_slots() {
    check(
        r#"
        var source='('.repeat(1000)+'(?='+'('.repeat(1000)+'[\\q{a|😀}]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'dgyv');r.lastIndex=2;var m=r.exec('!😀a');
        var ok=m.index===1&&m.length===2001&&m[0]===''&&r.lastIndex===1;
        for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':'😀')&&m.indices[i][0]===1&&m.indices[i][1]===(i<=1000?1:3);
        r.lastIndex=5;ok=ok&&r.exec('!😀a')===null&&r.lastIndex===0;ok
    "#,
    );
}

#[test]
fn word_q_classes_keep_original_polarity_source_and_named_capture_ranges() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['\\b[\\q{a|b}]\\b','ab a!',3,'a'],['\\b[\\q{a|b}]\\B','ab!',0,'a'],['\\B[\\q{a|😀}]\\B','😀😀',0,'😀']]){
            var source='(?<outside>(?=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=row[2];var m=r.exec(row[1]);
            ok=ok&&m.index===row[2]&&m[0]===''&&m.groups.outside===''&&m.groups.run===row[3]&&m.indices.groups.outside[0]===row[2]&&m.indices.groups.outside[1]===row[2]&&m.indices.groups.run[0]===row[2]&&m.indices.groups.run[1]===row[2]+row[3].length&&r.lastIndex===row[2]&&r.source===source;
            var copy=new RegExp(r);copy.lastIndex=row[2];ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===row[3];
        }ok
    "#,
    );
}

#[test]
fn word_q_class_line_fold_property_and_backspace_context_remain_original() {
    check(
        r#"
        var m=/(?=(?<run>^\b[\q{a|😀}]\b$))/dimv.exec('\nA\n'),ok=m.index===1&&m.groups.run==='A'&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===2;
        for(var text of ['S','ſ','K','K']){var m=/(?=(?<run>\b[\q{ſ|k}]\b))/div.exec('!'+text+'!');ok=ok&&m.index===1&&m.groups.run===text&&m.indices.groups.run[1]===2}
        var m=/(?=(?<run>\b[\p{ASCII}\q{😀}]\b))/div.exec('!ſ!');ok=ok&&m.index===1&&m.groups.run==='ſ';
        var m=/(?=(?<run>\B[\q{\u0008|a}]\B))/dv.exec('\b');ok=ok&&m.index===0&&m.groups.run==='\b'&&m.indices.groups.run[1]===1;ok
    "#,
    );
}

#[test]
fn word_q_class_string_consumers_copies_and_collection_keep_original_extents() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?=(?<run>\\b[\\q{a|b}]\\b)))',r=new RegExp(source,'dgv'),all=Array.from('!a! b'.matchAll(r));
        var ok=all.length===2&&all[0].index===1&&all[0].groups.run==='a'&&all[1].index===4&&all[1].groups.run==='b'&&r.lastIndex===0;
        ok=ok&&'!a! b'.replace(r,'<$<run>>')==='!<a>a! <b>b'&&JSON.stringify('!a! b'.split(new RegExp('(?=(\\b[\\q{a|b}]\\b))','v')))==='["!","a","a! ","b","b"]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!a!').groups.run==='a';ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('!b!');m.index===1&&m.groups.outside===''&&m.groups.run==='b'&&m.indices.groups.run[1]===2&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn word_q_class_search_preserves_complete_unicode_candidates_and_initial_pair_rewind() {
    check(
        "var r=new RegExp('(?=(?<run>\\\\B[\\\\q{😀|a}]\\\\B))','dgyv');r.lastIndex=1;var m=r.exec('😀😀');m.index===0&&m.groups.run==='😀'&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===2&&r.lastIndex===0",
    );
}

#[test]
fn named_operand_and_outside_captures_keep_independent_indices_and_greediness() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var row of [['a+','aaa'],['a+?','a'],['a{2,3}','aaa'],['a{2,3}?','aa']]){
            var source='(?<outside>(?=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+mode);r.lastIndex=1;var m=r.exec('baaa!');
            ok=ok&&m.index===1&&m[0]===''&&m[1]===''&&m[2]===row[1]&&m.groups.outside===''&&m.groups.run===row[1]&&r.lastIndex===1&&r.source===source;
            ok=ok&&JSON.stringify(m.indices[0])==='[1,1]'&&JSON.stringify(m.indices[1])==='[1,1]'&&m.indices[2][0]===1&&m.indices[2][1]===1+row[1].length&&m.indices.groups.run===m.indices[2];
        }
        for(var mode of ['u','v'])for(var row of [['a*','aaa'],['a*?',''],['a{0}','']]){
            var m=new RegExp('(?=(?<run>'+row[0]+'))','d'+mode).exec('aaa');ok=ok&&m[0]===''&&m.groups.run===row[1]&&m.indices[0][1]===0&&m.indices.groups.run[1]===row[1].length;
        }ok
    "#,
    );
}

#[test]
fn astral_operand_captures_use_leading_pair_candidates_and_empty_last_index() {
    check(
        r#"
        var ok=true,input='😀😀!';
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp('(?<outside>(?=(?<run>😀+)))','d'+state+mode);r.lastIndex=start;var m=r.exec(input),active=r.global||r.sticky,point=active?start:0;
            if(point===1||point===3)point--;
            if(point>=4){ok=ok&&m===null&&r.lastIndex===0;continue}
            ok=ok&&m.index===point&&m[0]===''&&m.groups.outside===''&&m.groups.run===input.slice(point,4)&&m.indices.groups.run[0]===point&&m.indices.groups.run[1]===4&&r.lastIndex===(active?point:start);
        }ok
    "#,
    );
}

#[test]
fn flags_and_positive_ascii_property_follow_original_context_and_fold_rules() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var m=new RegExp('(?=(?<run>^a+$))','dim'+mode).exec('x\nAA\nz');ok=ok&&m.index===2&&m.groups.run==='AA'&&m.indices.groups.run[1]===4;
            var m=new RegExp('(?=(?<run>.+?))','ds'+mode).exec('\n😀');ok=ok&&m.index===0&&m.groups.run==='\n'&&m.indices.groups.run[1]===1;
            var m=new RegExp('(?=(?<run>.+?))','d'+mode).exec('\n😀');ok=ok&&m.index===1&&m.groups.run==='😀'&&m.indices.groups.run[1]===3;
            var m=new RegExp('(?=(?<run>\\p{ASCII}+))','di'+mode).exec('ſKa');ok=ok&&m.index===0&&m.groups.run==='ſKa'&&m.indices.groups.run[1]===3;
        }ok
    "#,
    );
}

#[test]
fn string_consumers_observe_zero_width_progress_and_consuming_operand_values() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var r=new RegExp('(?=(?<run>a+))','dg'+mode),all=Array.from('aaa'.matchAll(r));
            ok=ok&&all.length===3&&all[0].groups.run==='aaa'&&all[1].groups.run==='aa'&&all[2].groups.run==='a'&&all[2].index===2&&r.lastIndex===0;
            ok=ok&&'aaa'.replace(r,'<$<run>>')==='<aaa>a<aa>a<a>a'&&'aaa'.replaceAll(r,'<$<run>>')==='<aaa>a<aa>a<a>a';
            var offsets=[];ok=ok&&'aaa'.replace(r,function(match,run,offset,input,groups){offsets.push(offset);ok=ok&&match===''&&run===input.slice(offset)&&groups.run===run;return '!';})==='!a!a!a'&&JSON.stringify(offsets)==='[0,1,2]';
            r.lastIndex=3;ok=ok&&'baaa'.search(r)===1&&r.lastIndex===3;
            var split='baaac'.split(new RegExp('(?=(a+))',mode));ok=ok&&JSON.stringify(split)==='["b","aaa","a","aa","a","a","ac"]';
        }ok
    "#,
    );
}

#[test]
fn cloning_and_collection_preserve_named_operand_capture_storage() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?=(?<run>a+)))/dgu;r.lastIndex=3;var copy=new RegExp(r),ok=copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags;
        var m=copy.exec('baaa');ok&&m.index===1&&m.groups.outside===''&&m.groups.run==='aaa'&&m.indices.groups.outside[1]===1&&m.indices.groups.run[1]===4&&r.lastIndex===3
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;var m=copy.exec('baaa');m.index===2&&m.groups.outside===''&&m.groups.run==='aa'&&m.indices.groups.run[1]===4&&copy.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn deep_wrappers_and_unrepresentable_counts_keep_original_capture_inventory() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='('.repeat(1000)+'(?='+'('.repeat(1000)+'a+'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'d'+mode),m=r.exec('baaa');
            ok=ok&&m.length===2001&&m.index===1&&m[0]==='';
            for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':'aaa')&&m.indices[i][0]===1&&m.indices[i][1]===(i<=1000?1:4);
            var huge=new RegExp('(?=(?<run>a{'+'9'.repeat(20000)+'}))','dg'+mode);huge.lastIndex=1;ok=ok&&huge.exec('aaaa')===null&&huge.lastIndex===0;
        }ok
    "#,
    );
}

#[test]
fn coercions_last_index_errors_and_original_validation_precede_execution() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var events=[],r=new RegExp('(?=(?<run>a+))','dgy'+mode);r.lastIndex={valueOf(){events.push('index');return 1}};
            var m=r.exec({toString(){events.push('input');return 'baaa'}});ok=ok&&events.join(',')==='input,index'&&m.groups.run==='aaa'&&r.lastIndex===1;
            Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('baaa')}catch(e){caught=e instanceof TypeError}ok=ok&&caught;
            for(var source of ['(?=(a{2,1}))','(?=(?<x>a+))\\k<missing>','(?=(\\p{Not_A_Property}+))']){var caught=false;try{new RegExp(source,mode)}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}
        }ok
    "#,
    );
}

#[test]
fn opted_in_construction_abort_cannot_publish_partial_capture_state() {
    let pattern = JsString::from(format!("(?=(?<run>a{{{}}}))", "9".repeat(20_000)).as_str());
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=new RegExp({pattern:?},'dgu')}}catch(e){{marker=1}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7&&marker===0"), Ok(Value::Boolean(true)));
}

#[test]
fn word_endpoints_select_lazy_captures_and_keep_original_outside_slots() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy']){
            var source='(?<outside>(?=(?<run>\\ba+?\\b)))',r=new RegExp(source,'d'+state+mode);r.lastIndex=1;var m=r.exec('!aaa!');
            ok=ok&&m.index===1&&m[0]===''&&m.groups.outside===''&&m.groups.run==='aaa'&&m.indices.groups.outside[1]===1&&m.indices.groups.run[0]===1&&m.indices.groups.run[1]===4&&r.lastIndex===1&&r.source===source;
        }
        for(var mode of ['u','v'])for(var row of [['a+','aa'],['a+?','a']]){
            var m=new RegExp('(?=(?<run>\\B'+row[0]+'\\B))','d'+mode).exec('aaaa');ok=ok&&m.index===1&&m.groups.run===row[1]&&m.indices.groups.run[1]===1+row[1].length;
        }ok
    "#,
    );
}

#[test]
fn word_lookahead_flags_empty_operands_and_string_consumers_keep_original_context() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var m=new RegExp('(?=(?<run>^\\b.+?\\b$))','dim'+mode).exec('\nAA\n');ok=ok&&m.index===1&&m.groups.run==='AA'&&m.indices.groups.run[1]===3;
            var m=new RegExp('(?=(?<run>\\bs+?\\b))','di'+mode).exec('!ſſ!');ok=ok&&m.index===1&&m.groups.run==='ſſ'&&m.indices.groups.run[1]===3;
            var m=new RegExp('(?=(?<run>\\B.*?\\B))','d'+mode).exec('!a');ok=ok&&m.index===0&&m.groups.run===''&&m.indices.groups.run[1]===0;
            var r=new RegExp('(?=(?<run>\\ba+?\\b))','dg'+mode),all=Array.from('!aaa!'.matchAll(r));ok=ok&&all.length===1&&all[0].index===1&&all[0].groups.run==='aaa'&&r.lastIndex===0;
            ok=ok&&'!aaa!'.replace(r,'<$<run>>')==='!<aaa>aaa!'&&JSON.stringify('!aaa!'.split(new RegExp('(?=(\\ba+?\\b))',mode)))==='["!","aaa","aaa!"]';
        }ok
    "#,
    );
}

#[test]
fn word_lookahead_copies_and_collection_retain_operand_extents() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?=(?<run>\ba+?\b)))/dgv;r.lastIndex=3;var copy=new RegExp(r),m=copy.exec('!aaa!');
        m.index===1&&m.groups.outside===''&&m.groups.run==='aaa'&&m.indices.groups.run[1]===4&&copy.lastIndex===1&&r.lastIndex===3
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('!a!');m.index===1&&m.groups.run==='a'&&m.groups.outside===''&&m.indices.groups.run[1]===2&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn word_lookahead_search_uses_complete_unicode_candidates_and_initial_pair_rewind() {
    // Independent specification expectation: V8 can search inside this pair.
    check(
        "var ok=true;for(var mode of ['u','v']){var r=new RegExp('(?=(?<run>\\\\Bſ*?\\\\B))','dg'+mode);var m=r.exec('a😀ab!');ok=ok&&m.index===4&&m.groups.run===''&&m.indices.groups.run[0]===4&&r.lastIndex===4;r.lastIndex=2;var m=r.exec('a😀ab!');ok=ok&&m.index===4;r=new RegExp('(?=(?<run>\\\\ba+\\\\b))','dgy'+mode);r.lastIndex=1;ok=ok&&r.exec('😀a!')===null&&r.lastIndex===0}ok",
    );
}

#[test]
fn bare_atoms_capture_one_character_and_preserve_original_source_and_names() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var row of [['a','a'],['[a-z]','a'],['\\d','2'],['.','😀'],['\\u{1F600}','😀'],['\\uD83D\\uDE00','😀'],['\\uD800','\ud800']]){
            var source='(?<outside>(?=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+mode),m=r.exec(row[1]);
            ok=ok&&m!==null&&m.index===0&&m[0]===''&&m.groups.outside===''&&m.groups.run===row[1]&&m.indices.groups.outside[0]===0&&m.indices.groups.outside[1]===0&&m.indices.groups.run[0]===0&&m.indices.groups.run[1]===row[1].length&&r.source===source&&r.lastIndex===0;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===row[1];
        }ok
    "#,
    );
}

#[test]
fn bare_atom_line_dot_property_and_lone_surrogate_contexts_are_preserved() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var m=new RegExp('(?=(?<run>^a$))','dim'+mode).exec('\nA\n');ok=ok&&m.index===1&&m.groups.run==='A'&&m.indices.groups.run[1]===2;
            var m=new RegExp('(?=(?<run>^\\$$))','d'+mode).exec('$');ok=ok&&m.index===0&&m.groups.run==='$'&&m.indices.groups.run[1]===1;
            var m=new RegExp('(?=(?<run>\\p{ASCII}))','di'+mode).exec('ſa');ok=ok&&m.index===0&&m.groups.run==='ſ'&&m.indices.groups.run[1]===1;
            var m=new RegExp('(?=(?<run>.))','ds'+mode).exec('\na');ok=ok&&m.index===0&&m.groups.run==='\n';
            var r=new RegExp('(?=(?<run>\\uD800))','dg'+mode);ok=ok&&r.exec('\ud800\udc00')===null&&r.lastIndex===0;
            var m=r.exec('\ud800x\udc00');ok=ok&&m.index===0&&m.groups.run==='\ud800'&&m.indices.groups.run[1]===1;
        }ok
    "#,
    );
}

#[test]
fn bare_atom_string_consumers_progress_over_complete_input_characters() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var r=new RegExp('(?<outside>(?=(?<run>.)))','dg'+mode),all=Array.from('😀a'.matchAll(r));
            ok=ok&&all.length===2&&all[0].index===0&&all[0].groups.run==='😀'&&all[0].groups.outside===''&&all[1].index===2&&all[1].groups.run==='a'&&r.lastIndex===0;
            ok=ok&&'😀a'.replace(r,'<$<run>>')==='<😀>😀<a>a'&&JSON.stringify('😀a'.split(new RegExp('(?=(.))',mode)))==='["😀","a","a"]';
        }ok
    "#,
    );
}

#[test]
fn bare_atom_deep_wrappers_keep_empty_outside_and_full_operand_capture_ranges() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='('.repeat(1000)+'(?='+'('.repeat(1000)+'[a-z]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'d'+mode),m=r.exec('!!a');
            ok=ok&&m.index===2&&m.length===2001&&m[0]==='';
            for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':'a')&&m.indices[i][0]===2&&m.indices[i][1]===(i<=1000?2:3);
        }ok
    "#,
    );
}

#[test]
fn bare_word_atoms_keep_polarity_source_and_independent_named_ranges() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var row of [['\\ba\\B','aaa'],['\\Ba\\B','aaaa'],['\\b[a-z]\\b','!a!']]){
            var source='(?<outside>(?=(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+mode);r.lastIndex=row[0]==='\\ba\\B'?0:1;var m=r.exec(row[1]),point=row[0]==='\\ba\\B'?0:1;
            ok=ok&&m.index===point&&m[0]===''&&m.groups.outside===''&&m.groups.run==='a'&&m.indices.groups.outside[0]===point&&m.indices.groups.outside[1]===point&&m.indices.groups.run[0]===point&&m.indices.groups.run[1]===point+1&&r.source===source&&r.lastIndex===point;
        }ok
    "#,
    );
}

#[test]
fn bare_word_atom_line_fold_and_class_backspace_contexts_are_preserved() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var m=new RegExp('(?=(?<run>^\\ba\\b$))','dim'+mode).exec('\nA\n');ok=ok&&m.index===1&&m.groups.run==='A'&&m.indices.groups.run[1]===2;
            var m=new RegExp('(?=(?<run>\\bſ\\b))','di'+mode).exec('!ſ!');ok=ok&&m.index===1&&m.groups.run==='ſ'&&m.indices.groups.run[1]===2;
            var m=new RegExp('(?=(?<run>\\b\\p{ASCII}\\b))','di'+mode).exec('!ſ!');ok=ok&&m.index===1&&m.groups.run==='ſ';
            var m=new RegExp('(?=(?<run>\\B[\\b]\\B))','d'+mode).exec('\b');ok=ok&&m.index===0&&m.groups.run==='\b'&&m.indices.groups.run[1]===1;
        }ok
    "#,
    );
}

#[test]
fn bare_word_atom_string_consumers_copies_and_collection_keep_one_character() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='(?=(?<run>\\ba\\b))',r=new RegExp(source,'dg'+mode),all=Array.from('!!a! a'.matchAll(r));
            ok=ok&&all.length===2&&all[0].index===2&&all[1].index===5&&all[0].groups.run==='a'&&r.lastIndex===0;
            ok=ok&&'!!a! a'.replace(r,'<$<run>>')==='!!<a>a! <a>a'&&JSON.stringify('!!a! a'.split(new RegExp('(?=(\\ba\\b))',mode)))==='["!!","a","a! ","a","a"]';
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!a!').groups.run==='a';
        }ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('!a!');m.index===1&&m.groups.run==='a'&&m.indices.groups.run[1]===2&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn bare_word_atom_dot_search_excludes_interior_surrogates_and_rewinds_initial_pairs() {
    // V8's ordinary search can return the low half here; the candidate is 4.
    check(
        "var ok=true;for(var mode of ['u','v']){var r=new RegExp('(?=(?<run>\\\\B.\\\\b))','dg'+mode),m=r.exec('a😀ab!');ok=ok&&m.index===4&&m.groups.run==='b'&&m.indices.groups.run[0]===4&&m.indices.groups.run[1]===5&&r.lastIndex===4;r=new RegExp('(?=(?<run>\\\\B.\\\\b))','dgy'+mode);r.lastIndex=2;ok=ok&&r.exec('a😀ab!')===null&&r.lastIndex===0}ok",
    );
}
