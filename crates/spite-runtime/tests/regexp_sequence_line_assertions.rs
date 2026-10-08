//! Fixed consuming bodies carry input/line assertions at exact UTF-16 offsets.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn internal_line_assertions_use_complete_input_positions_and_empty_capture_ranges() {
    check(
        r"let a=/(a)($)(\n)(^)(b)/dm.exec('xa\nbx');a.index===1&&a[0]==='a\nb'&&a[1]==='a'&&a[2]===''&&a[3]==='\n'&&a[4]===''&&a[5]==='b'&&a.indices[2][0]===2&&a.indices[4][0]===3",
    );
    check(
        r"let a=/(a)($)(\n)(^)(b)/d.exec('a\nb');a===null&&/a^b/.exec('ab')===null&&/a$b/.exec('ab')===null",
    );
    check(
        r"let a=/(^a)/d.exec('a'),b=/(a$)/d.exec('a');a[1]==='a'&&b[1]==='a'&&a.indices[1][0]===0&&b.indices[1][1]===1",
    );
}

#[test]
fn multiline_handles_each_line_terminator_and_every_crlf_position() {
    check(
        r"let a=/(\r)(^)(\n)(^)(a)/dm.exec('x\r\na');a.index===1&&a[1]==='\r'&&a[2]===''&&a[3]==='\n'&&a[4]===''&&a[5]==='a'&&a.indices[2][0]===2&&a.indices[4][0]===3",
    );
    check(
        r"let a=/(a)($)(\r)($)(\n)/dm.exec('a\r\n');a[2]===''&&a[4]===''&&a.indices[2][0]===1&&a.indices[4][0]===2",
    );
    check(
        r"let a=/(\u2028)(^)(a)/m.exec('\u2028a'),b=/(a)($)(\u2029)/m.exec('a\u2029');a[2]===''&&a[3]==='a'&&b[1]==='a'&&b[2]===''",
    );
}

#[test]
fn dotall_ignore_case_word_and_input_assertions_keep_independent_flags() {
    check(
        r"/(.)(^)(.)/ms.exec('\nA')[0]==='\nA'&&/(.)(^)(.)/m.exec('\nA')===null&&/(.)(^)(.)/s.exec('\nA')===null",
    );
    check(
        r"let a=/(\r)(^\b)(a)/dmi.exec('\rA');a[1]==='\r'&&a[2]===''&&a[3]==='A'&&a.indices[2][0]===1&&/(\r)(^\B)(a)/mi.exec('\rA')===null",
    );
    check(
        r"let a=/(\^)(a)(\$)/d.exec('^a$');a[1]==='^'&&a[3]==='$'&&/([\^$])(^)(a)/m.exec('^a')===null",
    );
}

#[test]
fn sticky_assertions_keep_input_identity_and_ordered_last_index_writes() {
    check(
        r"let r=/(^a)/my;r.lastIndex=2;let a=r.exec('x\na');a.index===2&&a[1]==='a'&&r.lastIndex===3&&r.exec('x\na')===null&&r.lastIndex===0",
    );
    check(r"let r=/(^a)/y;r.lastIndex=1;r.exec('xaa')===null&&r.lastIndex===0");
    check(
        r"let r=/(a$)/my;r.lastIndex=1;let a=r.exec('xa\ny');a.index===1&&a[1]==='a'&&r.lastIndex===2",
    );
    check(
        r"let r=/(^a)/g;Object.defineProperty(r,'lastIndex',{writable:false});let failed=false;try{r.exec('a')}catch(e){failed=e instanceof TypeError}failed",
    );
}

#[test]
fn zero_width_assertion_groups_nested_outer_anchors_and_alternatives_keep_slots() {
    check(
        r"let a=/(^$\B)()/d.exec('');a[0]===''&&a[1]===''&&a[2]===''&&a.indices[1][0]===0&&a.indices[2][1]===0",
    );
    check(
        r"let a=/^((^a))$/d.exec('a'),b=/^((a$))$/d.exec('a');a[1]==='a'&&a[2]==='a'&&b[1]==='a'&&b[2]==='a'&&a.indices[2][1]===1",
    );
    check(
        r"let a=/(a$\n^b)|(a\nb)/dm.exec('a\nb');a[1]==='a\nb'&&a[2]===undefined&&Object.hasOwn(a,'2')&&a.indices[2]===undefined",
    );
    check(
        r"let a=/([\s\S])(^)(a)/dm.exec('💩\na');a.index===2&&a[1]==='\n'&&a[2]===''&&a[3]==='a'&&a.indices[3][0]===3",
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_arrays_preserve_empty_assertion_captures() {
    check(
        r"'a\nb a\nb'.replace(/(a)($)(\n)(^)(b)/gm,'$5$1')==='ba ba'&&'a\nbxa\nb'.split(/(a)($)(\n)(^)(b)/m).join(',')===',a,,\n,,b,x,a,,\n,,b,'",
    );
    check(
        r"let seen;let s='a\nb'.replace(/(a)($)(\n)(^)(b)/m,(whole,a,end,nl,start,b,index,input)=>{seen=[whole,a,end,nl,start,b,index,input];return 'x'});s==='x'&&seen[2]===''&&seen[4]===''&&seen[6]===0&&seen[7]==='a\nb'",
    );
    check(
        r"let r=/(a)($)(\n)(^)(b)/dgm,a=[...'a\nb a\nb'.matchAll(r)];a.length===2&&a[1].index===4&&a[1].indices[4][0]===6&&r.lastIndex===0",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(^)a/d.exec('a');calls===0&&a[1]===''&&Object.hasOwn(a,'1')&&a.indices[1][1]===0",
    );
}

#[test]
fn large_flat_input_assertions_copies_and_long_searches_survive_collection_without_limits() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('a('+'$'.repeat(100000)+')','dgm'),copy=new RegExp(r),s='x'.repeat(100000)+'a\\n'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a.index===100000&&a[1]===''&&a.indices[1][0]===100001&&copy.lastIndex===100001&&r.lastIndex===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
    assert_eq!(
        realm.eval("new RegExp('a('+'$'.repeat(100000)+')').exec(s)===null"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn optional_work_and_variable_assertion_gaps_remain_distinct_from_exceptions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(a$)/m,s='x'.repeat(2000),flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/(?:(?:a+($)b+){2}){2}/m.test('ab')",
        r"/(?:(?:a^b+c+){2}){2}/m.test('abc')",
        r"/(a$|b)+/m.test('a')",
        r"/(?:(?:(a|bc)^){2}){2}/m.test('a')",
        r"/(?:(?<n>a)^(?:\k<n>)+){2}/.test('a')",
        r"/(^a)/u.test('a')",
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
