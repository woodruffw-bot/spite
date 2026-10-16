"""Generate RegExp Script and Script_Extensions sets from local Unicode data."""

import argparse
from array import array
import hashlib
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
REFERENCE = ROOT / "crates/spite-core/src/testdata"
OUTPUT = ROOT / "crates/spite-core/src/regexp_script_data.rs"
EXTENSIONS_OUTPUT = ROOT / "crates/spite-core/src/regexp_script_extensions_data.rs"
VERSION = "18.0.0"
LIMIT = 0x110000
FIRST_KIND = 206
EXTENSIONS_FIRST_KIND = 540
SOURCES = {
    "PropertyValueAliases": "06c4c8eaf7b0bf34abe73b113da1215bd784ac254d4c223600b90267caa4bbbd",
    "Scripts": "0071fd81b6aeae25f6e8bce8efec3066a6476a91b49bdb2f52dc76e817862a6a",
    "CaseFolding": "a004797658a457bec4dc11683e39f69249ea3b595b752dbea6721c4c9f587b0d",
}
# Keep SOURCES and its metadata order unchanged for the existing Script output.
EXTENSION_SOURCES = {
    **SOURCES,
    "ScriptExtensions": "5c9d34a922f687726f2a8bcf57d49f905987e51f1b21b58c95a00fbe255cec23",
}
# Preserve all earlier property kinds and their specialized complement plans.
LEGACY = {
    "Han": 26,
    "Latin": 184,
    "Greek": 186,
    "Cyrillic": 188,
    "Arabic": 190,
    "Hebrew": 192,
    "Common": 194,
    "Inherited": 196,
    "Devanagari": 198,
    "Bengali": 200,
    "Gurmukhi": 202,
    "Gujarati": 204,
}


def records(data):
    for line in data.decode("utf-8").splitlines():
        fields = [field.strip() for field in line.split("#", 1)[0].split(";")]
        if len(fields) > 1:
            yield fields


def complement(intervals):
    result = []
    cursor = 0
    for first, last in intervals:
        if cursor < first:
            result.append((cursor, first - 1))
        cursor = last + 1
    if cursor < LIMIT:
        result.append((cursor, LIMIT - 1))
    return result


def parse_sources(sources):
    for name, digest in EXTENSION_SOURCES.items():
        if hashlib.sha256(sources[name]).hexdigest() != digest:
            raise ValueError(f"{name} differs from the pinned SHA-256")
    values = {}
    aliases = set()
    for fields in records(sources["PropertyValueAliases"]):
        if fields[0] != "sc":
            continue
        canonical = fields[2]
        names = set(fields[1:])
        if canonical in values or aliases.intersection(names):
            raise ValueError("ambiguous Script aliases")
        if any(not re.fullmatch(r"[A-Za-z_0-9]+", name) for name in names):
            raise ValueError("non-ASCII Script alias")
        values[canonical] = sorted(names)
        aliases.update(names)
    if len(values) != 179 or len(aliases) != 351 or not LEGACY.keys() <= values.keys():
        raise ValueError("unexpected canonical Script or alias inventory")
    names = sorted(values)
    indices = {name: index for index, name in enumerate(names)}
    unknown = indices["Unknown"]
    # Include @missing and canonical values with no explicit Scripts records.
    if "# @missing: 0000..10FFFF; Unknown" not in sources["Scripts"].decode("utf-8"):
        raise ValueError("unexpected Scripts default")
    owner = array("H", [unknown]) * LIMIT
    explicit = bytearray(LIMIT)
    for fields in records(sources["Scripts"]):
        bounds = fields[0].split("..")
        first, last = int(bounds[0], 16), int(bounds[-1], 16)
        if fields[1] not in indices or not 0 <= first <= last < LIMIT:
            raise ValueError("invalid Scripts interval")
        if any(explicit[first:last + 1]):
            raise ValueError("overlapping Scripts records")
        explicit[first:last + 1] = b"\1" * (last - first + 1)
        owner[first:last + 1] = array("H", [indices[fields[1]]]) * (last - first + 1)
    intervals = {name: [] for name in names}
    first = 0
    for point in range(1, LIMIT + 1):
        if point == LIMIT or owner[point] != owner[first]:
            intervals[names[owner[first]]].append((first, point - 1))
            first = point
    if intervals["Katakana_Or_Hiragana"] or len(intervals["Unknown"]) != 733:
        raise ValueError("unexpected empty/default Script membership")
    remaining = sorted(values.keys() - LEGACY.keys())
    crossings = set()
    folds = []
    folded_sources = set()
    for fields in records(sources["CaseFolding"]):
        if fields[1] not in ("C", "S"):
            continue
        targets = fields[2].split()
        if len(targets) != 1:
            raise ValueError("a simple/common fold has multiple targets")
        source, target = int(fields[0], 16), int(targets[0], 16)
        if not 0 <= source < LIMIT or not 0 <= target < LIMIT:
            raise ValueError("fold outside Unicode domain")
        if source in folded_sources:
            raise ValueError("duplicate common/simple fold source")
        folded_sources.add(source)
        folds.append((source, target))
        if owner[source] != owner[target]:
            source_script, target_script = names[owner[source]], names[owner[target]]
            crossings.add((source, target, source_script, target_script))
            if source_script in remaining or target_script in remaining:
                raise ValueError("a generic Script is not C/S-fold invariant")
    if len(folds) != 1533 or crossings != {
        (0xB5, 0x3BC, "Common", "Greek"),
        (0x345, 0x3B9, "Inherited", "Greek"),
    }:
        raise ValueError("unexpected fold inventory; review legacy specialized plans")
    fold_map = dict(folds)
    if any(fold_map.get(target, target) != target for _, target in folds):
        raise ValueError("common/simple folding is not idempotent")
    return values, names, owner, intervals, folds


def generate_scripts(values, intervals):
    remaining = sorted(values.keys() - LEGACY.keys())
    end = FIRST_KIND + 2 * len(remaining)
    if len(remaining) != 167 or end != 540 or end > 0x10000:
        raise ValueError("unexpected property-kind bound")
    alias_kinds = sorted(
        (alias, FIRST_KIND + 2 * index)
        for index, name in enumerate(remaining)
        for alias in values[name]
    )
    if len(alias_kinds) != 326:
        raise ValueError("unexpected remaining Script aliases")
    lines = [
        "//! Pinned Unicode 18 Script sets invariant under common/simple folding.",
        "// Generated by tools/generate-regexp-scripts.py. Do not edit.",
        "// Copyright © 2026 Unicode, Inc. See third-party/unicode/LICENSE.",
    ]
    for name, digest in SOURCES.items():
        lines += [f"// Source: https://www.unicode.org/Public/{VERSION}/ucd/{name}.txt", f"// SHA-256: {digest}"]
    lines += [
        "// Existing kinds 0..205, including specialized non-invariant Scripts, are unchanged.",
        "// Unknown honors @missing; Katakana_Or_Hiragana is a valid empty Script.",
        "",
        f"pub(crate) const FIRST_KIND: u16 = {FIRST_KIND};",
        f"pub(crate) const KIND_COUNT: u16 = {end};",
        "",
        "pub(crate) fn kind(name: &[u16]) -> Option<u16> {",
        "    let name = name",
        "        .strip_prefix(&[0x53, 0x63, 0x72, 0x69, 0x70, 0x74, 0x3d])",
        "        .or_else(|| name.strip_prefix(&[0x73, 0x63, 0x3d]))?;",
        "    let index = ALIASES",
        "        .binary_search_by(|&(alias, _)| alias.encode_utf16().cmp(name.iter().copied()))",
        "        .ok()?;",
        "    Some(ALIASES[index].1)",
        "}",
        "",
        "pub(crate) fn construction_intervals(kind: u16) -> Option<usize> {",
        "    let sets = INTERVALS.get(usize::from(kind.checked_sub(FIRST_KIND)?) / 2)?;",
        "    Some(sets.positive.len().max(sets.negative.len()))",
        "}",
        "",
        "pub(crate) fn ranges(kind: u16) -> Option<&'static [(u32, u32)]> {",
        "    let offset = kind.checked_sub(FIRST_KIND)?;",
        "    let sets = INTERVALS.get(usize::from(offset) / 2)?;",
        "    Some(if offset % 2 == 0 {",
        "        sets.positive",
        "    } else {",
        "        sets.negative",
        "    })",
        "}",
        "",
        "struct ScriptIntervals {",
        "    positive: &'static [(u32, u32)],",
        "    negative: &'static [(u32, u32)],",
        "}",
        "",
        "const ALIASES: &[(&str, u16)] = &[",
    ]
    lines += [f'    ("{alias}", {kind}),' for alias, kind in alias_kinds]
    lines += ["];", "", "#[rustfmt::skip]", "const INTERVALS: &[ScriptIntervals] = &["]
    pair_count = 0
    for index, name in enumerate(remaining):
        positive = intervals[name]
        negative = complement(positive)
        if sum(last - first + 1 for first, last in positive + negative) != LIMIT:
            raise ValueError("invalid complementary partition")
        lines += [f"    // {name}, kinds {FIRST_KIND + 2 * index}/{FIRST_KIND + 2 * index + 1}.", "    ScriptIntervals {"]
        for polarity, spans in [("positive", positive), ("negative", negative)]:
            pair_count += len(spans)
            if not spans:
                lines += [f"        {polarity}: &[],"]
            else:
                lines += [f"        {polarity}: &["]
                lines += [f"            (0x{first:x}, 0x{last:x})," for first, last in spans]
                lines += ["        ],"]
        lines += ["    },"]
    if pair_count != 2764:
        raise ValueError("unexpected generated interval inventory")
    lines += ["];", ""]
    return "\n".join(lines).encode("utf-8")


def normalized(intervals):
    result = []
    for first, last in sorted(intervals):
        if not 0 <= first <= last < LIMIT:
            raise ValueError("interval outside Unicode domain")
        if result and first <= result[-1][1] + 1:
            result[-1] = (result[-1][0], max(last, result[-1][1]))
        else:
            result.append((first, last))
    return result


def contains(intervals, point):
    low, high = 0, len(intervals)
    while low < high:
        middle = (low + high) // 2
        first, last = intervals[middle]
        if point < first:
            high = middle
        elif point > last:
            low = middle + 1
        else:
            return True
    return False


def extension_intervals(source, values, names, owner):
    if "# @missing: 0000..10FFFF; <script>" not in source.decode("utf-8"):
        raise ValueError("unexpected Script_Extensions default")
    aliases = {alias: index for index, name in enumerate(names) for alias in values[name]}
    singletons = [1 << index for index in range(len(names))]
    # A missing record is the singleton Script set. An explicit record replaces it.
    membership = [singletons[index] for index in owner]
    explicit = bytearray(LIMIT)
    overrides = 0
    for fields in records(source):
        bounds = fields[0].split("..")
        first, last = int(bounds[0], 16), int(bounds[-1], 16)
        if len(fields) != 2 or not 0 <= first <= last < LIMIT:
            raise ValueError("invalid Script_Extensions interval")
        if any(explicit[first:last + 1]):
            raise ValueError("overlapping Script_Extensions records")
        members = fields[1].split()
        if not members or any(alias not in aliases for alias in members):
            raise ValueError("invalid Script_Extensions value")
        indices = {aliases[alias] for alias in members}
        if len(indices) != len(members):
            raise ValueError("duplicate Script_Extensions value")
        mask = sum(singletons[index] for index in indices)
        membership[first:last + 1] = [mask] * (last - first + 1)
        explicit[first:last + 1] = b"\1" * (last - first + 1)
        overrides += 1
    if overrides != 210 or sum(explicit) != 741:
        raise ValueError("unexpected Script_Extensions override inventory")
    intervals = {name: [] for name in names}
    starts = [None] * len(names)
    previous = 0
    # One domain scan, with transitions only for values whose membership changes.
    for point in range(LIMIT + 1):
        current = membership[point] if point < LIMIT else 0
        changed = previous ^ current
        while changed:
            bit = changed & -changed
            index = bit.bit_length() - 1
            if current & bit:
                starts[index] = point
            else:
                intervals[names[index]].append((starts[index], point - 1))
            changed ^= bit
        previous = current
    if intervals["Katakana_Or_Hiragana"] or sum(
        last - first + 1 for first, last in intervals["Unknown"]
    ) != 941239:
        raise ValueError("unexpected empty/default Script_Extensions membership")
    return intervals


def generate_extensions(values, names, owner, folds, source):
    intervals = extension_intervals(source, values, names, owner)
    end = EXTENSIONS_FIRST_KIND + 2 * len(names)
    if EXTENSIONS_FIRST_KIND != 540 or end != 898 or end > 0x10000:
        raise ValueError("unexpected Script_Extensions property-kind bound")
    alias_kinds = sorted(
        (alias, EXTENSIONS_FIRST_KIND + 2 * index)
        for index, name in enumerate(names)
        for alias in values[name]
    )
    if len(alias_kinds) != 351:
        raise ValueError("unexpected Script_Extensions aliases")
    lines = [
        "//! Pinned Unicode 18 Script_Extensions sets and Unicode-set folding plans.",
        "// Generated by tools/generate-regexp-scripts.py. Do not edit.",
        "// Copyright © 2026 Unicode, Inc. See third-party/unicode/LICENSE.",
    ]
    for name, digest in EXTENSION_SOURCES.items():
        lines += [f"// Source: https://www.unicode.org/Public/{VERSION}/ucd/{name}.txt", f"// SHA-256: {digest}"]
    lines += [
        "// Explicit Script_Extensions records replace singleton Script fallback sets.",
        "// Unknown honors @missing; Katakana_Or_Hiragana is a valid empty set.",
        "// Flagged negatives exclude all fold preimages before a v+i class union.",
        "",
        f"pub(crate) const FIRST_KIND: u16 = {EXTENSIONS_FIRST_KIND};",
        f"pub(crate) const KIND_COUNT: u16 = {end};",
        "",
        "pub(crate) fn kind(name: &[u16]) -> Option<u16> {",
        "    let name = name",
        "        .strip_prefix(&[",
        "            0x53, 0x63, 0x72, 0x69, 0x70, 0x74, 0x5f, 0x45, 0x78, 0x74, 0x65, 0x6e, 0x73, 0x69,",
        "            0x6f, 0x6e, 0x73, 0x3d,",
        "        ])",
        "        .or_else(|| name.strip_prefix(&[0x73, 0x63, 0x78, 0x3d]))?;",
        "    let index = ALIASES",
        "        .binary_search_by(|&(alias, _)| alias.encode_utf16().cmp(name.iter().copied()))",
        "        .ok()?;",
        "    Some(ALIASES[index].1)",
        "}",
        "",
        "pub(crate) fn construction_intervals(kind: u16) -> Option<usize> {",
        "    let sets = INTERVALS.get(usize::from(kind.checked_sub(FIRST_KIND)?) / 2)?;",
        "    let flagged = sets",
        "        .negative_unicode_sets_ignore_case",
        "        .unwrap_or(sets.negative);",
        "    Some(",
        "        sets.positive",
        "            .len()",
        "            .max(sets.negative.len())",
        "            .max(flagged.len()),",
        "    )",
        "}",
        "",
        "pub(crate) fn ranges(",
        "    kind: u16,",
        "    unicode_sets: bool,",
        "    ignore_case: bool,",
        ") -> Option<&'static [(u32, u32)]> {",
        "    let offset = kind.checked_sub(FIRST_KIND)?;",
        "    let sets = INTERVALS.get(usize::from(offset) / 2)?;",
        "    Some(if offset % 2 == 0 {",
        "        sets.positive",
        "    } else if unicode_sets && ignore_case {",
        "        sets.negative_unicode_sets_ignore_case",
        "            .unwrap_or(sets.negative)",
        "    } else {",
        "        sets.negative",
        "    })",
        "}",
        "",
        "struct ScriptExtensionIntervals {",
        "    positive: &'static [(u32, u32)],",
        "    negative: &'static [(u32, u32)],",
        "    negative_unicode_sets_ignore_case: Option<&'static [(u32, u32)]>,",
        "}",
        "",
        "const ALIASES: &[(&str, u16)] = &[",
    ]
    lines += [f'    ("{alias}", {kind}),' for alias, kind in alias_kinds]
    lines += ["];", "", "#[rustfmt::skip]", "const INTERVALS: &[ScriptExtensionIntervals] = &["]
    pair_count = 0
    positive_count = 0
    negative_count = 0
    flagged_count = 0
    membership_count = 0
    for index, name in enumerate(names):
        positive = intervals[name]
        if positive != normalized(positive):
            raise ValueError("unnormalized Script_Extensions intervals")
        negative = complement(positive)
        # The shared constructor adds canonical targets to raw ranges. In a v+i
        # negative class member, first exclude that image and all its preimages.
        folded = normalized(positive + [
            (target, target) for source, target in folds if contains(positive, source)
        ])
        preimages = normalized(folded + [
            (source, source) for source, target in folds if contains(folded, target)
        ])
        flagged = complement(preimages)
        if any(contains(preimages, source) != contains(preimages, target) for source, target in folds):
            raise ValueError("Script_Extensions fold-preimage closure is incomplete")
        if negative != normalized(negative) or flagged != normalized(flagged):
            raise ValueError("unnormalized Script_Extensions complement")
        if sum(last - first + 1 for first, last in positive + negative) != LIMIT:
            raise ValueError("invalid Script_Extensions complementary partition")
        positive_count += len(positive)
        negative_count += len(negative)
        membership_count += sum(last - first + 1 for first, last in positive)
        lines += [f"    // {name}, kinds {EXTENSIONS_FIRST_KIND + 2 * index}/{EXTENSIONS_FIRST_KIND + 2 * index + 1}.", "    ScriptExtensionIntervals {"]
        fields = [("positive", positive, False), ("negative", negative, False)]
        if flagged != negative:
            fields.append(("negative_unicode_sets_ignore_case", flagged, True))
            flagged_count += 1
        for polarity, spans, optional in fields:
            pair_count += len(spans)
            if not spans:
                lines += [f"        {polarity}: " + ("Some(&[])," if optional else "&[],")]
            else:
                lines += [f"        {polarity}: " + ("Some(&[" if optional else "&[")]
                lines += [f"            (0x{first:x}, 0x{last:x})," for first, last in spans]
                lines += ["        ])," if optional else "        ],"]
        if flagged == negative:
            lines += ["        negative_unicode_sets_ignore_case: None,"]
        lines += ["    },"]
    if (positive_count, negative_count, flagged_count, pair_count, membership_count) != (
        2107, 2284, 2, 4604, 1115175
    ):
        raise ValueError("unexpected Script_Extensions interval inventory")
    lines += ["];", ""]
    return "\n".join(lines).encode("utf-8")


def generate(sources):
    values, names, owner, intervals, folds = parse_sources(sources)
    return (
        generate_scripts(values, intervals),
        generate_extensions(values, names, owner, folds, sources["ScriptExtensions"]),
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", type=Path, default=REFERENCE, help="local directory of unchanged versioned UCD reference files")
    parser.add_argument("--output", type=Path, default=OUTPUT, help="generated Rust output")
    parser.add_argument("--extensions-output", type=Path, default=EXTENSIONS_OUTPUT, help="generated Script_Extensions Rust output")
    parser.add_argument("--check", action="store_true", help="verify both checked-in outputs without writing or downloading")
    args = parser.parse_args()
    sources = {name: (args.source / f"{name}-{VERSION}.txt").read_bytes() for name in EXTENSION_SOURCES}
    scripts, extensions = generate(sources)
    if args.check:
        if args.output.read_bytes() != scripts:
            raise SystemExit("RegExp Script tables differ from pinned data. Regenerate and review them.")
        if args.extensions_output.read_bytes() != extensions:
            raise SystemExit("RegExp Script_Extensions tables differ from pinned data. Regenerate and review them.")
        print(f"Verified Unicode {VERSION} RegExp Script and Script_Extensions tables from local references.")
    else:
        args.output.write_bytes(scripts)
        args.extensions_output.write_bytes(extensions)
        print(f"Generated Unicode {VERSION} RegExp Script and Script_Extensions tables from local references.")


if __name__ == "__main__":
    main()
