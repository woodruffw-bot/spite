//! A finite overflowing upper bound retains positive-minimum capture priority.
use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn unicode_overflowing_maximum_nullable_alternative_names_scopes_and_surrogates() {
    check(
        r"let p='(a|){0003,000999999999999999999999999}?',r=new RegExp(p,'dvg'),c=new RegExp(r),m=c.exec('aaaa');c.source===p&&c.flags==='dgv'&&m[0]==='aaa'&&m[1]==='a'",
    );
}
