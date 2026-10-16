//! Exact Inherited Script aliases, complements, consumers, and Unicode 18 changes.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn unicode_inherited_script_aliases_complements_and_fold_modes() {
    check(
        r#"let patterns=["^\\p{Script=Zinh}$", "^\\P{Script=Zinh}$", "^[\\p{Script=Zinh}]$", "^[\\P{Script=Zinh}]$", "^[^\\p{Script=Zinh}]$", "^[^\\P{Script=Zinh}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Zinh}$", "^\\P{Script=Zinh}$", "^[\\p{Script=Zinh}]$", "^[\\P{Script=Zinh}]$", "^[^\\p{Script=Zinh}]$", "^[^\\P{Script=Zinh}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110011111000111100101|0000001111110111000011010|1111110011111000111100101|1111110000001000111100101|0000001100000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Zinh}$", "^\\P{Script=Zinh}$", "^[\\p{Script=Zinh}]$", "^[\\P{Script=Zinh}]$", "^[^\\p{Script=Zinh}]$", "^[^\\P{Script=Zinh}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Zinh}$", "^\\P{Script=Zinh}$", "^[\\p{Script=Zinh}]$", "^[\\P{Script=Zinh}]$", "^[^\\p{Script=Zinh}]$", "^[^\\P{Script=Zinh}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110000001000111100101|0000001111110111000011010|1111110000001000111100101|1111110000001000111100101|0000001111110111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Inherited}$", "^\\P{Script=Inherited}$", "^[\\p{Script=Inherited}]$", "^[\\P{Script=Inherited}]$", "^[^\\p{Script=Inherited}]$", "^[^\\P{Script=Inherited}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Inherited}$", "^\\P{Script=Inherited}$", "^[\\p{Script=Inherited}]$", "^[\\P{Script=Inherited}]$", "^[^\\p{Script=Inherited}]$", "^[^\\P{Script=Inherited}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110011111000111100101|0000001111110111000011010|1111110011111000111100101|1111110000001000111100101|0000001100000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Inherited}$", "^\\P{Script=Inherited}$", "^[\\p{Script=Inherited}]$", "^[\\P{Script=Inherited}]$", "^[^\\p{Script=Inherited}]$", "^[^\\P{Script=Inherited}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Inherited}$", "^\\P{Script=Inherited}$", "^[\\p{Script=Inherited}]$", "^[\\P{Script=Inherited}]$", "^[^\\p{Script=Inherited}]$", "^[^\\P{Script=Inherited}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110000001000111100101|0000001111110111000011010|1111110000001000111100101|1111110000001000111100101|0000001111110111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Qaai}$", "^\\P{Script=Qaai}$", "^[\\p{Script=Qaai}]$", "^[\\P{Script=Qaai}]$", "^[^\\p{Script=Qaai}]$", "^[^\\P{Script=Qaai}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Qaai}$", "^\\P{Script=Qaai}$", "^[\\p{Script=Qaai}]$", "^[\\P{Script=Qaai}]$", "^[^\\p{Script=Qaai}]$", "^[^\\P{Script=Qaai}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110011111000111100101|0000001111110111000011010|1111110011111000111100101|1111110000001000111100101|0000001100000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Qaai}$", "^\\P{Script=Qaai}$", "^[\\p{Script=Qaai}]$", "^[\\P{Script=Qaai}]$", "^[^\\p{Script=Qaai}]$", "^[^\\P{Script=Qaai}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{Script=Qaai}$", "^\\P{Script=Qaai}$", "^[\\p{Script=Qaai}]$", "^[\\P{Script=Qaai}]$", "^[^\\p{Script=Qaai}]$", "^[^\\P{Script=Qaai}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110000001000111100101|0000001111110111000011010|1111110000001000111100101|1111110000001000111100101|0000001111110111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Zinh}$", "^\\P{sc=Zinh}$", "^[\\p{sc=Zinh}]$", "^[\\P{sc=Zinh}]$", "^[^\\p{sc=Zinh}]$", "^[^\\P{sc=Zinh}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Zinh}$", "^\\P{sc=Zinh}$", "^[\\p{sc=Zinh}]$", "^[\\P{sc=Zinh}]$", "^[^\\p{sc=Zinh}]$", "^[^\\P{sc=Zinh}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110011111000111100101|0000001111110111000011010|1111110011111000111100101|1111110000001000111100101|0000001100000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Zinh}$", "^\\P{sc=Zinh}$", "^[\\p{sc=Zinh}]$", "^[\\P{sc=Zinh}]$", "^[^\\p{sc=Zinh}]$", "^[^\\P{sc=Zinh}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Zinh}$", "^\\P{sc=Zinh}$", "^[\\p{sc=Zinh}]$", "^[\\P{sc=Zinh}]$", "^[^\\p{sc=Zinh}]$", "^[^\\P{sc=Zinh}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110000001000111100101|0000001111110111000011010|1111110000001000111100101|1111110000001000111100101|0000001111110111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Inherited}$", "^\\P{sc=Inherited}$", "^[\\p{sc=Inherited}]$", "^[\\P{sc=Inherited}]$", "^[^\\p{sc=Inherited}]$", "^[^\\P{sc=Inherited}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Inherited}$", "^\\P{sc=Inherited}$", "^[\\p{sc=Inherited}]$", "^[\\P{sc=Inherited}]$", "^[^\\p{sc=Inherited}]$", "^[^\\P{sc=Inherited}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110011111000111100101|0000001111110111000011010|1111110011111000111100101|1111110000001000111100101|0000001100000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Inherited}$", "^\\P{sc=Inherited}$", "^[\\p{sc=Inherited}]$", "^[\\P{sc=Inherited}]$", "^[^\\p{sc=Inherited}]$", "^[^\\P{sc=Inherited}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Inherited}$", "^\\P{sc=Inherited}$", "^[\\p{sc=Inherited}]$", "^[\\P{sc=Inherited}]$", "^[^\\p{sc=Inherited}]$", "^[^\\P{sc=Inherited}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110000001000111100101|0000001111110111000011010|1111110000001000111100101|1111110000001000111100101|0000001111110111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Qaai}$", "^\\P{sc=Qaai}$", "^[\\p{sc=Qaai}]$", "^[\\P{sc=Qaai}]$", "^[^\\p{sc=Qaai}]$", "^[^\\P{sc=Qaai}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Qaai}$", "^\\P{sc=Qaai}$", "^[\\p{sc=Qaai}]$", "^[\\P{sc=Qaai}]$", "^[^\\p{sc=Qaai}]$", "^[^\\P{sc=Qaai}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110011111000111100101|0000001111110111000011010|1111110011111000111100101|1111110000001000111100101|0000001100000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Qaai}$", "^\\P{sc=Qaai}$", "^[\\p{sc=Qaai}]$", "^[\\P{sc=Qaai}]$", "^[^\\p{sc=Qaai}]$", "^[^\\P{sc=Qaai}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001110000111000011010|1111110001111000111100101|0000001110000111000011010|1111110001111000111100101|1111110001111000111100101|0000001110000111000011010""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Qaai}$", "^\\P{sc=Qaai}$", "^[\\p{sc=Qaai}]$", "^[\\P{sc=Qaai}]$", "^[^\\p{sc=Qaai}]$", "^[^\\P{sc=Qaai}]$"],points=[0, 10, 32, 65, 97, 181, 768, 769, 837, 921, 953, 8126, 1488, 8205, 8204, 65039, 55296, 56320, 57344, 64976, 119143, 119163, 128512, 917760, 1114111],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="0000001111110111000011010|1111110000001000111100101|0000001111110111000011010|1111110000001000111100101|1111110000001000111100101|0000001111110111000011010""#,
    );
}

#[test]
fn unicode_inherited_script_unicode18_additions() {
    check(
        r#"let patterns=["^\\p{sc=Zinh}$", "^\\P{sc=Zinh}$", "^[\\p{sc=Zinh}]$", "^[\\P{sc=Zinh}]$", "^[^\\p{sc=Zinh}]$", "^[^\\P{sc=Zinh}]$"],points=[6878, 6879, 6892, 6893, 6894, 6895, 6896, 119079, 119080, 119387, 119388],flags="u";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11111111111|00000000000|11111111111|00000000000|00000000000|11111111111""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Zinh}$", "^\\P{sc=Zinh}$", "^[\\p{sc=Zinh}]$", "^[\\P{sc=Zinh}]$", "^[^\\p{sc=Zinh}]$", "^[^\\P{sc=Zinh}]$"],points=[6878, 6879, 6892, 6893, 6894, 6895, 6896, 119079, 119080, 119387, 119388],flags="ui";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11111111111|00000000000|11111111111|00000000000|00000000000|11111111111""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Zinh}$", "^\\P{sc=Zinh}$", "^[\\p{sc=Zinh}]$", "^[\\P{sc=Zinh}]$", "^[^\\p{sc=Zinh}]$", "^[^\\P{sc=Zinh}]$"],points=[6878, 6879, 6892, 6893, 6894, 6895, 6896, 119079, 119080, 119387, 119388],flags="v";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11111111111|00000000000|11111111111|00000000000|00000000000|11111111111""#,
    );
    check(
        r#"let patterns=["^\\p{sc=Zinh}$", "^\\P{sc=Zinh}$", "^[\\p{sc=Zinh}]$", "^[\\P{sc=Zinh}]$", "^[^\\p{sc=Zinh}]$", "^[^\\P{sc=Zinh}]$"],points=[6878, 6879, 6892, 6893, 6894, 6895, 6896, 119079, 119080, 119387, 119388],flags="vi";patterns.map(pattern=>{let r=new RegExp(pattern,flags);return points.map(cp=>r.test(String.fromCodePoint(cp))?'1':'0').join('')}).join('|')==="11111111111|00000000000|11111111111|00000000000|00000000000|11111111111""#,
    );
}

#[test]
fn unicode_inherited_script_results_snapshot() {
    let mut rows = String::new();
    for name in [
        "Script=Inherited",
        "Script=Zinh",
        "Script=Qaai",
        "sc=Inherited",
        "sc=Zinh",
        "sc=Qaai",
    ] {
        for body in [
            format!(r"\p{{{name}}}"),
            format!(r"\P{{{name}}}"),
            format!(r"(?<x>[\p{{{name}}}K])"),
        ] {
            let pattern = JsString::from(body.as_str());
            for flags in ["dug", "diug", "divg", "divy"] {
                for input in ["ͅΙιὶ𝅧\n", "X\r\ǹ\u{2028}א", ""] {
                    let input = JsString::from(input);
                    for start in [0, 1, 3] {
                        let script = format!(
                            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify({{m,index:m?.index,groups:m?.groups,indices:m?.indices,indexGroups:m?.indices?.groups,lastIndex:r.lastIndex,source:r.source}})"
                        );
                        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                            panic!("{script}")
                        };
                        writeln!(
                            rows,
                            "{body:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}
#[test]
fn unicode_inherited_script_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{sc=Zinh}}K]))$/dim{mode}g,m=r.exec('A\r\ǹ\u2028');m.index===3&&m[1]==='̀'&&m[2]==='̀'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\p{{Script=Inherited}})/d{mode}y;r.lastIndex=1;let m=r.exec('𝅧A');m.index===0&&m[1]==='𝅧'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\P{{sc=Inherited}}/d{mode}g;r.lastIndex=1;let m=r.exec('𝅧\uD800');m.index===2&&m[0]==='\uD800'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\P{{Script=Zinh}}/{mode}y;r.lastIndex=1;r.exec('𝅧\uD800')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_inherited_script_consumers_preserve_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'̀😀Á'.matchAll(/(?<x>\P{{sc=Inherited}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='A'&&a[1].index===3"
        ));
        check(&format!(
            r"'̀́A'.replace(/(?<x>\p{{sc=Zinh}})/{mode}g,'<$1:$<x>>')==='<̀:̀><́:́>A'"
        ));
        check(&format!(
            r"'À́A'.split(/(\p{{Script=Inherited}})/{mode}).join('|')==='A|̀||́|A'&&'A😀̀'.search(/\p{{sc=Zinh}}/{mode})===3&&'̀́A'.match(/\p{{Script=Zinh}}/{mode}g).join('')==='̀́'"
        ));
        check(&format!(
            r"let t='',x='̀́À'.replace(/(\p{{sc=Inherited}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__A_'&&t==='013'"
        ));
    }
}

#[test]
fn unicode_inherited_script_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{sc=Zinh})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{sc=Zinh})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('̀').groups.x==='̀'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\p{Script=Inherited}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '𝅧'}});t==='si'&&m[0]==='𝅧'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{sc=Inherited}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('̀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Script=Zinh}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('𝅧\uD800')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_inherited_script_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{sc=Zinh}\\\\p{Cs}\\\\p{ASCII}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='A'.repeat(100000)+'\\ǹ'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='̀'&&m[3000]==='̀'&&m.indices[3000][0]===100001&&m.indices[3000][1]===100002&&c.lastIndex===100002"), Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_inherited_script_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Script=Inherited}/ug,marker=0,text='A'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    check(r"/\p{Script_Extensions=Inherited}/u.exec('A')===null");
    check(r"/[\p{sc=Zinh}&&A]/v.exec('A')===null");
    check(r"/[\p{sc=Zinh}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x300),m=/[\p{sc=Zinh}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [
        r"/\p{sc=Zinh}+b/u.exec('AA')",
        r"/\p{sc=Zinh}A/v.exec('AA')",
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

#[test]
fn unicode_inherited_script_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{sc=Zinh}})+\B/d{mode}g,m=r.exec('µ𝅧𝅧𝅧µ');m.index===1&&m[0]==='𝅧𝅧𝅧'&&m.groups.x==='𝅧'&&m.indices.groups.x[0]===5&&m.indices.groups.x[1]===7&&r.lastIndex===7"
        ));
    }
}

#[test]
fn unicode_inherited_script_former_direct_eval_context_executes_normally() {
    check(
        r#"let marker=0,result;try{result=eval("marker=1; /\\p{Script=Inherited}/u.test('̀');")}catch{marker=2}finally{marker=3}marker===3&&result===true"#,
    );
}
