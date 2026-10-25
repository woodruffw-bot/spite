"""Build static RegExp expectations from explicitly captured native results.

Set SPITE_REGEXP_ORACLE_CAPTURE to a TSV path when running Runtime tests with
Node 24.19 available, then pass the captured files to this tool. Capture mode
also compares the actual engine result. Normal tests use only the static table.
Review generated changes before committing; Node is a comparison source, not
the authority for edition-17 semantic differences.
"""

import argparse
import json
from pathlib import Path


def raw_string(value):
    hashes = "########"
    while '"' + hashes in value:
        hashes += "#"
    return "r" + hashes + '"' + value + '"' + hashes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("captures", type=Path, nargs="+")
    args = parser.parse_args()
    rows = {}
    for path in args.captures:
        for line in path.read_text(encoding="utf-8").splitlines():
            source, expected = (
                bytes.fromhex(value).decode("utf-8") for value in line.split("\t")
            )
            json.loads(expected)
            if source in rows and rows[source] != expected:
                raise ValueError(f"Conflicting native results for {source!r}")
            # Native resource exhaustion is not a language-result expectation.
            if expected != '["throw","RangeError"]':
                rows[source] = expected
    output = (
        "// Native results for previously unsupported RegExp compositions.\n"
        "// Generated with Node 24.19; see docs/coverage.md for review policy.\n"
        "const EXPECTATIONS: &[(&str, &str)] = &[\n"
    )
    output += "".join(
        f"    ({raw_string(source)}, {raw_string(expected)}),\n"
        for source, expected in sorted(rows.items())
    )
    output += "];\n"
    root = Path(__file__).resolve().parent.parent
    destination = root / "crates/spite-runtime/tests/common/regexp_oracle_data.rs"
    destination.write_text(output, encoding="utf-8")
    print(f"Generated {len(rows)} static native expectations.")


if __name__ == "__main__":
    main()
