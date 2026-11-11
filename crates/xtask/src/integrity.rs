//! Hashes and portable paths used by the fixture manifests.

use crate::Result;
use crate::invalid;
use sha1::Digest;
use sha1::Sha1;
use sha2::Sha256;
use std::fs;
use std::path::Path;

pub(crate) fn object_id(kind: &str, data: &[u8]) -> String {
    let mut hash = Sha1::new();
    hash.update(format!("{kind} {}\0", data.len()).as_bytes());
    hash.update(data);
    format!("{:x}", hash.finalize())
}

pub(crate) fn sha256(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

pub(crate) fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn path_parts(path: &str) -> Result<Vec<&str>> {
    let parts: Vec<_> = path.split('/').collect();
    if path.contains(['\\', ':'])
        || path.chars().any(char::is_control)
        || parts
            .iter()
            .any(|part| part.is_empty() || matches!(*part, "." | ".."))
    {
        return Err(invalid(format!("invalid upstream path: {path}")).into());
    }
    Ok(parts)
}

pub(crate) fn rows(path: &Path, fields: usize) -> Result<Vec<Vec<String>>> {
    let text = fs::read_to_string(path)?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let row: Vec<String> = line.split('\t').map(str::to_owned).collect();
        if row.len() != fields {
            return Err(invalid(format!(
                "{}:{}: expected {fields} fields",
                path.display(),
                index + 1
            ))
            .into());
        }
        rows.push(row);
    }
    Ok(rows)
}
