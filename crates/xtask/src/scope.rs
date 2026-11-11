//! Reconstruct the upstream Git tree and verify complete scope accounting.

mod reviews;
#[cfg(test)]
mod tests;

use crate::Result;
use crate::integrity;
use crate::invalid;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Clone)]
struct Object {
    mode: String,
    id: String,
}

#[derive(Default)]
struct Tree {
    children: BTreeMap<String, Node>,
}

enum Node {
    Tree(Tree),
    Object(Object),
}

type Entries = BTreeMap<String, Object>;
type Dispositions = BTreeMap<String, String>;

impl Tree {
    fn insert(&mut self, parts: &[&str], object: Object) -> Result<()> {
        let name = parts[0];
        if parts.len() == 1 {
            if self
                .children
                .insert(name.to_owned(), Node::Object(object))
                .is_some()
            {
                return Err(invalid(format!("upstream file/directory collision: {name}")).into());
            }
            return Ok(());
        }
        let child = self
            .children
            .entry(name.to_owned())
            .or_insert_with(|| Node::Tree(Self::default()));
        let Node::Tree(tree) = child else {
            return Err(invalid(format!("upstream file/directory collision: {name}")).into());
        };
        tree.insert(&parts[1..], object)
    }

    fn id(&self) -> Result<String> {
        let mut entries = Vec::new();
        for (name, node) in &self.children {
            let (mode, id) = match node {
                Node::Tree(tree) => ("040000", tree.id()?),
                Node::Object(object) => (object.mode.as_str(), object.id.clone()),
            };
            // Git compares directories with a trailing slash, files with NUL.
            let key = format!("{name}{}", if mode == "040000" { '/' } else { '\0' });
            let mut data = format!("{} {name}\0", mode.trim_start_matches('0')).into_bytes();
            for offset in (0..id.len()).step_by(2) {
                data.push(u8::from_str_radix(&id[offset..offset + 2], 16)?);
            }
            entries.push((key, data));
        }
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        let data: Vec<u8> = entries.into_iter().flat_map(|(_, data)| data).collect();
        Ok(integrity::object_id("tree", &data))
    }
}

fn read_tree(path: &Path) -> Result<(Entries, Tree)> {
    let mut entries = Entries::new();
    let mut tree = Tree::default();
    for row in integrity::rows(path, 3)? {
        let [mode, id, name] = row.as_slice() else {
            unreachable!()
        };
        let parts = integrity::path_parts(name)?;
        if !matches!(mode.as_str(), "040000" | "100644" | "100755" | "120000")
            || !integrity::is_hex(id, 40)
        {
            return Err(invalid(format!("invalid upstream object: {name}")).into());
        }
        if mode == "040000" && (parts.len() != 1 || matches!(name.as_str(), "test" | "harness")) {
            return Err(
                invalid(format!("test/harness trees must be fully expanded: {name}")).into(),
            );
        }
        let object = Object {
            mode: mode.clone(),
            id: id.clone(),
        };
        if entries.insert(name.clone(), object.clone()).is_some() {
            return Err(invalid(format!("duplicate upstream object: {name}")).into());
        }
        tree.insert(&parts, object)?;
    }
    for name in ["test", "harness"] {
        if !matches!(tree.children.get(name), Some(Node::Tree(_))) {
            return Err(invalid("upstream test and harness trees must be fully expanded").into());
        }
    }
    Ok((entries, tree))
}

pub(crate) fn check(root: &Path) -> Result<()> {
    let audit = root.join("audit");
    let revision = fs::read_to_string(root.join("REVISION"))?;
    let revision = revision.trim();
    let commit = fs::read(audit.join("upstream-commit.txt"))?;
    if !integrity::is_hex(revision, 40) || integrity::object_id("commit", &commit) != revision {
        return Err(invalid("upstream commit object does not match REVISION").into());
    }
    let (entries, tree) = read_tree(&audit.join("upstream-tree.tsv"))?;
    if commit.split(|byte| *byte == b'\n').next() != Some(format!("tree {}", tree.id()?).as_bytes())
    {
        return Err(invalid("upstream tree does not match the pinned commit").into());
    }
    let retained = retained(root, &entries)?;
    let originals: BTreeSet<_> = entries
        .keys()
        .filter(|path| {
            path.starts_with("test/") && path.ends_with(".js") && !path.contains("_FIXTURE")
        })
        .cloned()
        .collect();
    if !retained.is_subset(&originals) {
        return Err(invalid("runner contains a non-original upstream path").into());
    }
    let mut scoped = Dispositions::new();
    for row in integrity::rows(&audit.join("scope.tsv"), 2)? {
        let [reason, path] = row.as_slice() else {
            unreachable!()
        };
        if !valid_disposition(reason, path) {
            return Err(invalid(format!("invalid scope disposition: {reason}: {path}")).into());
        }
        if scoped.insert(path.clone(), reason.clone()).is_some() {
            return Err(invalid(format!("duplicate scope disposition: {path}")).into());
        }
    }
    let missing: BTreeSet<_> = originals.difference(&retained).cloned().collect();
    if scoped.keys().cloned().collect::<BTreeSet<_>>() != missing {
        return Err(
            invalid("scope must account for every unretained original exactly once").into(),
        );
    }
    reviews::check(&audit, revision, &entries, &scoped)
}

fn retained(root: &Path, entries: &Entries) -> Result<BTreeSet<String>> {
    let mut manifest = BTreeSet::new();
    for row in integrity::rows(&root.join("manifest.tsv"), 3)? {
        let path = &row[2];
        integrity::path_parts(path)?;
        if !manifest.insert(path.clone()) {
            return Err(invalid(format!("duplicate retained file: {path}")).into());
        }
        let object = entries
            .get(path)
            .filter(|object| object.mode != "040000")
            .ok_or_else(|| invalid(format!("retained file absent from upstream: {path}")))?;
        if integrity::object_id("blob", &fs::read(root.join("upstream").join(path))?) != object.id {
            return Err(invalid(format!("retained bytes differ from upstream: {path}")).into());
        }
    }
    let mut retained = BTreeSet::new();
    for row in integrity::rows(&root.join("runner.tsv"), 4)? {
        let path = &row[0];
        if !manifest.contains(path) || !retained.insert(path.clone()) {
            return Err(invalid(format!("duplicate or unmanifested runner path: {path}")).into());
        }
    }
    Ok(retained)
}

fn valid_disposition(reason: &str, path: &str) -> bool {
    let prefixes: &[&str] = match reason {
        "optional-annex-b" => &["test/annexB/"],
        "outside-ecma262" => &["test/intl402/"],
        "harness-self-test" => &["test/harness/"],
        "staging-review" | "native-resource-constraint" => &["test/staging/"],
        "baseline-review" => &["test/built-ins/", "test/language/"],
        "unicode-version-difference" => {
            &["test/built-ins/RegExp/", "test/language/literals/regexp/"]
        }
        "optional-legacy-helpers" => &[
            "test/built-ins/Object/prototype/",
            "test/built-ins/TypedArrayConstructors/ctors/no-species.js",
            "test/language/expressions/class/",
            "test/language/statements/class/",
            "test/staging/",
        ],
        "optional-annex-b-dependent" => &["test/built-ins/", "test/language/", "test/staging/"],
        "optional-function-introspection" => &[
            "test/built-ins/Function/",
            "test/language/arguments-object/",
            "test/staging/",
        ],
        "post-publication-assertion" => &[
            "test/built-ins/Iterator/prototype/take/",
            "test/built-ins/Iterator/prototype/drop/",
        ],
        "allocation-assumption" => &[
            "test/built-ins/ArrayBuffer/",
            "test/built-ins/SharedArrayBuffer/",
            "test/built-ins/TypedArrayConstructors/",
        ],
        "implementation-defined-date-parse" => &["test/staging/sm/Date/"],
        "permitted-regexp-source" => &["test/staging/sm/RegExp/"],
        "unicode-case-version-difference" => {
            &["test/staging/sm/String/string-upper-lower-mapping.js"]
        }
        _ => {
            let Some(feature) = reason.strip_prefix("outside-edition17:") else {
                return false;
            };
            if !matches!(
                feature,
                "Temporal"
                    | "ShadowRealm"
                    | "explicit-resource-management"
                    | "source-phase-imports"
                    | "import-defer"
                    | "await-dictionary"
                    | "joint-iteration"
                    | "iterator-chunking"
                    | "iterator-includes"
                    | "Iterator.prototype.join"
                    | "immutable-arraybuffer"
                    | "error-stack-accessor"
                    | "decorators"
                    | "import-text"
                    | "nonextensible-applies-to-private"
            ) {
                return false;
            }
            &["test/built-ins/", "test/language/", "test/staging/"]
        }
    };
    prefixes.iter().any(|prefix| path.starts_with(prefix))
}
