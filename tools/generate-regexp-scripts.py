"""Generate the remaining RegExp Script sets from unchanged local Unicode data."""

import argparse
from array import array
import hashlib
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
REFERENCE = ROOT / "crates/spite-core/src/testdata"
OUTPUT = ROOT / "crates/spite-core/src/regexp_script_data.rs"
VERSION = "18.0.0"
LIMIT = 0x110000
FIRST_KIND = 206
SOURCES = {
    "PropertyValueAliases": "06c4c8eaf7b0bf34abe73b113da1215bd784ac254d4c223600b90267caa4bbbd",
    "Scripts": "0071fd81b6aeae25f6e8bce8efec3066a6476a91b49bdb2f52dc76e817862a6a",
    "CaseFolding": "a004797658a457bec4dc11683e39f69249ea3b595b752dbea6721c4c9f587b0d",
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


def generate(sources):
    for name, digest in SOURCES.items():
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
    folds = 0
    for fields in records(sources["CaseFolding"]):
        if fields[1] not in ("C", "S"):
            continue
        targets = fields[2].split()
        if len(targets) != 1:
            raise ValueError("a simple/common fold has multiple targets")
        source, target = int(fields[0], 16), int(targets[0], 16)
        if not 0 <= source < LIMIT or not 0 <= target < LIMIT:
            raise ValueError("fold outside Unicode domain")
        folds += 1
        if owner[source] != owner[target]:
            source_script, target_script = names[owner[source]], names[owner[target]]
            crossings.add((source, target, source_script, target_script))
            if source_script in remaining or target_script in remaining:
                raise ValueError("a generic Script is not C/S-fold invariant")
    if folds != 1533 or crossings != {
        (0xB5, 0x3BC, "Common", "Greek"),
        (0x345, 0x3B9, "Inherited", "Greek"),
    }:
        raise ValueError("unexpected fold inventory; review legacy specialized plans")
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", type=Path, default=REFERENCE, help="local directory of unchanged versioned UCD reference files")
    parser.add_argument("--output", type=Path, default=OUTPUT, help="generated Rust output")
    parser.add_argument("--check", action="store_true", help="verify the checked-in output without writing or downloading")
    args = parser.parse_args()
    sources = {name: (args.source / f"{name}-{VERSION}.txt").read_bytes() for name in SOURCES}
    generated = generate(sources)
    if args.check:
        if args.output.read_bytes() != generated:
            raise SystemExit("RegExp Script tables differ from pinned data. Regenerate and review them.")
        print(f"Verified Unicode {VERSION} remaining RegExp Script tables from local references.")
    else:
        args.output.write_bytes(generated)
        print(f"Generated Unicode {VERSION} remaining RegExp Script tables from local references.")


if __name__ == "__main__":
    main()
