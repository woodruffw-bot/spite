//! Exact published-source verification.

use crate::Result;
use crate::integrity;
use crate::invalid;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

pub(crate) fn check(root: &Path, source: Option<&Path>) -> Result<()> {
    let identity: Value = serde_json::from_slice(&fs::read(
        root.join("tests/test262/audit/specification.json"),
    )?)?;
    let revision = identity["revision"]
        .as_str()
        .ok_or_else(|| invalid("missing specification revision"))?;
    let path = identity["path"]
        .as_str()
        .ok_or_else(|| invalid("missing specification path"))?;
    let url = format!("https://raw.githubusercontent.com/tc39/ecma262/{revision}/{path}");
    if !integrity::is_hex(revision, 40) || identity["source_url"].as_str() != Some(url.as_str()) {
        return Err(invalid("specification source must name its complete pinned revision").into());
    }
    let data = if let Some(source) = source {
        fs::read(source)?
    } else {
        let output = Command::new("curl")
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                "--max-time",
                "40",
                &url,
            ])
            .output()?;
        if !output.status.success() {
            return Err(invalid(format!(
                "specification download failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
            .into());
        }
        output.stdout
    };
    if identity["size"].as_u64() != Some(data.len() as u64)
        || identity["sha256"].as_str() != Some(integrity::sha256(&data).as_str())
        || identity["git_blob"].as_str() != Some(integrity::object_id("blob", &data).as_str())
    {
        return Err(
            invalid("published specification bytes do not match the pinned identity").into(),
        );
    }
    println!("Verified exact published specification source.");
    Ok(())
}
