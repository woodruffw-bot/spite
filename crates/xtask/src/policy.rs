//! Workspace dependency and lint policy.

use crate::Result;
use crate::invalid;
use serde_json::Value;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

pub(crate) fn check(root: &Path) -> Result<()> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .current_dir(root)
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .output()?;
    if !output.status.success() {
        return Err(invalid(String::from_utf8_lossy(&output.stderr)).into());
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)?;
    let members: BTreeSet<&str> = metadata["workspace_members"]
        .as_array()
        .ok_or_else(|| invalid("Cargo metadata has no workspace members"))?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| invalid("Cargo metadata has no packages"))?;
    let packages: Vec<_> = packages
        .iter()
        .filter(|package| {
            package["id"]
                .as_str()
                .is_some_and(|id| members.contains(id))
        })
        .collect();
    let mut paths = BTreeSet::new();
    for package in &packages {
        let manifest = text(package, "manifest_path")?;
        let parent = Path::new(manifest)
            .parent()
            .ok_or_else(|| invalid("manifest has no parent"))?;
        paths.insert(parent.canonicalize()?);
    }
    let workspace: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)?;
    if workspace["workspace"]["lints"]["rust"]["unsafe_code"].as_str() != Some("forbid") {
        return Err(invalid("workspace must forbid unsafe_code").into());
    }
    for package in packages {
        let name = text(package, "name")?;
        let manifest: toml::Value =
            toml::from_str(&fs::read_to_string(text(package, "manifest_path")?)?)?;
        if manifest
            .get("lints")
            .and_then(|lints| lints.get("workspace"))
            .and_then(toml::Value::as_bool)
            != Some(true)
        {
            return Err(invalid(format!("{name}: workspace lints must be inherited")).into());
        }
        let dependencies = package["dependencies"]
            .as_array()
            .ok_or_else(|| invalid(format!("{name}: missing dependency metadata")))?;
        for dependency in dependencies {
            let dependency_name = text(dependency, "name")?;
            if let Some(path) = dependency["path"].as_str()
                && paths.contains(&Path::new(path).canonicalize()?)
            {
                if dependency_name == "xtask" {
                    return Err(
                        invalid(format!("{name}: engine crates cannot depend on xtask")).into(),
                    );
                }
                continue;
            }
            let kind = dependency["kind"].as_str();
            let registry = dependency["source"].as_str()
                == Some("registry+https://github.com/rust-lang/crates.io-index");
            if !registry || !allowed(name, dependency_name, kind) {
                return Err(invalid(format!(
                    "{name}: unapproved {kind:?} dependency {dependency_name}"
                ))
                .into());
            }
        }
    }
    println!("Verified workspace dependency and lint policy.");
    Ok(())
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| invalid(format!("missing Cargo metadata string: {key}")).into())
}

fn allowed(package: &str, dependency: &str, kind: Option<&str>) -> bool {
    match kind {
        None if package == "xtask" => matches!(dependency, "serde_json" | "toml" | "sha1" | "sha2"),
        None => matches!(dependency, "regex" | "regress" | "jiff" | "jiff-tzdb"),
        Some("dev") => matches!(dependency, "insta" | "test-case"),
        Some(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::allowed;

    #[test]
    fn tooling_dependencies_do_not_expand_the_engine_allowlist() {
        assert!(allowed("xtask", "sha2", None));
        assert!(!allowed("spite-runtime", "sha2", None));
        assert!(allowed("spite-core", "insta", Some("dev")));
        assert!(allowed("spite-runtime", "test-case", Some("dev")));
        assert!(!allowed("spite-runtime", "test-case", None));
        assert!(!allowed("xtask", "sha2", Some("build")));
        assert!(!allowed("spite-runtime", "unapproved", None));
    }
}
