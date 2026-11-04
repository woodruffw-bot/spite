//! Relative Test262 source identifiers, confined to the corpus test directory.

use crate::{ModuleSource, runner::ModuleLoadError};
use spite_core::Span;
use spite_parser::ast::ModuleRequest;
use spite_runtime::{Error, ExceptionKind};
use std::{fs, io, path::Path};

// A genuinely absent source is a host-selected loading exception. Broken
// symlinks, escapes and other filesystem failures remain preparation errors.
fn absent_source(path: &Path, test_root: &Path) -> Result<(), String> {
    let mut candidate = Some(path);
    while let Some(path) = candidate {
        match fs::symlink_metadata(path) {
            Ok(_) => {
                let canonical = path.canonicalize().map_err(|error| error.to_string())?;
                return if canonical.starts_with(test_root) {
                    Ok(())
                } else {
                    Err("module source resolves outside corpus test directory".into())
                };
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                candidate = path.parent();
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("module source has no confined filesystem ancestor".into())
}

pub(super) fn load_source(
    root: &Path,
    referrer: &str,
    request: &ModuleRequest,
) -> Result<ModuleSource, ModuleLoadError> {
    let specifier = String::from_utf16(request.specifier.code_units())
        .map_err(|_| "module specifier is not a UTF-8 filesystem name".to_owned())?;
    if specifier.starts_with('/') || specifier.contains(['\\', ':']) {
        return Err(format!("unsupported filesystem module specifier: {specifier:?}").into());
    }
    let mut parts: Vec<_> = referrer.split('/').collect();
    parts.pop();
    for part in specifier.split('/') {
        match part {
            "" | "." => (),
            ".." => {
                if parts.len() <= 1 {
                    return Err("module request leaves corpus test directory"
                        .to_owned()
                        .into());
                }
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    if parts.first() != Some(&"test") {
        return Err("invalid corpus module path".to_owned().into());
    }
    let identifier = parts.join("/");
    let test_root = root
        .join("upstream/test")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let requested = root.join("upstream").join(&identifier);
    let path = match requested.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            absent_source(&requested, &test_root)?;
            return Err(ModuleLoadError::Resolution(Error::Exception {
                kind: ExceptionKind::TypeError,
                span: Span::new(0, 0),
                message: format!("module {identifier}: {error}"),
            }));
        }
        Err(error) => return Err(format!("module {identifier}: {error}").into()),
    };
    if !path.starts_with(test_root) {
        return Err("module source resolves outside corpus test directory"
            .to_owned()
            .into());
    }
    if path.is_dir() {
        return Err(ModuleLoadError::Resolution(Error::Exception {
            kind: ExceptionKind::TypeError,
            span: Span::new(0, 0),
            message: format!("module {identifier} is a directory"),
        }));
    }
    let source =
        fs::read_to_string(path).map_err(|error| format!("module {identifier}: {error}"))?;
    Ok(ModuleSource { identifier, source })
}
