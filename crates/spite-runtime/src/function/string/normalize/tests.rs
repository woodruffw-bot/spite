//! Independent invariants from the complete, unchanged Unicode normalization suite.

use super::*;

fn text(field: &str) -> JsString {
    JsString::from_code_units(
        field
            .split_whitespace()
            .flat_map(|point| {
                let point = char::from_u32(u32::from_str_radix(point, 16).unwrap()).unwrap();
                let mut units = [0; 2];
                let length = point.encode_utf16(&mut units).len();
                units.into_iter().take(length)
            })
            .collect(),
    )
}

#[test]
fn complete_unicode_18_normalization_invariants() {
    let mut realm = Realm::default();
    let mut part1 = false;
    let mut listed = vec![false; 0x110000];
    let mut count = 0;
    for (line_number, line) in
        include_str!("../../../../tests/fixtures/NormalizationTest-18.0.0.txt")
            .lines()
            .enumerate()
    {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('@') {
            part1 = line == "@Part1";
            continue;
        }
        let fields: Vec<_> = line.split(';').take(5).collect();
        assert_eq!(fields.len(), 5);
        let columns: Vec<_> = fields.iter().map(|field| text(field)).collect();
        if part1 {
            let points: Vec<_> = fields[0].split_whitespace().collect();
            assert_eq!(points.len(), 1);
            listed[u32::from_str_radix(points[0], 16).unwrap() as usize] = true;
        }
        for (input, column) in columns.iter().enumerate() {
            for (form, expected) in [
                (Form::Nfc, if input < 3 { 1 } else { 3 }),
                (Form::Nfd, if input < 3 { 2 } else { 4 }),
                (Form::Nfkc, 3),
                (Form::Nfkd, 4),
            ] {
                assert_eq!(
                    realm
                        .normalize_string(column, form, Span::new(0, 0))
                        .unwrap(),
                    columns[expected],
                    "line {} / column {} / {form:?}",
                    line_number + 1,
                    input + 1
                );
            }
        }
        count += 1;
    }
    assert_eq!(count, 20171, "the complete official suite must be present");
    assert_eq!(listed.iter().filter(|&&point| point).count(), 17154);
    // UAX #15's second invariant covers every assigned point not in Part 1.
    // Checking all remaining scalars additionally protects unassigned identity.
    for (point, &listed) in listed.iter().enumerate() {
        if listed {
            continue;
        }
        let Some(point) = char::from_u32(point as u32) else {
            continue;
        };
        let original = JsString::from_code_units(point.encode_utf16(&mut [0; 2]).to_vec());
        for form in [Form::Nfc, Form::Nfd, Form::Nfkc, Form::Nfkd] {
            assert_eq!(
                realm
                    .normalize_string(&original, form, Span::new(0, 0))
                    .unwrap(),
                original,
                "U+{:04X} / {form:?}",
                point as u32
            );
        }
    }
}

#[test]
fn intermediate_decomposition_does_not_consume_the_final_string_quota() {
    let mut realm = Realm::new(crate::Limits {
        max_string_units: Some(1),
        ..crate::Limits::default()
    });
    assert_eq!(
        realm.normalize_string(&JsString::from("é"), Form::Nfc, Span::new(0, 0)),
        Ok(JsString::from("é"))
    );
    assert!(matches!(
        realm.normalize_string(&JsString::from("é"), Form::Nfd, Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
}
