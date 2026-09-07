#!/usr/bin/env python3
"""Collect locked Cargo dependency notices without touching a signed bundle.

Requires local Cargo sources and Rust documentation; installs nothing and uses no
network. Missing crate license files may be supplied using --supplements JSON:
  {"packages": {"crate@version": {"selected_license": "MIT", "note": "...",
    "files": [{"path": "relative/file.txt", "sha256": "...", "url": "https://..."}]}}}
Each supplemental path is relative to that JSON. A previously generated public
manifest.json is also accepted, reusing its recorded supplemental license files.
Standard license templates must be labelled as such,
not represented as an upstream copyright notice. Existing upstream notices are
copied verbatim, including copyright and attribution. Output is deliberately a
conservative normal + build dependency inventory, not a binary-linkage claim.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
LICENSE_NAME = re.compile(r"^(?:licen[cs]e|copying|copyright|unlicense|notices?)(?:$|[._-])", re.I)
SAFE_COMPONENT = re.compile(r"^[A-Za-z0-9_.+-]+$")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def command(tool, toolchain, *args):
    cmd = [tool] + (["+" + toolchain] if toolchain else []) + list(args)
    return subprocess.check_output(cmd, cwd=ROOT, text=True).strip()


def inside(root, path):
    """Reject absolute paths, symlinks and traversal in supplemental input."""
    relative = Path(path)
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError("Supplement path must be relative and remain inside its directory")
    candidate = root / relative
    if any(part.is_symlink() for part in [candidate, *candidate.parents] if part != root.parent):
        # /tmp may itself be a platform alias; root is resolved by the caller.
        raise ValueError("Supplement path contains a symlink")
    candidate.resolve().relative_to(root)
    return candidate


def selected_packages(metadata):
    packages = {p["id"]: p for p in metadata["packages"]}
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    root_id = metadata["resolve"]["root"]
    seen, pending, incoming = set(), [root_id], {}
    while pending:
        node = pending.pop()
        if node in seen:
            continue
        seen.add(node)
        for dep in nodes[node]["deps"]:
            kinds = {kind["kind"] or "normal" for kind in dep["dep_kinds"] if kind["kind"] != "dev"}
            if kinds:
                pending.append(dep["pkg"])
                incoming.setdefault(dep["pkg"], set()).update(kinds)
    return packages[root_id], sorted(
        [(packages[p], sorted(incoming[p])) for p in seen if p != root_id],
        key=lambda row: (row[0]["name"], row[0]["version"]),
    )


def license_files(crate_root, explicit):
    files = set()
    if explicit:
        files.add(inside(crate_root, explicit))
    for path in crate_root.rglob("*"):
        if not path.is_file() or path.is_symlink():
            continue
        relative = path.relative_to(crate_root)
        # Source files called copying.rs implement NSObject copying, not licensing.
        if path.suffix.lower() in {".rs", ".c", ".h", ".cpp", ".py", ".js"}:
            continue
        if LICENSE_NAME.match(path.name) or any(p.lower() in {"license", "licenses", "licences"} for p in relative.parts[:-1]):
            files.add(path)
    return sorted(files)


def collect(args):
    out = args.out.resolve()
    if out.exists():
        raise ValueError("Output already exists; use a new directory")
    # Never create notices within the signed bundle or its content directories.
    if any(part.endswith(".plugin") for part in out.parts):
        raise ValueError("Notice output must remain outside any .plugin bundle")
    lock_before = (ROOT / "Cargo.lock").read_bytes()
    manifest_before = (ROOT / "Cargo.toml").read_bytes()
    raw = command("cargo", args.toolchain, "metadata", "--format-version", "1", "--locked", "--offline", "--filter-platform", args.target)
    metadata = json.loads(raw)
    own, packages = selected_packages(metadata)
    if own["version"] != args.expected_version:
        raise ValueError("Root package version differs from --expected-version")
    if Path(own["manifest_path"]).resolve() != ROOT / "Cargo.toml":
        raise ValueError("Cargo metadata root is not this checkout")
    supplements = {}
    if args.supplements:
        supplement_root = args.supplements.resolve().parent
        supplements = json.loads(args.supplements.read_text())["packages"]
        if isinstance(supplements, list):
            # The release's own third-party directory is a portable source for
            # later offline reproduction; private staging paths are unnecessary.
            supplements = {
                p["name"] + "@" + p["version"]: {
                    "selected_license": p["selected_license"], "note": p["supplement_note"],
                    "files": [{"path": item["file"], "sha256": item["sha256"], "url": item["source"]["url"]}
                              for item in p["files"] if item["source"]["kind"] == "supplemental"],
                } for p in supplements if "supplement_note" in p
            }
    entries, collected, missing = [], {}, []

    def add_file(destination, data, source):
        if not data or b"\x00" in data:
            raise ValueError("Empty or binary license text: " + destination)
        data.decode("utf-8")
        if destination in collected and collected[destination] != data:
            raise ValueError("Notice filename collision")
        collected[destination] = data
        return {"file": destination, "sha256": sha(data), "source": source}

    for package, kinds in packages:
        name, version = package["name"], package["version"]
        if not SAFE_COMPONENT.fullmatch(name) or not SAFE_COMPONENT.fullmatch(version):
            raise ValueError("Unexpected crate name/version")
        crate_root = Path(package["manifest_path"]).resolve().parent
        key, prefix = name + "@" + version, "licenses/" + name + "-" + version
        entry = {"name": name, "version": version, "license_expression": package["license"],
                 "authors_from_manifest": package["authors"], "dependency_kinds": kinds,
                 "crate_url": "https://crates.io/crates/" + name + "/" + version, "files": []}
        if not package["source"] or not package["source"].startswith("registry+"):
            raise ValueError("Review non-registry dependency separately: " + key)
        vcs_path = crate_root / ".cargo_vcs_info.json"
        if vcs_path.exists():
            vcs = json.loads(vcs_path.read_text())
            entry["upstream_commit"] = vcs.get("git", {}).get("sha1")
        checksum_path = crate_root / ".cargo-checksum.json"
        if checksum_path.exists():
            entry["registry_package_sha256"] = json.loads(checksum_path.read_text()).get("package")
        for path in license_files(crate_root, package.get("license_file")):
            relative = path.relative_to(crate_root).as_posix()
            entry["files"].append(add_file(prefix + "/" + relative, path.read_bytes(),
                {"kind": "registry-package", "crate_file": relative, "crate_url": entry["crate_url"]}))
        if key in supplements:
            supplement = supplements[key]
            entry["supplement_note"] = supplement["note"]
            entry["selected_license"] = supplement["selected_license"]
            for item in supplement["files"]:
                path = inside(supplement_root, item["path"])
                data = path.read_bytes()
                if sha(data) != item["sha256"] or not item["url"].startswith("https://"):
                    raise ValueError("Invalid supplemental license provenance: " + key)
                destination = prefix + "/supplemental/" + path.name
                entry["files"].append(add_file(destination, data,
                    {"kind": "supplemental", "url": item["url"], "sha256": item["sha256"]}))
        if not entry["files"]:
            missing.append(key)
        entries.append(entry)
    if missing:
        raise ValueError("Supply reviewed license texts for: " + ", ".join(missing))
    sysroot = Path(command("rustc", args.toolchain, "--print", "sysroot"))
    rust_version = command("rustc", args.toolchain, "-Vv")
    rust_notice = sysroot / "share/doc/rust/COPYRIGHT-library.html"
    rust_file = add_file("licenses/rust/COPYRIGHT-library.html", rust_notice.read_bytes(),
                        {"kind": "installed-rust-standard-library-notices", "rustc": rust_version})
    if lock_before != (ROOT / "Cargo.lock").read_bytes() or manifest_before != (ROOT / "Cargo.toml").read_bytes():
        raise ValueError("Cargo inputs changed during notice collection")
    manifest = {"schema": 1, "product": own["name"], "version": own["version"], "target": args.target,
                "cargo_lock_sha256": sha(lock_before), "cargo_toml_sha256": sha(manifest_before),
                "scope": "Cargo target-filtered normal and build dependency graph; dev-only edges excluded. Conservative inventory, not a proof every crate is linked.",
                "packages": entries, "rust_standard_library": rust_file}
    lines = ["DynamicFX third-party notices", "", "Version: " + own["version"], "Target: " + args.target,
             "Cargo.lock SHA-256: " + sha(lock_before), "", manifest["scope"],
             "Local registry notices are preserved verbatim. Missing texts are supplied with explicit provenance;",
             "standard license templates do not assert package-specific copyright ownership.",
             "Upstream OR expressions retain the offered alternatives; listed texts do not relicense any crate.",
             "Rust standard-library notices: licenses/rust/COPYRIGHT-library.html", "",
             "Keep this file, manifest.json and licenses/ together, outside DynamicFx.plugin.", ""]
    for entry in entries:
        lines += [entry["name"] + " " + entry["version"], "Declared license: " + str(entry["license_expression"]),
                  "Source: " + entry["crate_url"]]
        if entry["authors_from_manifest"]:
            lines.append("Package authors (Cargo metadata): " + "; ".join(entry["authors_from_manifest"]))
        if entry.get("supplement_note"):
            lines.append(entry["supplement_note"])
        lines += ["  " + item["file"] for item in entry["files"]] + [""]
    out.parent.mkdir(parents=True, exist_ok=True)
    # Prepare atomically so failed collection leaves no misleading partial bundle.
    with tempfile.TemporaryDirectory(prefix="dynamicfx-notices-", dir=out.parent) as temp:
        stage = Path(temp) / "notices"
        stage.mkdir()
        for relative, data in collected.items():
            destination = stage / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        (stage / "manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
        (stage / "THIRD_PARTY_NOTICES.txt").write_text("\n".join(lines))
        if out.exists():
            raise ValueError("Output appeared during collection")
        stage.rename(out)
    return {"version": own["version"], "target": args.target, "packages": len(entries),
            "license_files": len(collected), "cargo_lock_sha256": sha(lock_before),
            "manifest_sha256": sha((out / "manifest.json").read_bytes()), "output": str(out)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--expected-version", required=True)
    parser.add_argument("--target", default="aarch64-apple-darwin")
    parser.add_argument("--toolchain", default=os.environ.get("RUSTUP_TOOLCHAIN"))
    parser.add_argument("--supplements", type=Path)
    args = parser.parse_args()
    print(json.dumps(collect(args), indent=2))


if __name__ == "__main__":
    main()
