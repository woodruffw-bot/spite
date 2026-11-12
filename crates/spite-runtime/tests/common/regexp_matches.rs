//! Match observations for RegExp consumer snapshots.
//!
//! The report keeps capture slots, UTF-16 ranges, named aliases, and group
//! prototype checks visible as separate fields. Undefined slots use a sentinel.

pub const DESCRIBE_MATCH: &str = r#"
function describeMatch(match) {
    if (match === null) return null;
    var groupNames = match.groups === undefined ? [] : Object.keys(match.groups);
    return {
        matches: Array.from(match, value => value === undefined ? '<undefined>' : value),
        indices: Array.from(match.indices, value => value === undefined ? '<undefined>' : value),
        index: match.index,
        input: match.input,
        groupNames: groupNames,
        namedCaptures: groupNames.map(name =>
            match.groups[name] === undefined ? '<undefined>' : match.groups[name]),
        namedIndices: groupNames.map(name =>
            match.indices.groups[name] === undefined ? '<undefined>' : match.indices.groups[name]),
        namedAliasesValid: groupNames.every((name, slot) =>
            Object.hasOwn(match.groups, name) && Object.hasOwn(match.indices.groups, name) &&
            match.groups[name] === match[slot + 1] &&
            match.indices.groups[name] === match.indices[slot + 1]),
        groupsPrototypeValid: match.groups === undefined ||
            Object.getPrototypeOf(match.groups) === null
    };
}
"#;
