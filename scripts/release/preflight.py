#!/usr/bin/env python3
"""Read-only publication and macOS release-archive checks; never print matches.

Uses the tracked plus nonignored working tree. This is a focused heuristic,
not a replacement for reviewing images, evidence purpose, or restricted terms.
No network, signing, staging, install, commit, push, or release mutation.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import plistlib
import re
import stat
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[2]
PATTERNS = {
    "private_key": rb"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----",
    "github_token": rb"(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{60,})",
    "service_key": rb"\bsk-(?:proj-|svcacct-)?[A-Za-z0-9_-]{32,}",
    "aws_access_key": rb"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b",
    "literal_auth": rb'''(?i)(?:authorization["']?\s*[:=]\s*["'](?:bearer|basic)\s+[A-Za-z0-9+/_.=-]{16,}|(?:api[_-]?key|access[_-]?token|auth[_-]?token|password)["']?\s*[:=]\s*["'][A-Za-z0-9+/_.=-]{24,}["'])''',
    "url_credentials": rb"https?://[^\s/:@]{2,}:[^\s/@]{6,}@",
}
MACHINE_PATH = re.compile(rb"(?:/Users/[^/\s\"']+/|/private/var/folders/|/var/folders/|[A-Za-z]:\\\\?Users\\\\?[^\\\s]+)")
SENSITIVE_NAMES = {".env", ".netrc", ".npmrc", "auth-token", "credentials.json", "id_rsa", "id_ed25519"}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])


def source_version():
    text = (ROOT / "Cargo.toml").read_text()
    section = text.split("[package]", 1)[1].split("[", 1)[0]
    return re.search(r'^version\s*=\s*"([^"]+)"', section, re.M)[1]


def publication_scan(restricted_terms):
    names = sorted(set(git("ls-files", "--cached", "--others", "--exclude-standard", "-z").decode().split("\0")) - {""})
    secrets, paths, review, skipped, inventory = [], [], [], [], []
    compiled = {k: re.compile(v) for k, v in PATTERNS.items()}
    for name in names:
        path = ROOT / name
        if not path.exists():
            continue  # A tracked deletion is absent from the candidate tree.
        if path.is_symlink():
            review.append({"file": name, "reason": "symlink requires target review"})
            continue
        if not path.is_file():
            skipped.append(name)
            continue
        data = path.read_bytes()
        inventory.append({"file": name, "bytes": len(data), "sha256": sha(data)})
        kinds = [kind for kind, pattern in compiled.items() if pattern.search(data)]
        if path.name.lower() in SENSITIVE_NAMES or path.suffix.lower() in (".p12", ".pfx", ".key"):
            kinds.append("sensitive_filename")
        if restricted_terms and any(term in data.lower() for term in restricted_terms):
            kinds.append("restricted_term")
        if kinds:
            secrets.append({"file": name, "categories": kinds})
        if MACHINE_PATH.search(data):
            paths.append({"file": name, "evidence_or_harness": name.startswith(("docs/audits/evidence/", "scripts/", "spike/"))})
        if name.endswith(".bridge.json") or path.suffix == ".jsonl":
            review.append({"file": name, "reason": "transport envelope or conversation-like stream"})
        if path.suffix.lower() in (".png", ".jpg", ".jpeg", ".webp", ".psd", ".aep", ".mov", ".mp4"):
            review.append({"file": name, "reason": "visual or project content needs human review"})
    fingerprint = sha(json.dumps(inventory, sort_keys=True, separators=(",", ":")).encode())
    return {
        "files_scanned": len(inventory), "bytes_scanned": sum(x["bytes"] for x in inventory),
        "candidate_tree_fingerprint": fingerprint,
        "credential_or_restricted_findings": secrets,
        "machine_paths_requiring_purpose_review": paths,
        "manual_content_review": review, "nonfiles_skipped": skipped,
        "restricted_terms_check": "RUN" if restricted_terms else "NOT_RUN: no private terms file supplied",
        "limitations": "Only file/category names are reported. Patterns do not prove the absence of every secret; images and historical vendor-specific terms need separate review.",
    }


def archive_check(path, expected_version):
    checks = []
    def check(name, passed):
        checks.append({"check": name, "passed": bool(passed)})
    with zipfile.ZipFile(path) as archive:
        infos = archive.infolist(); names = [i.filename for i in infos]
        credential_members = []
        for info in infos:
            if info.is_dir():
                continue
            data = archive.read(info)
            kinds = [kind for kind, pattern in PATTERNS.items() if re.search(pattern, data)]
            if PurePosixPath(info.filename).name.lower() in SENSITIVE_NAMES:
                kinds.append("sensitive_filename")
            if kinds:
                credential_members.append({"file": info.filename, "categories": kinds})
        check("no recognized credentials in archive", not credential_members)
        check("unique member paths", len(names) == len(set(names)))
        safe = all(not n.startswith(("/", "\\")) and ".." not in PurePosixPath(n).parts and "\\" not in n for n in names)
        check("relative nontraversing member paths", safe)
        check("no symlink members", all(not stat.S_ISLNK(i.external_attr >> 16) for i in infos))
        check("no Finder or AppleDouble metadata", all("__MACOSX" not in PurePosixPath(n).parts and not any(p == ".DS_Store" or p.startswith("._") for p in PurePosixPath(n).parts) for n in names))
        bundles = [n[:-len("Contents/Info.plist")] for n in names if n.endswith(".plugin/Contents/Info.plist")]
        check("exactly one plugin bundle", len(bundles) == 1)
        identity = None
        if len(bundles) == 1:
            base = bundles[0]; p = plistlib.loads(archive.read(base + "Contents/Info.plist"))
            check("both plist versions match release", p.get("CFBundleVersion") == p.get("CFBundleShortVersionString") == expected_version)
            required = ["Contents/MacOS/DynamicFx", "Contents/Resources/DynamicFx.rsrc", "Contents/Resources/packaging-context.json", "Contents/PkgInfo", "Contents/_CodeSignature/CodeResources"]
            check("complete bundle including signature and context", all(base + n in names for n in required))
            executable = base + "Contents/MacOS/DynamicFx"
            if executable in names:
                binary = archive.read(executable)
                check("Mach-O arm64 header", len(binary) >= 8 and binary[:8] == b"\xcf\xfa\xed\xfe\x0c\x00\x00\x01")
                check("executable permission preserved", archive.getinfo(executable).external_attr >> 16 & 0o111)
                identity = {"binary_sha256": sha(binary), "binary_bytes": len(binary), "version": p.get("CFBundleShortVersionString")}
        docs = [PurePosixPath(n).name.lower() for n in names if ".plugin/" not in n]
        check("installation instructions outside bundle", any(n in ("install.txt", "install.md") for n in docs))
        check("MIT license outside bundle", any(n in ("license", "license.txt", "license.md") for n in docs))
        sums = [n for n in names if PurePosixPath(n).name == "SHA256SUMS"]
        check("one SHA256SUMS file", len(sums) == 1)
        if len(sums) == 1:
            parent = str(PurePosixPath(sums[0]).parent)
            parent = "" if parent == "." else parent + "/"
            rows = archive.read(sums[0]).decode().splitlines(); valid = bool(rows); covered = set()
            for row in rows:
                m = re.fullmatch(r"([0-9a-fA-F]{64}) [ *](.+)", row)
                if not m:
                    valid = False; continue
                member = parent + m[2]
                if member not in names or member.endswith("/"):
                    valid = False; continue
                valid &= sha(archive.read(member)) == m[1].lower(); covered.add(member)
            check("SHA256SUMS entries match archived bytes", valid)
            if len(bundles) == 1:
                check("checksum covers executable and PiPL", all(bundles[0] + n in covered for n in ["Contents/MacOS/DynamicFx", "Contents/Resources/DynamicFx.rsrc"]))
    return {"archive": path.name, "archive_sha256": sha(path.read_bytes()), "archive_bytes": path.stat().st_size,
            "checks": checks, "identity": identity, "credential_members": credential_members,
            "passed": all(x["passed"] for x in checks),
            "not_checked": "This byte check does not validate codesign, notarization, Gatekeeper, host behavior or correspondence to a source commit. Extract to a private temporary directory, verify with scripts/macos.py, then compare to the host-tested identity."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected-version", required=True)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--restricted-terms-file", type=Path, help="Private newline-separated terms; values never appear in output")
    args = parser.parse_args()
    terms = [x.strip().lower() for x in args.restricted_terms_file.read_bytes().splitlines() if x.strip()] if args.restricted_terms_file else []
    publication = publication_scan(terms)
    version = source_version()
    result = {"baseline_commit": git("rev-parse", "HEAD").decode().strip(), "working_tree": True,
              "expected_version": args.expected_version, "source_version": version, "version_matches": version == args.expected_version,
              "publication": publication, "archive": archive_check(args.archive, args.expected_version) if args.archive else None}
    result["automated_checks_passed"] = (result["version_matches"] and not publication["credential_or_restricted_findings"]
                                         and (result["archive"] is None or result["archive"]["passed"]))
    result["authorization_not_evaluated_by_script"] = True
    result["authorization_note"] = "Static preflight checks artifact contents; it does not evaluate user authorization."
    print(json.dumps(result, indent=2))
    return 0 if result["automated_checks_passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
