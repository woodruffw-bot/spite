//! Verified Array index probes preserve the ordinary descriptor semantics.
use spite_runtime::{Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn array_index_lookup_shapes_snapshot() {
    let mut rows = String::new();
    for source in [
        "let a=[10,20,30];return [a[0],a[1],a[2],a[3],Reflect.ownKeys(a),Object.getOwnPropertyDescriptor(a,'1')]",
        "let a=[];a.note=1;a[2]=20;a[0]=10;a[1]=15;return [a[0],a[1],a[2],a[3],Reflect.ownKeys(a)]",
        "let a=[10,20,30];delete a[0];a[0]=40;return [a[0],a[1],a[2],Reflect.ownKeys(a)]",
        "let a=[10,20,30];delete a[1];return [a[0],a[1],a[2],Object.hasOwn(a,'1'),a.length]",
        "let a=[10,20,30];Object.defineProperty(a,'1',{get(){return 99},configurable:true});return [a[1],Object.getOwnPropertyDescriptor(a,'1').enumerable,Reflect.ownKeys(a)]",
        "let a=[];a[4294967294]=1;a[4294967295]=2;a['01']=3;a['-0']=4;return [a.length,a[4294967294],a[4294967295],a['01'],a['-0'],a[0],Reflect.ownKeys(a)]",
        "let a=[1,2,3];a.length=1;a[2]=9;return [a[0],a[1],a[2],a.length,Reflect.ownKeys(a)]",
    ] {
        let Value::String(result) = Realm::default()
            .eval(&format!(
                r#"
                JSON.stringify((() => {{
                    {source}
                }})())
            "#
            ))
            .unwrap()
        else {
            panic!("{source}")
        };
        writeln!(
            rows,
            "{source} => {result}",
            result = result.to_utf8().expect("JSON output must be valid UTF-8"),
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        let a=[10,20,30];return [a[0],a[1],a[2],a[3],Reflect.ownKeys(a),Object.getOwnPropertyDescriptor(a,'1')] => [10,20,30,null,["0","1","2","length"],{"value":20,"writable":true,"enumerable":true,"configurable":true}]
        let a=[];a.note=1;a[2]=20;a[0]=10;a[1]=15;return [a[0],a[1],a[2],a[3],Reflect.ownKeys(a)] => [10,15,20,null,["0","1","2","length","note"]]
        let a=[10,20,30];delete a[0];a[0]=40;return [a[0],a[1],a[2],Reflect.ownKeys(a)] => [40,20,30,["0","1","2","length"]]
        let a=[10,20,30];delete a[1];return [a[0],a[1],a[2],Object.hasOwn(a,'1'),a.length] => [10,null,30,false,3]
        let a=[10,20,30];Object.defineProperty(a,'1',{get(){return 99},configurable:true});return [a[1],Object.getOwnPropertyDescriptor(a,'1').enumerable,Reflect.ownKeys(a)] => [99,true,["0","1","2","length"]]
        let a=[];a[4294967294]=1;a[4294967295]=2;a['01']=3;a['-0']=4;return [a.length,a[4294967294],a[4294967295],a['01'],a['-0'],a[0],Reflect.ownKeys(a)] => [4294967295,1,2,3,4,null,["4294967294","length","4294967295","01","-0"]]
        let a=[1,2,3];a.length=1;a[2]=9;return [a[0],a[1],a[2],a.length,Reflect.ownKeys(a)] => [1,null,9,3,["0","2","length"]]
        "#);
}

#[test]
fn array_index_lookup_metadata_sparse_reordering_and_symbols() {
    assert_true(
        r#"
            let a = [];
            a.name = 'n';
            let s = Symbol('0');
            a[s] = 7;
            a[5] = 5;
            a[1] = 1;
            a[0] = 0;
            a[2] = 2;
            delete a[1];
            a[1] = 11;
            a[2] = 22;
            a[0] === 0 && a[1] === 11 && a[2] === 22 && a[3] === undefined && a[5] === 5 && a[s] === 7 && a
                .name === 'n' && Reflect.ownKeys(a).map(k => typeof k === 'symbol' ? 'symbol' : k).join(
                    ',') === '0,1,2,5,length,name,symbol'
        "#,
    );
    assert_true(
        r#"
            let a = [1, 2, 3];
            Object.defineProperty(a, '1', {
                value: 8,
                writable: false,
                configurable: false
            });
            let caught = false;
            try {
                Object.defineProperty(a, '1', {
                    value: 9
                })
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && a[1] === 8 && !Reflect.deleteProperty(a, '1') && a.length === 3
        "#,
    );
    assert_true(
        "let a=[1,2];a.extra=3;a.unshift(0);a.reverse();a.splice(1,1,9,8);a.join(',')==='2,9,8,0'&&a.extra===3",
    );
}

#[test]
fn array_index_lookup_inherited_accessors_and_readonly_length() {
    assert_true(
        r#"
            let t = '',
                p = Object.create(Array.prototype);
            Object.defineProperty(p, '1', {
                get() {
                    t += 'g';
                    return 7
                },
                set(v) {
                    t += 's' + v
                },
                configurable: true
            });
            let a = [];
            Object.setPrototypeOf(a, p);
            a[1] = 4;
            let v = a[1];
            t === 's4g' && v === 7 && !Object.hasOwn(a, '1') && a.length === 0
        "#,
    );
    assert_true(
        r#"
            let p = Object.create(Array.prototype);
            Object.defineProperty(p, '4', {
                value: 9,
                writable: false
            });
            let a = [1];
            Object.setPrototypeOf(a, p);
            !Reflect.set(a, '4', 10) && a[4] === 9 && a.length === 1 && !Object.hasOwn(a, '4')
        "#,
    );
    assert_true(
        r#"
            let a = [1, 2];
            Object.defineProperty(a, 'length', {
                writable: false
            });
            Reflect.set(a, '1', 9) && !Reflect.set(a, '2', 8) && a[1] === 9 && a.length === 2 && a[2] ===
                undefined
        "#,
    );
}

#[test]
fn array_index_lookup_canonical_keys_and_sparse_maximum() {
    assert_true(
        r#"
            let a = [];
            a['00'] = 10;
            a['1.0'] = 11;
            a['-0'] = 12;
            a['4294967295'] = 13;
            a['4294967296'] = 14;
            a['0'] = 15;
            a['1'] = 16;
            a.length === 2 && a[0] === 15 && a[1] === 16 && a['00'] === 10 && a['1.0'] === 11 && a[
                '-0'] === 12 && a[4294967295] === 13 && a[4294967296] === 14
        "#,
    );
    assert_true(
        r#"
            let a = [];
            a[4294967294] = 7;
            a[4294967293] = 6;
            delete a[4294967294];
            a[4294967294] = 8;
            a.length === 4294967295 && a[4294967292] === undefined && a[4294967293] === 6 && a[
                4294967294] === 8 && Reflect.ownKeys(a).join(',') === '4294967293,4294967294,length'
        "#,
    );
}

#[test]
fn array_index_lookup_consumers_gc_and_unlimited_dense_arrays() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    assert_eq!(
        realm.eval(
            r#"
                let a = [];
                for (let i = 0; i < 50000; i++) a[i] = i;
                let b = a.slice(0, 400).map(x => ({
                        value: x
                    })),
                    c = b.filter(x => x.value % 2 === 0).slice(100, 200);
                a.length === 50000 && a[49999] === 49999 && c.length === 100 && c[0].value === 200 && c[99]
                    .value === 398
            "#
        ),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a[49999]===49999&&b[399].value===399&&c[99].value===398"),
        Ok(Value::Boolean(true))
    );
    realm.eval("a=null;b=null;c=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}
