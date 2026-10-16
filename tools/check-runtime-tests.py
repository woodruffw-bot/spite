#!/usr/bin/env python3
"""Verify that the grouped runtime integration target includes every case file."""

from pathlib import Path, PurePosixPath
import re
import sys


COMMON = re.compile(r"\s*mod\s+common\s*;")
CASE = re.compile(
    r'\s*#\[\s*path\s*=\s*"([^"\r\n]+)"\s*\]\s*'
    r"mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;"
)


def check(root: Path) -> int:
    """Return the case-file count, or raise ValueError/OSError for an invalid inventory.

    The manifest deliberately accepts only comments, ``mod common;``, and plain
    path/module pairs. Rejecting other Rust syntax keeps conditional modules from
    silently omitting tests. Fixture helpers beneath cases/fixtures are excluded.
    """
    tests = root / "crates/spite-runtime/tests"
    targets = {path.name for path in tests.glob("*.rs")}
    if targets != {"integration.rs"}:
        raise ValueError(
            "runtime tests must have only the integration.rs root target; found "
            + ", ".join(sorted(targets))
        )
    directory_targets = sorted(tests.glob("*/main.rs"))
    if directory_targets:
        raise ValueError(
            "unexpected runtime directory test target: "
            + directory_targets[0].relative_to(tests).as_posix()
        )
    if not (tests / "common/mod.rs").is_file():
        raise ValueError("runtime common/mod.rs helper is missing")

    source = (tests / "integration.rs").read_text(encoding="utf-8")
    source = re.sub(r"(?m)^[ \t]*//[^\n]*(?:\n|$)", "", source)
    common = COMMON.match(source)
    if common is None:
        raise ValueError("integration.rs must begin with unconditional mod common;")
    position = common.end()
    declared = set()
    modules = {"common"}
    while source[position:].strip():
        declaration = CASE.match(source, position)
        if declaration is None:
            raise ValueError(
                "integration.rs accepts only unconditional #[path] module pairs "
                "after mod common;"
            )
        path, module = declaration.groups()
        relative = PurePosixPath(path)
        if (
            relative.is_absolute()
            or relative.as_posix() != path
            or "\\" in path
            or ".." in relative.parts
            or not relative.parts
            or relative.parts[0] != "cases"
            or relative.suffix != ".rs"
        ):
            raise ValueError(f"invalid runtime case path: {path}")
        if module != relative.stem:
            raise ValueError(f"runtime module must match its case-file stem: {path}")
        if path in declared:
            raise ValueError(f"duplicate runtime case path: {path}")
        if module in modules:
            raise ValueError(f"duplicate runtime module: {module}")
        declared.add(path)
        modules.add(module)
        position = declaration.end()

    actual = {
        path.relative_to(tests).as_posix()
        for path in (tests / "cases").rglob("*.rs")
        if path.relative_to(tests / "cases").parts[0] != "fixtures"
    }
    if not actual:
        raise ValueError("runtime case inventory is empty")
    if missing := sorted(actual - declared):
        raise ValueError("runtime cases missing from integration.rs: " + ", ".join(missing))
    if extra := sorted(declared - actual):
        raise ValueError("declared runtime cases are missing or are helpers: " + ", ".join(extra))
    return len(actual)


def main() -> int:
    try:
        count = check(Path(__file__).resolve().parent.parent)
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print(f"Verified all {count} runtime case files in the grouped integration target.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
