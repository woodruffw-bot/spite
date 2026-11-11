//! Symbol/BigInt [[Construct]] capability differs from invoking that method.

use crate::common::assert_true;

#[test]
fn symbol_and_bigint_are_valid_new_targets_and_superclasses() {
    assert_true(
        r#"
            for (let C of [Symbol, BigInt]) {
                function F(value) {
                    this.value = value;
                    this.target = new.target
                }
                let x = Reflect.construct(F, [7], C);
                if (Object.getPrototypeOf(x) !== C.prototype || x.value !== 7 || x.target !== C || !(
                        x instanceof C)) throw 1;
                class D extends C {
                    constructor() {
                        return {
                            accepted: C
                        }
                    }
                }
                if (new D().accepted !== C || Object.getPrototypeOf(D) !== C || Object.getPrototypeOf(D
                        .prototype) !== C.prototype) throw 2;
                class Default extends C {}
                class Explicit extends C {
                    constructor() {
                        super()
                    }
                }
                for (let Child of [Default, Explicit]) {
                    let caught = false;
                    try {
                        new Child()
                    } catch (e) {
                        caught = e instanceof TypeError
                    }
                    if (!caught) throw 3;
                }
                let bound = C.bind(null),
                    y = Reflect.construct(F, [], bound);
                if (Object.getPrototypeOf(y) !== Object.prototype || y.target !== bound) throw 4;
            }
            true
        "#,
    );
}

#[test]
fn construction_rejects_before_coercion_or_new_target_prototype_lookup() {
    assert_true(
        r#"
            let log = [],
                value = new Proxy({}, {
                    get() {
                        throw 1
                    }
                });
            let NewTarget = new Proxy(function() {}, {
                get(target, key, receiver) {
                    if (key === 'prototype') throw 2;
                    return Reflect.get(target, key, receiver)
                }
            });
            let argumentsList = {
                get length() {
                    log.push('length');
                    return 1
                },
                get 0() {
                    log.push('value');
                    return value
                }
            };
            for (let C of [Symbol, BigInt]) {
                for (let attempt of [() => new C(value), () => Reflect.construct(C, argumentsList,
                        NewTarget)]) {
                    let caught = false;
                    try {
                        attempt()
                    } catch (e) {
                        caught = e instanceof TypeError
                    }
                    if (!caught) throw 3;
                }
            }
            log.join(',') === 'length,value,length,value'
        "#,
    );
}

#[test]
fn bound_proxy_capabilities_and_array_factories_use_the_constructor_predicate() {
    assert_true(
        r#"
            for (let C of [Symbol, BigInt]) {
                let pair = Proxy.revocable(C, {
                    construct(target, args, newTarget) {
                        return {
                            target,
                            args,
                            newTarget
                        }
                    }
                });
                let bound = pair.proxy.bind(null, 3),
                    result = new bound(4);
                if (result.target !== C || result.args.join(',') !== '3,4' || result.newTarget !== pair
                    .proxy) throw 1;
                pair.revoke();
                let outer = new Proxy(bound, {
                    construct() {
                        return {
                            ok: true
                        }
                    }
                });
                if (!new outer().ok) throw 2;
                for (let factory of [() => Array.of.call(C, 1, 2), () => Array.from.call(C, [3, 4])]) {
                    let caught = false;
                    try {
                        factory()
                    } catch (e) {
                        caught = e instanceof TypeError
                    }
                    if (!caught) throw 3;
                }
                let proxy = new Proxy(C, {
                    construct() {
                        return []
                    }
                });
                if (Array.of.call(proxy, 1, 2).join(',') !== '1,2' || Array.from.call(proxy, [3, 4]).join(
                        ',') !== '3,4') throw 4;
            }
            true
        "#,
    );
}
