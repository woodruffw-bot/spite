"""Reject external production dependencies and missing workspace lint inheritance."""

import json
import pathlib
import subprocess
import tomllib

metadata = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"], text=True
))
members = set(metadata["workspace_members"])
paths = {
    pathlib.Path(p["manifest_path"]).parent.resolve()
    for p in metadata["packages"] if p["id"] in members
}
errors = []
workspace = tomllib.loads((pathlib.Path(metadata["workspace_root"]) / "Cargo.toml").read_text())
if workspace.get("workspace", {}).get("lints", {}).get("rust", {}).get("unsafe_code") != "forbid":
    errors.append("workspace must forbid unsafe_code")
for package in metadata["packages"]:
    if package["id"] not in members:
        continue
    manifest = tomllib.loads(pathlib.Path(package["manifest_path"]).read_text())
    if manifest.get("lints", {}).get("workspace") is not True:
        errors.append(f'{package["name"]}: workspace lints must be inherited')
    for dep in package["dependencies"]:
        if dep["kind"] == "dev":
            continue
        path = dep.get("path")
        if not path or pathlib.Path(path).resolve() not in paths:
            errors.append(f'{package["name"]}: external production dependency {dep["name"]}')
if errors:
    raise SystemExit("\n".join(errors))
print("All workspace crates inherit lints and use only workspace production dependencies.")
