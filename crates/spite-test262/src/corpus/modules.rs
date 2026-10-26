//! Relative Test262 source identifiers, confined to the corpus test directory.

use crate::ModuleSource;
use spite_parser::ast::ModuleRequest;
use std::{fs, path::Path};

pub(super) fn load_source(
    root: &Path,
    referrer: &str,
    request: &ModuleRequest,
) -> Result<ModuleSource, String> {
    let specifier = String::from_utf16(request.specifier.code_units())
        .map_err(|_| "module specifier is not a UTF-8 filesystem name".to_owned())?;
    if specifier.starts_with('/') || specifier.contains(['\\', ':']) {
        return Err(format!(
            "unsupported filesystem module specifier: {specifier:?}"
        ));
    }
    let mut parts: Vec<_> = referrer.split('/').collect();
    parts.pop();
    for part in specifier.split('/') {
        match part {
            "" | "." => (),
            ".." => {
                if parts.len() <= 1 {
                    return Err("module request leaves corpus test directory".into());
                }
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    if parts.first() != Some(&"test") || parts.len() < 2 {
        return Err("invalid corpus module path".into());
    }
    let identifier = parts.join("/");
    let test_root = root
        .join("upstream/test")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let path = root
        .join("upstream")
        .join(&identifier)
        .canonicalize()
        .map_err(|error| format!("module {identifier}: {error}"))?;
    if !path.starts_with(test_root) {
        return Err("module source resolves outside corpus test directory".into());
    }
    let source =
        fs::read_to_string(path).map_err(|error| format!("module {identifier}: {error}"))?;
    Ok(ModuleSource { identifier, source })
}
