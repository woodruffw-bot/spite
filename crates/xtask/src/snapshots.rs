//! Keep small snapshots beside their assertions and reject orphaned files.

use crate::{Result, invalid};
use std::fs;
use std::path::Path;

pub(crate) fn check(root: &Path) -> Result<()> {
    let mut directories = vec![root.join("crates")];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                directories.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "snap")
            {
                let contents = fs::read_to_string(&path)?;
                let source = validate(&contents)
                    .map_err(|error| invalid(format!("{}: {error}", path.display())))?;
                if !root.join(source).is_file() {
                    return Err(invalid(format!(
                        "{}: snapshot source does not exist: {source}",
                        path.display()
                    ))
                    .into());
                }
            }
        }
    }
    println!("Verified sidecar snapshot sizes and source paths.");
    Ok(())
}

fn validate(contents: &str) -> Result<&str> {
    let mut lines = contents.lines();
    if lines.next() != Some("---") {
        return Err(invalid("snapshot has no metadata header").into());
    }
    let mut source = None;
    let mut closed = false;
    for line in lines.by_ref() {
        if line == "---" {
            closed = true;
            break;
        }
        if let Some(path) = line.strip_prefix("source: ") {
            source = Some(path);
        }
    }
    if !closed {
        return Err(invalid("snapshot metadata header is not closed").into());
    }
    let source = source.ok_or_else(|| invalid("snapshot has no source path"))?;
    if lines.count() <= 100 {
        return Err(invalid("snapshots of at most 100 lines must be inline").into());
    }
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::validate;

    #[test]
    fn sidecars_start_above_the_inline_threshold() {
        let header = "---\nsource: crates/example/tests/cases.rs\n---\n";
        assert!(validate(&format!("{header}{}", "line\n".repeat(100))).is_err());
        assert_eq!(
            validate(&format!("{header}{}", "line\n".repeat(101))).unwrap(),
            "crates/example/tests/cases.rs"
        );
    }

    #[test]
    fn windows_line_endings_use_the_same_threshold() {
        let contents = format!(
            "---\r\nsource: crates/example/tests/cases.rs\r\n---\r\n{}",
            "line\r\n".repeat(101)
        );
        assert_eq!(
            validate(&contents).unwrap(),
            "crates/example/tests/cases.rs"
        );
    }

    #[test]
    fn missing_or_incomplete_metadata_is_rejected() {
        assert!(validate("body without a header").is_err());
        assert!(validate("---\nsource: tests/cases.rs\n").is_err());
        assert!(validate(&format!("---\n---\n{}", "line\n".repeat(101))).is_err());
    }
}
