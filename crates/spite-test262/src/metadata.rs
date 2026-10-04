use std::{borrow::Cow, collections::BTreeSet, fmt};

/// A stage at which a negative Test262 test expects an exception.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// Parsing or static early-error validation.
    Parse,
    /// Module resolution and linking.
    Resolution,
    /// Script or module evaluation.
    Runtime,
}

/// The required phase and error-constructor name of a negative test.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Negative {
    /// Stage at which the exception must originate.
    pub phase: Phase,
    /// Name of the exception's constructor.
    pub error_type: String,
}

/// An execution variant requested by Test262 metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    /// Script source without an injected directive (including raw tests).
    Script,
    /// Script source with the required initial use-strict directive.
    StrictScript,
    /// Module source, whose grammar supplies strictness.
    Module,
}

impl Mode {
    /// Applies only the strict-mode transformation specified by INTERPRETING.md.
    /// Raw and module sources remain unchanged because their planned modes do not
    /// request an injected directive.
    pub fn prepare_source<'a>(self, source: &'a str) -> Cow<'a, str> {
        if self == Self::StrictScript {
            Cow::Owned(format!("\"use strict\";\n{source}"))
        } else {
            Cow::Borrowed(source)
        }
    }
}

/// Why Test262 frontmatter could not be interpreted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataErrorKind {
    /// Missing, malformed, duplicate, or contradictory metadata.
    Invalid,
    /// Validity or meaning requires YAML syntax or a field not yet supported.
    Unsupported,
    /// The frontmatter exceeds an opted-in host size quota.
    Limit,
}

/// A metadata error, separate from any JavaScript test result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataError {
    /// Failure category.
    pub kind: MetadataErrorKind,
    /// Explanation of the rejected metadata.
    pub message: String,
}
impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} metadata: {}", self.kind, self.message)
    }
}
impl std::error::Error for MetadataError {}

fn error(kind: MetadataErrorKind, message: impl Into<String>) -> MetadataError {
    MetadataError {
        kind,
        message: message.into(),
    }
}

/// Execution-relevant Test262 frontmatter.
///
/// The reader accepts top-level fields, simple inline/block string lists, and a
/// block mapping for `negative`. Descriptive scalar/block fields are ignored.
/// YAML aliases, tags, escapes, flow mappings, and complex scalars are explicitly
/// unsupported. Unknown fields and flags are never silently ignored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    flags: BTreeSet<String>,
    /// Harness includes in their declared order, without the default harness.
    pub includes: Vec<String>,
    /// Feature annotations; these do not themselves change execution semantics.
    pub features: Vec<String>,
    /// Locale annotations, retained for host selection.
    pub locales: Vec<String>,
    /// Expected exception, if the test is negative.
    pub negative: Option<Negative>,
}

impl Metadata {
    /// Reads frontmatter without modifying the source or interpreting its JavaScript.
    pub fn parse(source: &str) -> Result<Self, MetadataError> {
        Self::parse_with_limit(source, None)
    }

    /// Reads frontmatter with an opted-in maximum UTF-8 frontmatter size.
    pub fn parse_with_frontmatter_limit(
        source: &str,
        max_bytes: usize,
    ) -> Result<Self, MetadataError> {
        Self::parse_with_limit(source, Some(max_bytes))
    }

    fn parse_with_limit(source: &str, max_bytes: Option<usize>) -> Result<Self, MetadataError> {
        use MetadataErrorKind::{Invalid, Limit, Unsupported};
        let start = source
            .find("/*---")
            .ok_or_else(|| error(Invalid, "missing frontmatter"))?
            + 5;
        let end = source[start..]
            .find("---*/")
            .ok_or_else(|| error(Invalid, "unterminated frontmatter"))?
            + start;
        let body = &source[start..end];
        if max_bytes.is_some_and(|limit| body.len() > limit) {
            return Err(error(Limit, "frontmatter size limit exceeded"));
        }
        let lines: Vec<_> = body.lines().collect();
        let mut index = 0;
        let mut seen = BTreeSet::new();
        let mut metadata = Self {
            flags: BTreeSet::new(),
            includes: Vec::new(),
            features: Vec::new(),
            locales: Vec::new(),
            negative: None,
        };
        while index < lines.len() {
            let line = lines[index];
            index += 1;
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            if line.starts_with(char::is_whitespace) {
                return Err(error(Unsupported, "unexpected frontmatter indentation"));
            }
            let (key, value) = line
                .split_once(':')
                .ok_or_else(|| error(Unsupported, "expected a top-level mapping field"))?;
            if !seen.insert(key) {
                return Err(error(Invalid, format!("duplicate field {key}")));
            }
            let value = value.trim();
            let block_start = index;
            while index < lines.len()
                && (lines[index].trim().is_empty()
                    || lines[index].starts_with(char::is_whitespace)
                    || lines[index].trim_start().starts_with('#'))
            {
                index += 1;
            }
            let block = &lines[block_start..index];
            match key {
                "flags" => {
                    for flag in list(value, block)? {
                        if !matches!(
                            flag.as_str(),
                            "onlyStrict"
                                | "noStrict"
                                | "module"
                                | "raw"
                                | "async"
                                | "generated"
                                | "CanBlockIsFalse"
                                | "CanBlockIsTrue"
                                | "non-deterministic"
                        ) {
                            return Err(error(
                                Unsupported,
                                format!("unknown execution flag {flag}"),
                            ));
                        }
                        if !metadata.flags.insert(flag) {
                            return Err(error(Invalid, "duplicate execution flag"));
                        }
                    }
                }
                "includes" => metadata.includes = list(value, block)?,
                "features" => metadata.features = list(value, block)?,
                "locale" => metadata.locales = list(value, block)?,
                "negative" => metadata.negative = Some(negative(value, block)?),
                "description" | "info" | "esid" | "es5id" | "es6id" | "author" => {
                    // Block text may contain arbitrary spec grammar. Its contents
                    // do not become execution fields, even if they look like YAML.
                    if !matches!(value, "|" | "|-" | "|+" | ">" | ">-" | ">+")
                        && significant(block).next().is_some()
                    {
                        return Err(error(
                            Unsupported,
                            "multiline descriptive scalar without a block indicator",
                        ));
                    }
                }
                _ => return Err(error(Unsupported, format!("unknown metadata field {key}"))),
            }
        }
        if metadata.has_flag("onlyStrict")
            && (metadata.has_flag("noStrict")
                || metadata.has_flag("raw")
                || metadata.has_flag("module"))
            || metadata.has_flag("module") && metadata.has_flag("noStrict")
            || metadata.has_flag("CanBlockIsTrue") && metadata.has_flag("CanBlockIsFalse")
        {
            return Err(error(Invalid, "contradictory execution flags"));
        }
        if metadata
            .negative
            .as_ref()
            .is_some_and(|n| n.phase == Phase::Resolution)
            && !metadata.has_flag("module")
        {
            return Err(error(Invalid, "resolution negative requires a module"));
        }
        Ok(metadata)
    }

    /// Returns whether a known execution flag was present.
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    /// Plans all required variants, including the default pair of Script modes.
    pub fn modes(&self) -> Vec<Mode> {
        if self.has_flag("module") {
            vec![Mode::Module]
        } else if self.has_flag("raw") || self.has_flag("noStrict") {
            vec![Mode::Script]
        } else if self.has_flag("onlyStrict") {
            vec![Mode::StrictScript]
        } else {
            vec![Mode::Script, Mode::StrictScript]
        }
    }

    /// Lists harness files in execution order; raw tests have no harness.
    pub fn harness_files(&self) -> Vec<&str> {
        if self.has_flag("raw") {
            return Vec::new();
        }
        let mut files = vec!["assert.js", "sta.js"];
        if self.has_flag("async") {
            files.push("doneprintHandle.js");
        }
        files.extend(self.includes.iter().map(String::as_str));
        files
    }
}

fn significant<'a>(block: &'a [&str]) -> impl Iterator<Item = &'a str> {
    block
        .iter()
        .copied()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
}

fn scalar(text: &str) -> Result<String, MetadataError> {
    use MetadataErrorKind::{Invalid, Unsupported};
    let text = text.trim();
    if text.is_empty() {
        return Err(error(Invalid, "empty metadata value"));
    }
    let unsigned = text.trim_start_matches(['+', '-']);
    if !text.starts_with(['\'', '"'])
        && (matches!(
            text.to_ascii_lowercase().as_str(),
            "null" | "true" | "false" | "yes" | "no" | "on" | "off" | "y" | "n"
        ) || matches!(unsigned.to_ascii_lowercase().as_str(), ".inf" | ".nan")
            || unsigned
                .trim_start_matches('.')
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_digit))
    {
        return Err(error(Unsupported, "implicit non-string YAML scalar"));
    }
    let value = if text.starts_with(['\'', '"']) {
        let quote = text.as_bytes()[0] as char;
        if text.len() < 2 || !text.ends_with(quote) {
            return Err(error(Unsupported, "unclosed quoted scalar"));
        }
        &text[1..text.len() - 1]
    } else {
        text
    };
    if value.is_empty() {
        return Err(error(Invalid, "empty metadata value"));
    }
    if !value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"_-./".contains(&b))
    {
        return Err(error(Unsupported, "complex metadata scalar"));
    }
    Ok(value.into())
}

fn list(value: &str, block: &[&str]) -> Result<Vec<String>, MetadataError> {
    use MetadataErrorKind::{Invalid, Unsupported};
    if !value.is_empty() {
        if significant(block).next().is_some() {
            return Err(error(
                Unsupported,
                "inline list with additional block contents",
            ));
        }
        let body = value
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .ok_or_else(|| error(Unsupported, "expected a simple inline list"))?
            .trim();
        if body.is_empty() {
            return Ok(Vec::new());
        }
        return body.split(',').map(scalar).collect();
    }
    let mut values = Vec::new();
    let mut indentation = None;
    for line in significant(block) {
        if line.contains('\t') {
            return Err(error(Unsupported, "tabs in list indentation"));
        }
        let trimmed = line.trim_start();
        let spaces = line.len() - trimmed.len();
        if spaces == 0 || indentation.is_some_and(|n| n != spaces) {
            return Err(error(Unsupported, "inconsistent list indentation"));
        }
        indentation = Some(spaces);
        values.push(scalar(
            trimmed
                .strip_prefix("- ")
                .ok_or_else(|| error(Unsupported, "expected a block-list item"))?,
        )?);
    }
    if values.is_empty() {
        return Err(error(Invalid, "missing list value"));
    }
    Ok(values)
}

fn negative(value: &str, block: &[&str]) -> Result<Negative, MetadataError> {
    use MetadataErrorKind::{Invalid, Unsupported};
    if !value.is_empty() {
        return Err(error(Unsupported, "negative must be a block mapping"));
    }
    let mut phase = None;
    let mut error_type = None;
    let mut indentation = None;
    for line in significant(block) {
        if line.contains('\t') {
            return Err(error(Unsupported, "tabs in negative mapping"));
        }
        let trimmed = line.trim_start();
        let spaces = line.len() - trimmed.len();
        if spaces == 0 || indentation.is_some_and(|n| n != spaces) {
            return Err(error(Unsupported, "inconsistent negative indentation"));
        }
        indentation = Some(spaces);
        let (key, value) = trimmed
            .split_once(':')
            .ok_or_else(|| error(Unsupported, "expected a negative mapping field"))?;
        let value = scalar(value)?;
        match key {
            "phase" if phase.is_none() => {
                phase = Some(match value.as_str() {
                    "parse" => Phase::Parse,
                    "resolution" => Phase::Resolution,
                    "runtime" => Phase::Runtime,
                    _ => return Err(error(Invalid, "unknown negative phase")),
                })
            }
            "type" if error_type.is_none() => error_type = Some(value),
            "phase" | "type" => return Err(error(Invalid, "duplicate negative field")),
            _ => return Err(error(Unsupported, "unknown negative field")),
        }
    }
    Ok(Negative {
        phase: phase.ok_or_else(|| error(Invalid, "missing negative phase"))?,
        error_type: error_type.ok_or_else(|| error(Invalid, "missing negative type"))?,
    })
}
