//! Native RegExp construction, original slots and matching boundaries.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};

use crate::common::assert_true;

#[test]
fn default_patterns_flags_metadata_and_lastindex_have_standard_descriptors() {
    assert_true(
        r#"
            let a = RegExp(),
                b = new RegExp(undefined, undefined),
                c = RegExp('', 'ysgdim'),
                d = Object.getOwnPropertyDescriptor(c, 'lastIndex');
            a !== b && a.source === '(?:)' && a.flags === '' && c.flags === 'dgimsy' && c.dotAll && c
                .global && c.hasIndices && c.ignoreCase && c.multiline && c.sticky && !c.unicode && !c
                .unicodeSets && d.value === 0 && 1 / d.value === Infinity && d.writable && !d.enumerable &&
                !d.configurable && Object.getOwnPropertyNames(c).join(',') === 'lastIndex' && Object
                .getPrototypeOf(c) === RegExp.prototype && c instanceof RegExp && c.toString() ===
                '/(?:)/dgimsy'
        "#,
    );
    for flag in ["d", "g", "i", "m", "s", "u", "v", "y"] {
        assert_true(&format!(
            "let r=new RegExp('a','{flag}');r.flags==='{flag}' && r.source==='a' && r.lastIndex===0"
        ));
    }
    assert_true(
        r#"
            RegExp(null).source === 'null' && RegExp(7).source === '7' && RegExp(true).source === 'true' &&
                RegExp(12n).source === '12'
        "#,
    );
}

#[test]
fn identity_requires_a_call_truthy_match_undefined_flags_and_the_active_constructor() {
    assert_true(
        r#"
            let r = new RegExp('a', 'g');
            RegExp(r) === r && RegExp(r, undefined) === r && new RegExp(r) !== r && RegExp(r, '') !== r &&
                RegExp.call({}, r) === r && RegExp(r).lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = {
                    get [Symbol.match]() {
                        t += 'm';
                        return 1;
                    },
                    get constructor() {
                        t += 'c';
                        return RegExp;
                    },
                    get source() {
                        throw 7;
                    },
                    get flags() {
                        throw 8;
                    }
                };
            RegExp(r) === r && t === 'mc'
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = {
                    get [Symbol.match]() {
                        t += 'm';
                        return false;
                    },
                    get constructor() {
                        throw 7;
                    },
                    toString() {
                        t += 's';
                        return 'a';
                    }
                };
            RegExp(r).source === 'a' && t === 'ms'
        "#,
    );
    assert_true(
        r#"
            let r = new RegExp('a');
            r.lastIndex = 9;
            r.constructor = function() {};
            let copy = RegExp(r);
            copy !== r && copy.source === 'a' && copy.lastIndex === 0
        "#,
    );
}

#[test]
fn regexp_like_property_gets_precede_all_string_coercions_and_explicit_flags_skip_get() {
    assert_true(
        r#"
            let t = '',
                r = {
                    get [Symbol.match]() {
                        t += 'm';
                        return true;
                    },
                    get constructor() {
                        t += 'c';
                        return {};
                    },
                    get source() {
                        t += 's';
                        return {
                            [Symbol.toPrimitive](h) {
                                t += h;
                                return 'a';
                            }
                        };
                    },
                    get flags() {
                        t += 'f';
                        return {
                            [Symbol.toPrimitive](h) {
                                t += h;
                                return 'g';
                            }
                        };
                    }
                };
            let a = RegExp(r);
            a.source === 'a' && a.flags === 'g' && t === 'mcsfstringstring'
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = {
                    get [Symbol.match]() {
                        t += 'm';
                        return true;
                    },
                    get constructor() {
                        throw 8;
                    },
                    get source() {
                        t += 's';
                        return {
                            toString() {
                                t += 'p';
                                return 'a';
                            }
                        };
                    },
                    get flags() {
                        throw 7;
                    }
                },
                f = {
                    toString() {
                        t += 'f';
                        return 'i';
                    }
                };
            let a = new RegExp(r, f);
            a.ignoreCase && t === 'mspf'
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = {
                    get [Symbol.match]() {
                        t += 'm';
                        return true;
                    },
                    get source() {
                        t += 's';
                        return {
                            toString() {
                                t += 'p';
                                throw 7;
                            }
                        };
                    },
                    get flags() {
                        t += 'f';
                        return 'g';
                    }
                };
            try {
                new RegExp(r);
            } catch (e) {
                t += e;
            }
            t === 'msfp7'
        "#,
    );
}

#[test]
fn newtarget_prototype_get_occurs_between_pattern_gets_and_string_conversion() {
    assert_true(
        r#"
            let t = '',
                prototype = {},
                r = {
                    get [Symbol.match]() {
                        t += 'm';
                        return true;
                    },
                    get source() {
                        t += 's';
                        return {
                            toString() {
                                t += 'p';
                                return 'a';
                            }
                        };
                    },
                    get flags() {
                        t += 'f';
                        return {
                            toString() {
                                t += 'q';
                                return 'g';
                            }
                        };
                    }
                };
            let Target = (function() {}).bind(null);
            Object.defineProperty(Target, 'prototype', {
                get() {
                    t += 'n';
                    return prototype;
                }
            });
            let result = Reflect.construct(RegExp, [r], Target);
            t === 'msfnpq' && Object.getPrototypeOf(result) === prototype && Object
                .getOwnPropertyDescriptor(RegExp.prototype, 'source').get.call(result) === 'a'
        "#,
    );
    assert_true(
        r#"
            let Target = (function() {}).bind(null);
            Object.defineProperty(Target, 'prototype', {
                get() {
                    throw 7;
                }
            });
            let t = '';
            try {
                Reflect.construct(RegExp, [{
                    toString() {
                        t += 's';
                        throw 8;
                    }
                }], Target);
            } catch (e) {
                t += e;
            }
            t === '7'
        "#,
    );
    assert_true(
        r#"
            let Target = (function() {}).bind(null);
            Target.prototype = null;
            let r = Reflect.construct(RegExp, ['a'], Target);
            Object.getPrototypeOf(r) === RegExp.prototype && r.source === 'a'
        "#,
    );
    assert_true(
        r#"
            class R extends RegExp {
                x = 7;
            }
            let r = new R('a', 'g');
            r instanceof R && r instanceof RegExp && r.x === 7 && r.source === 'a' && r.global && r
                .lastIndex === 0
        "#,
    );
}

#[test]
fn cloning_uses_original_slots_after_match_override_and_ignores_public_overrides() {
    assert_true(
        r#"
            let r = new RegExp('/\n', 'yg');
            r.lastIndex = 7;
            Object.defineProperty(r, 'source', {
                get() {
                    throw 7;
                }
            });
            Object.defineProperty(r, 'flags', {
                get() {
                    throw 8;
                }
            });
            r[Symbol.match] = false;
            Object.defineProperty(r, 'constructor', {
                get() {
                    throw 9;
                }
            });
            let a = RegExp(r),
                b = new RegExp(r, 'i');
            a !== r && a.source === '\\/\\n' && a.flags === 'gy' && a.lastIndex === 0 && b.source === a
                .source && b.flags === 'i'
        "#,
    );
    assert_true(
        r#"
            let r = new RegExp('[/]');
            r[Symbol.match] = false;
            let caught = false;
            try {
                new RegExp(r, 'v');
            } catch (e) {
                caught = e instanceof SyntaxError;
            }
            caught
        "#,
    );
}

#[test]
fn invalid_flags_and_patterns_are_catchable_syntax_errors_after_ordered_coercion() {
    for flags in [
        "z", "gg", "uu", "uv", "vu", "gG", r"\\u0067", "null", "true", "0", r"\ud800",
    ] {
        assert_true(&format!(
            "let ok=false;try{{new RegExp('a','{flags}');}}catch(e){{ok=e instanceof SyntaxError;}}ok"
        ));
    }
    for pattern in ["(", "a{3,2}", "[z-a]", "(?<a>x)(?<a>y)", "\\\\8", "(?ii:a)"] {
        assert_true(&format!(
            "let ok=false;try{{new RegExp('{pattern}');}}catch(e){{ok=e instanceof SyntaxError;}}ok"
        ));
    }
    assert_true(
        r#"
            let t = '';
            try {
                RegExp({
                    toString() {
                        t += 'p';
                        return '(';
                    }
                }, {
                    toString() {
                        t += 'f';
                        return 'z';
                    }
                });
            } catch (e) {
                t += e instanceof SyntaxError;
            }
            t === 'pftrue'
        "#,
    );
    assert_true(
        r#"
            let t = '';
            try {
                RegExp({
                    toString() {
                        t += 'p';
                        throw 7;
                    }
                }, {
                    toString() {
                        t += 'f';
                        throw 8;
                    }
                });
            } catch (e) {
                t += e;
            }
            t === 'p7'
        "#,
    );
    assert_true(
        "let ok=false;try{RegExp(Symbol(),{toString(){throw 7;}});}catch(e){ok=e instanceof TypeError;}ok",
    );
}

#[test]
fn source_getters_escape_delimiters_terminators_and_preserve_surrogates() {
    assert_true(
        r"new RegExp('/').source==='\\/' && new RegExp('\n\r\u2028\u2029').source==='\\n\\r\\u2028\\u2029' && new RegExp('a(b|c)+').source==='a(b|c)+'",
    );
    let mut realm = Realm::default();
    assert_eq!(
        realm.eval("new RegExp('\\ud800').source"),
        Ok(Value::String(JsString::from_code_units(vec![0xd800])))
    );
    assert_eq!(
        realm.eval("new RegExp('\\ud800\\udc00','u').source"),
        Ok(Value::String(JsString::from_code_units(vec![
            0xd800, 0xdc00
        ])))
    );
    for name in [
        "source",
        "global",
        "ignoreCase",
        "hasIndices",
        "multiline",
        "dotAll",
        "unicode",
        "unicodeSets",
        "sticky",
    ] {
        assert_true(&format!(
            "let get=Object.getOwnPropertyDescriptor(RegExp.prototype,'{name}').get,r=new RegExp('a','g'),ok=false;try{{get.call(Object.create(r));}}catch(e){{ok=e instanceof TypeError;}}ok"
        ));
    }
}

#[test]
fn isregexp_and_object_tag_use_own_native_brands_after_symbol_overrides() {
    assert_true(
        r#"
            let r = new RegExp('a');
            r[Symbol.match] = undefined;
            let caught = false;
            try {
                'a'.includes(r);
            } catch (e) {
                caught = e instanceof TypeError;
            }
            caught && Object.prototype.toString.call(r) === '[object RegExp]' && Object.prototype.toString
                .call(Object.create(r)) === '[object Object]' && Object.prototype.toString.call(RegExp
                    .prototype) === '[object Object]'
        "#,
    );
    assert_true(
        r#"
            let r = new RegExp('a');
            r[Symbol.match] = false;
            !''.includes(r) && Object.prototype.toString.call(r) === '[object RegExp]' && (r[Symbol
                .toStringTag] = 'custom', Object.prototype.toString.call(r) === '[object custom]')
        "#,
    );
    assert_true(
        "let r=new RegExp('a');r[Symbol.toStringTag]=7;Object.prototype.toString.call(r)==='[object RegExp]'",
    );
}

#[test]
fn native_exec_validates_brand_and_coerces_input_before_the_matching_boundary() {
    assert_true(
        r#"
            let called = false;
            try {
                RegExp.prototype.exec.call({}, {
                    toString() {
                        called = true;
                        throw 7;
                    }
                });
            } catch (e) {
                called = called || !(e instanceof TypeError);
            }!called
        "#,
    );
    assert_true(
        "let r=new RegExp('a'),t='';try{r.exec({toString(){t+='s';throw 7;}});}catch(e){t+=e;}t==='s7'",
    );
    assert_true(
        r#"
            let m = new RegExp('(?:(?:(?:(?:a(a|bc)){2}){2})|){2,3}').exec('a');
            m.index === 0 && m[0] === '' && m[1] === undefined
        "#,
    );
    assert_true(
        r#"
            let r = new RegExp('(?:(?:(?:(?:a(a|bc)){2}){2})|){2,3}');
            r.exec = undefined;
            RegExp.prototype.test.call(r, 'a') === true
        "#,
    );
    assert_true(
        r#"
            let r = new RegExp('a');
            r.exec = function(s) {
                return {
                    0: s,
                    index: 3,
                    length: 1
                };
            };
            r.test('x') && r[Symbol.search]('x') === 3 && r[Symbol.match]('x')[0] === 'x'
        "#,
    );
}

#[test]
fn lastindex_and_native_getters_ignore_inherited_setters_and_public_forgeries() {
    assert_true(
        r#"
            Object.defineProperty(Object.prototype, 'lastIndex', {
                set() {
                    throw 7;
                }
            });
            let r = new RegExp('a', 'g');
            r.lastIndex = 2;
            Object.freeze(r);
            let d = Object.getOwnPropertyDescriptor(r, 'lastIndex');
            d.value === 2 && !d.writable && !d.enumerable && !d.configurable
        "#,
    );
    assert_true(
        r#"
            let r = new RegExp('a', 'g'),
                get = Object.getOwnPropertyDescriptor(RegExp.prototype, 'global').get;
            Object.defineProperty(r, 'global', {
                value: false
            });
            Object.defineProperty(r, 'source', {
                value: 'forged'
            });
            Object.defineProperty(r, 'flags', {
                value: 'z'
            });
            get.call(r) && RegExp.prototype.toString.call(r) === '/forged/z' && new RegExp(r).source ===
                'a' && new RegExp(r).global
        "#,
    );
    assert_true(
        r#"
            let r = new RegExp('a');
            r.lastIndex = -0;
            let same = RegExp(r),
                copy = new RegExp(r);
            same === r && 1 / same.lastIndex === -Infinity && 1 / copy.lastIndex === Infinity
        "#,
    );
}

#[test]
fn every_argument_is_evaluated_before_identity_return_or_constructor_validation() {
    assert_true(
        "let r=new RegExp('a'),t='';RegExp(r,(t+='f',undefined),(t+='extra',7))===r && t==='fextra'",
    );
    assert_true(
        r#"
            let t = '',
                r = new RegExp((t += 'p', 'a'), (t += 'f', 'g'), (t += 'extra', 7));
            t === 'pfextra' && r.source === 'a' && r.global
        "#,
    );
    assert_true(
        r#"
            let t = '';
            try {
                new RegExp((t += 'p', '('), undefined, (() => {
                    t += 'extra';
                    throw 7;
                })());
            } catch (e) {
                t += e;
            }
            t === 'pextra7'
        "#,
    );
}

#[test]
fn reentrant_constructor_getters_and_conversions_use_existing_native_stack_guards() {
    for source in [
        "let r={get [Symbol.match](){return RegExp(r);}};RegExp(r)",
        "let r={toString(){return RegExp(r);}};RegExp(r)",
        r#"
            let Target = (function() {}).bind(null);
            Object.defineProperty(Target, 'prototype', {
                get() {
                    return Reflect.construct(RegExp, [], Target);
                }
            });
            Reflect.construct(RegExp, [], Target)
        "#,
    ] {
        let mut realm = Realm::default();
        assert!(
            matches!(realm.eval(source), Err(Error::Limit { .. })),
            "{source}"
        );
        assert_eq!(
            realm.eval("new RegExp('a').source"),
            Ok(Value::String(JsString::from("a")))
        );
    }
}

#[test]
fn construction_and_source_have_no_default_size_limit_and_opted_in_output_limits_abort() {
    assert_true("let r=new RegExp('a'.repeat(120000));r.source.length===120000 && r.lastIndex===0");
    let mut realm = Realm::new(Limits {
        max_string_units: Some(1),
        ..Limits::default()
    });
    realm.eval("let r=new RegExp('/'),done=false").unwrap();
    assert!(matches!(
        realm.eval("try{r.source;}finally{done=true;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("done"), Ok(Value::Boolean(false)));
    let mut realm = Realm::default();
    realm
        .eval("let big=new RegExp('a'.repeat(120000))")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("big.source.length"), Ok(Value::Number(120000.0)));
}

#[test]
fn native_matcher_depth_and_counter_limits_are_host_failures() {
    for program in [
        "new RegExp('('.repeat(512)+'a'+')'.repeat(512))",
        "new RegExp('(a?){888888888888888888888888888888}')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{{program}}}catch{{marker=1}}finally{{marker=2}}"
            )),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
    assert_true(
        r#"
            var caught = false;
            try {
                new RegExp('(a?){999999999999999999999999999999,1}')
            } catch (error) {
                caught = error instanceof SyntaxError
            }
            caught
        "#,
    );
}
