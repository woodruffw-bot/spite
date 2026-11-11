//! Source reviews and witnesses required for scoped upstream tests.

use super::Dispositions;
use super::Entries;
use crate::Result;
use crate::integrity;
use crate::invalid;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(super) fn check(
    audit: &Path,
    revision: &str,
    entries: &Entries,
    scoped: &Dispositions,
) -> Result<()> {
    staging(audit, entries, scoped)?;
    resources(audit, revision, entries, scoped)?;
    unicode(audit, scoped)
}

fn staging(audit: &Path, entries: &Entries, scoped: &Dispositions) -> Result<()> {
    let expected: BTreeSet<_> = scoped
        .iter()
        .filter(|(path, reason)| {
            path.starts_with("test/staging/") && reason.as_str() != "staging-review"
        })
        .map(|(path, _)| path.clone())
        .collect();
    let mut actual = BTreeSet::new();
    let file = audit.join("staging-reviewed.tsv");
    if file.exists() {
        for row in integrity::rows(&file, 4)? {
            let [reason, path, id, explanation] = row.as_slice() else {
                unreachable!()
            };
            let pinned = entries
                .get(path)
                .is_some_and(|object| object.mode == "100644" && object.id == *id);
            if !actual.insert(path.clone())
                || scoped.get(path) != Some(reason)
                || !expected.contains(path)
                || !pinned
                || explanation.trim().is_empty()
            {
                return Err(invalid(format!("invalid or unpinned staging review: {path}")).into());
            }
        }
    }
    if actual != expected {
        return Err(
            invalid("closed staging scope entries require matching pinned source reviews").into(),
        );
    }
    Ok(())
}

fn resources(audit: &Path, revision: &str, entries: &Entries, scoped: &Dispositions) -> Result<()> {
    let expected = paths_for(scoped, "native-resource-constraint");
    let mut actual = BTreeSet::new();
    let file = audit.join("resource-stress.json");
    if file.exists() {
        let receipt: Value = serde_json::from_slice(&fs::read(file)?)?;
        if receipt["revision"].as_str() != Some(revision) {
            return Err(invalid("resource receipt must name the pinned upstream revision").into());
        }
        if receipt
            .get("passing_credit")
            .is_some_and(|value| value.as_u64() != Some(0))
        {
            return Err(invalid("resource receipts cannot claim passing credit").into());
        }
        let records = receipt["results"]
            .as_array()
            .ok_or_else(|| invalid("resource receipt needs results"))?;
        for record in records {
            let path = record["path"]
                .as_str()
                .ok_or_else(|| invalid("resource receipt needs a path"))?;
            let pinned = entries.get(path).is_some_and(|object| {
                object.mode == "100644" && Some(object.id.as_str()) == record["git_blob"].as_str()
            });
            let outcomes = record["outcomes"].as_array();
            if !expected.contains(path)
                || !actual.insert(path.to_owned())
                || !pinned
                || record["runner"].as_str() != Some("defaults")
                || !outcomes.is_some_and(|outcomes| valid_outcomes(outcomes))
            {
                return Err(invalid(format!(
                    "resource constraints require pinned nonpassing Limit outcomes: {path}"
                ))
                .into());
            }
        }
    }
    if actual != expected {
        return Err(invalid("resource constraints require complete nonpassing receipts").into());
    }
    Ok(())
}

fn valid_outcomes(outcomes: &[Value]) -> bool {
    let mut modes = BTreeSet::new();
    !outcomes.is_empty()
        && outcomes.iter().all(|outcome| {
            let Some(mode) = outcome["mode"].as_str().filter(|mode| !mode.is_empty()) else {
                return false;
            };
            modes.insert(mode)
                && outcome["category"].as_str() == Some("limit")
                && outcome["stage"].as_str() == Some("Runtime")
                && outcome["message"]
                    .as_str()
                    .is_some_and(|message| !message.is_empty())
        })
}

fn unicode(audit: &Path, scoped: &Dispositions) -> Result<()> {
    let mut expected = paths_for(scoped, "unicode-version-difference");
    // These literal loops hard-code join-control IdentityEscapes, unlike the
    // property-table tests which need a concrete Unicode-version witness.
    for path in [
        "test/language/literals/regexp/S7.8.5_A1.4_T2.js",
        "test/language/literals/regexp/S7.8.5_A2.4_T2.js",
    ] {
        expected.remove(path);
    }
    let mut actual = BTreeSet::new();
    let file = audit.join("unicode-witnesses.tsv");
    if file.exists() {
        for row in integrity::rows(&file, 5)? {
            let [path, property, point, old, current] = row.as_slice() else {
                unreachable!()
            };
            let valid_point = u32::from_str_radix(point, 16).is_ok_and(|point| point <= 0x10ffff);
            let contradictory = matches!(
                (old.as_str(), current.as_str()),
                ("true", "false") | ("false", "true")
            );
            if !actual.insert(path.clone()) || property.is_empty() || !valid_point || !contradictory
            {
                return Err(invalid(format!(
                    "invalid or non-contradictory Unicode witness: {path}"
                ))
                .into());
            }
        }
    }
    if actual != expected {
        return Err(invalid(
            "Unicode witnesses must cover every version-dependent property original exactly",
        )
        .into());
    }
    Ok(())
}

fn paths_for(scoped: &Dispositions, reason: &str) -> BTreeSet<String> {
    scoped
        .iter()
        .filter(|(_, value)| value.as_str() == reason)
        .map(|(path, _)| path.clone())
        .collect()
}
