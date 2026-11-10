#!/usr/bin/env python3
"""Verify the exact published ECMA-262 source before using it as audit evidence."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import urllib.request


def check(data, identity):
    actual = {
        "size": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
        "git_blob": hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest(),
    }
    for key, value in actual.items():
        if identity[key] != value:
            raise ValueError(f"published specification {key} mismatch: expected {identity[key]}, got {value}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", type=Path, help="local unmodified spec.html")
    parser.add_argument("--download", action="store_true", help="fetch the exact pinned source")
    args = parser.parse_args()
    if (args.source is None) == (not args.download):
        parser.error("provide a local source or --download")
    root = Path(__file__).resolve().parent.parent
    identity = json.loads((root / "tests/test262/audit/specification.json").read_text())
    if not re.fullmatch(r"[0-9a-f]{40}", identity["revision"]):
        raise ValueError("specification revision must be a complete commit SHA")
    url = f'https://raw.githubusercontent.com/tc39/ecma262/{identity["revision"]}/{identity["path"]}'
    if identity["source_url"] != url:
        raise ValueError("specification URL must name its pinned revision and path")
    if args.download:
        with urllib.request.urlopen(url, timeout=40) as response:
            data = response.read()
    else:
        data = args.source.read_bytes()
    check(data, identity)
    print(f'Verified ECMA-262 edition {identity["edition"]}: {identity["revision"]}, blob {identity["git_blob"]}.')


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError) as error:
        raise SystemExit(str(error)) from error
