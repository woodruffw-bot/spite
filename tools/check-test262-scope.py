#!/usr/bin/env python3
"""Check the complete pinned Test262 universe and every retained file's provenance."""

from pathlib import Path

from test262_scope import check


if __name__ == "__main__":
    try:
        originals, retained, dispositions = check(Path(__file__).resolve().parent.parent / "tests/test262")
    except (ValueError, OSError) as error:
        raise SystemExit(str(error)) from error
    print(f"Verified pinned Git tree: {originals} originals, {retained} retained, {sum(dispositions.values())} scoped.")
    for name, count in sorted(dispositions.items()):
        print(f"  {name}: {count}")
