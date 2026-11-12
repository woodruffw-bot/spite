//! Match observations for RegExp consumer snapshots.
//!
//! The report keeps capture slots, UTF-16 ranges, named aliases, and group
//! prototype checks visible as separate fields. Undefined slots use a sentinel.

/// Starting offsets for match-result matrices, including the UTF-16 end boundary.
///
/// Short inputs can make the end or past-end offset coincide with 0, 1, or 2.
/// Each distinct offset is evaluated once.
pub fn last_index_cases(input_length: usize) -> Vec<usize> {
    let mut indices = vec![0, 1, 2, input_length, input_length + 1];
    indices.sort_unstable();
    indices.dedup();
    indices
}

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
