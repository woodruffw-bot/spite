//! Integrity and execution-review checks for retained Test262 files.

use crate::Result;
use crate::integrity;
use crate::invalid;
use crate::scope;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const EXECUTED: &[&str] = &[
    "raw-pass",
    "script-pass",
    "script-runtime-error",
    "script-resolution-error",
    "identifier-tokens",
    "parser-pass",
];
const NEGATIVE: &[&str] = &["raw-syntax-error", "parse-syntax-error", "identifier-error"];

pub(crate) fn check(root: &Path) -> Result<()> {
    let upstream = root.join("upstream");
    let mut modes = BTreeMap::new();
    for row in integrity::rows(&root.join("manifest.tsv"), 3)? {
        let [mode, checksum, path] = row.as_slice() else {
            unreachable!()
        };
        let parts = integrity::path_parts(path)?;
        let known = EXECUTED.contains(&mode.as_str())
            || NEGATIVE.contains(&mode.as_str())
            || matches!(mode.as_str(), "module-fixture" | "harness");
        let extension = Path::new(path).extension().and_then(|value| value.to_str());
        if !known
            || !(extension == Some("js") || (mode == "module-fixture" && extension == Some("json")))
        {
            return Err(
                invalid(format!("invalid fixture mode or extension: {mode}: {path}")).into(),
            );
        }
        if (mode == "harness") != (parts[0] == "harness")
            || (mode != "harness" && parts[0] != "test")
        {
            return Err(invalid(format!("fixture mode/path mismatch: {path}")).into());
        }
        if modes.insert(path.clone(), mode.clone()).is_some() {
            return Err(invalid(format!("duplicate fixture: {path}")).into());
        }
        if integrity::sha256(&fs::read(upstream.join(path))?) != *checksum {
            return Err(invalid(format!("fixture bytes changed: {path}")).into());
        }
    }
    let expected: BTreeSet<_> = modes.keys().cloned().collect();
    let mut actual = BTreeSet::new();
    inventory(&upstream, &upstream, &mut actual)?;
    if expected.is_empty() || expected != actual {
        return Err(invalid("fixture inventory does not match the manifest").into());
    }
    let mut reviewed = BTreeSet::new();
    for row in integrity::rows(&root.join("runner.tsv"), 4)? {
        let [path, start, end, message] = row.as_slice() else {
            unreachable!()
        };
        let Some(mode) = modes.get(path) else {
            return Err(invalid(format!("unknown runner path: {path}")).into());
        };
        if !reviewed.insert(path.clone()) {
            return Err(invalid(format!("duplicate runner path: {path}")).into());
        }
        if EXECUTED.contains(&mode.as_str()) {
            if [start, end, message]
                .iter()
                .any(|field| field.as_str() != "-")
            {
                return Err(
                    invalid(format!("executed runner entry has a parse review: {path}")).into(),
                );
            }
        } else if NEGATIVE.contains(&mode.as_str()) {
            let source = fs::read_to_string(upstream.join(path))?;
            if !valid_diagnostic(&source, start, end, message) {
                return Err(invalid(format!("invalid reviewed diagnostic: {path}")).into());
            }
        } else {
            return Err(invalid(format!("non-executable runner fixture: {path}")).into());
        }
    }
    let executable = modes
        .iter()
        .filter(|(_, mode)| EXECUTED.contains(&mode.as_str()) || NEGATIVE.contains(&mode.as_str()))
        .map(|(path, _)| path.clone())
        .collect();
    if reviewed != executable {
        return Err(invalid("runner inventory does not match selected fixture modes").into());
    }
    scope::check(root)?;
    println!("Verified Test262 fixture bytes, execution reviews and complete pinned scope.");
    Ok(())
}

fn valid_diagnostic(source: &str, start: &str, end: &str, message: &str) -> bool {
    let (Ok(start), Ok(end)) = (start.parse::<usize>(), end.parse::<usize>()) else {
        return false;
    };
    start <= end
        && end <= source.len()
        && (start < end || end == source.len())
        && source.is_char_boundary(start)
        && source.is_char_boundary(end)
        && !matches!(message, "" | "-")
}

fn inventory(directory: &Path, root: &Path, paths: &mut BTreeSet<String>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            inventory(&path, root, paths)?;
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)?
                .components()
                .map(|part| {
                    part.as_os_str()
                        .to_str()
                        .ok_or_else(|| invalid("non-UTF-8 fixture path"))
                })
                .collect::<std::result::Result<Vec<_>, _>>()?
                .join("/");
            paths.insert(relative);
        } else {
            return Err(invalid(format!(
                "fixture must be a regular file: {}",
                path.display()
            ))
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::valid_diagnostic;

    #[test]
    fn diagnostic_spans_must_follow_utf8_boundaries_and_allow_only_eof_empty_spans() {
        assert!(valid_diagnostic("π!", "0", "2", "unexpected token"));
        assert!(valid_diagnostic("π!", "3", "3", "expected an expression"));
        for (start, end, message) in [
            ("1", "2", "error"),
            ("0", "0", "error"),
            ("3", "4", "error"),
            ("2", "1", "error"),
            ("0", "2", "-"),
        ] {
            assert!(!valid_diagnostic("π!", start, end, message));
        }
    }
}
