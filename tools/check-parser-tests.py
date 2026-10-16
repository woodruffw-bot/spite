#!/usr/bin/env python3
"""Verify complete, unconditional Parser integration-test discovery."""

from pathlib import Path, PurePosixPath
import re
import sys
import tomllib


PATH = re.compile(r'#\[\s*path\s*=\s*"([^"\r\n]+)"\s*\]')
MODULE = re.compile(r"mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")


def skip_comments(source: str, position: int) -> int:
    """Skip whitespace and Rust line or nested block comments between tokens."""
    while position < len(source):
        if source[position].isspace():
            position += 1
        elif source.startswith("//", position):
            ending = source.find("\n", position + 2)
            position = len(source) if ending < 0 else ending + 1
        elif source.startswith("/*", position):
            position += 2
            depth = 1
            while depth and position < len(source):
                if source.startswith("/*", position):
                    depth += 1
                    position += 2
                elif source.startswith("*/", position):
                    depth -= 1
                    position += 2
                else:
                    position += 1
            if depth:
                raise ValueError("unterminated comment in Parser integration.rs")
        else:
            break
    return position


def check(root: Path) -> int:
    """Return the case-file count, or raise ValueError/OSError for an invalid inventory.

    Every Rust file beneath tests/, except the integration manifest itself, is a
    case. Only comments and unconditional path/module pairs belong in the manifest.
    """
    crate = root / "crates/spite-parser"
    cargo = tomllib.loads((crate / "Cargo.toml").read_text(encoding="utf-8"))
    package = cargo.get("package", {})
    if not isinstance(package, dict) or package.get("autotests") is not False:
        raise ValueError("Parser package.autotests must be false")
    targets = cargo.get("test", [])
    if not isinstance(targets, list) or len(targets) != 1:
        raise ValueError("Parser must declare exactly one explicit parser_integration test target")
    target = targets[0]
    if (
        not isinstance(target, dict)
        or target.get("name") != "parser_integration"
        or target.get("path") != "tests/integration.rs"
        or set(target) - {"name", "path", "harness", "test"}
        or target.get("harness", True) is not True
        or target.get("test", True) is not True
    ):
        raise ValueError(
            "Parser test target must be parser_integration at tests/integration.rs, "
            "with the default harness and no disabling or feature options"
        )

    tests = crate / "tests"
    manifest = tests / "integration.rs"
    if tests.is_symlink() or manifest.is_symlink():
        raise ValueError("Parser test paths cannot be symlinks")
    for path in tests.rglob("*"):
        if path.is_symlink() and (path.suffix == ".rs" or path.is_dir()):
            raise ValueError("Parser test paths cannot be symlinks")
    actual = set()
    for path in tests.rglob("*.rs"):
        if not path.is_file():
            raise ValueError("Parser Rust case path is not a file: " + str(path))
        if path != manifest:
            actual.add(path.relative_to(tests).as_posix())

    source = manifest.read_text(encoding="utf-8")
    position = skip_comments(source, 0)
    declared = set()
    modules = set()
    while position < len(source):
        attribute = PATH.match(source, position)
        declaration = None
        if attribute is not None:
            declaration = MODULE.match(source, skip_comments(source, attribute.end()))
        if declaration is None:
            raise ValueError("Parser integration.rs accepts only unconditional #[path] module pairs")
        path, module = attribute[1], declaration[1]
        relative = PurePosixPath(path)
        if (
            relative.is_absolute()
            or relative.as_posix() != path
            or "\\" in path
            or ":" in path
            or any(ord(char) < 32 or ord(char) == 127 for char in path)
            or ".." in relative.parts
            or relative.suffix != ".rs"
            or path == "integration.rs"
        ):
            raise ValueError(f"invalid Parser case path: {path}")
        if module != relative.stem:
            raise ValueError(f"Parser module must match its case-file stem: {path}")
        if path in declared:
            raise ValueError(f"duplicate Parser case path: {path}")
        if module in modules:
            raise ValueError(f"duplicate Parser module: {module}")
        declared.add(path)
        modules.add(module)
        position = skip_comments(source, declaration.end())

    if not actual:
        raise ValueError("Parser case inventory is empty")
    if missing := sorted(actual - declared):
        raise ValueError("Parser cases missing from integration.rs: " + ", ".join(missing))
    if extra := sorted(declared - actual):
        raise ValueError("declared Parser cases are missing: " + ", ".join(extra))
    return len(actual)


def main() -> int:
    try:
        count = check(Path(__file__).resolve().parent.parent)
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print(f"Verified all {count} Parser case files in the grouped integration target.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
