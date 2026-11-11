//! A finite overflowing upper bound retains positive-minimum capture priority.

use crate::common::assert_true;

#[test]
fn unicode_overflowing_maximum_nullable_alternative_names_scopes_and_surrogates() {
    assert_true(
        r"let p='(a|){0003,000999999999999999999999999}?',r=new RegExp(p,'dvg'),c=new RegExp(r),m=c.exec('aaaa');c.source===p&&c.flags==='dgv'&&m[0]==='aaa'&&m[1]==='a'",
    );
}
