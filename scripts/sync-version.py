#!/usr/bin/env python3
"""Synchronize or verify Nala's version across all manifests.

Usage:
    python3 scripts/sync-version.py              # check that all manifests agree
    python3 scripts/sync-version.py 0.6.0        # set version across all manifests
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def replace_once(path: Path, pattern: str, replacement: str, flags: int = 0) -> None:
    text = path.read_text(encoding="utf-8")
    new_text, count = re.subn(pattern, replacement, text, count=1, flags=flags)
    if count != 1:
        raise RuntimeError(f"Expected 1 match in {path.relative_to(ROOT)}, found {count}")
    path.write_text(new_text, encoding="utf-8")


def set_version(version: str) -> None:
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError(f"Version must be X.Y.Z, got {version!r}")

    # 1. windows/package.json
    pkg_path = ROOT / "windows/package.json"
    pkg = json.loads(pkg_path.read_text(encoding="utf-8"))
    pkg["version"] = version
    pkg_path.write_text(json.dumps(pkg, indent=2) + "\n", encoding="utf-8")

    # 2. windows/package-lock.json
    lock_path = ROOT / "windows/package-lock.json"
    replace_once(
        lock_path,
        r'("name":\s*"(?:aira-windows|nala-windows)",\s*"version":\s*")[0-9.]+(")',
        rf"\g<1>{version}\2",
    )
    replace_once(
        lock_path,
        r'("":\s*\{\s*"name":\s*"(?:aira-windows|nala-windows)",\s*"version":\s*")[0-9.]+(")',
        rf"\g<1>{version}\2",
    )

    # 3. windows/src-tauri/tauri.conf.json
    tauri_path = ROOT / "windows/src-tauri/tauri.conf.json"
    replace_once(
        tauri_path,
        r'("productName":\s*"(?:Aira|Nala)",\s*"version":\s*")[0-9.]+(")',
        rf"\g<1>{version}\2",
    )

    # 4. windows/Cargo.toml
    cargo_toml = ROOT / "windows/Cargo.toml"
    replace_once(
        cargo_toml,
        r'(\[workspace\.package\]\s*\r?\nversion\s*=\s*")[0-9.]+(")',
        rf"\g<1>{version}\2",
    )

    # 5. windows/Cargo.lock (workspace crates: aira/nala and aira-hook/nala-hook)
    cargo_lock = ROOT / "windows/Cargo.lock"
    for crate_name in ("aira", "aira-hook", "nala", "nala-hook"):
        try:
            replace_once(
                cargo_lock,
                rf'(name\s*=\s*"{crate_name}"\s*\r?\nversion\s*=\s*")[0-9.]+(")',
                rf"\g<1>{version}\2",
            )
        except Exception:
            pass

    # 6. linux/arch/PKGBUILD
    pkgbuild = ROOT / "linux/arch/PKGBUILD"
    replace_once(pkgbuild, r"(^pkgver=)[0-9.]+", rf"\g<1>{version}", flags=re.MULTILINE)

    # 7. NotchBuddy/project.yml (first target: NotchBuddy GitHub build)
    project_yml = ROOT / "NotchBuddy/project.yml"
    replace_once(
        project_yml,
        r'(CFBundleShortVersionString:\s*")[0-9.]+(")',
        rf"\g<1>{version}\2",
    )

    # 8. NotchBuddy/Resources/Info.plist
    info_plist = ROOT / "NotchBuddy/Resources/Info.plist"
    replace_once(
        info_plist,
        r"(<key>CFBundleShortVersionString</key>\s*<string>)[0-9.]+(</string>)",
        rf"\g<1>{version}\2",
    )

    # 9. windows/src/whats-new/whats-new.json
    win_whats_new = ROOT / "windows/src/whats-new/whats-new.json"
    replace_once(
        win_whats_new,
        r'("releases":\s*\[\s*\{\s*"version":\s*")[0-9.]+(")',
        rf"\g<1>{version}\2",
    )

    # 10. NotchBuddy/Resources/whats-new.json
    mac_whats_new = ROOT / "NotchBuddy/Resources/whats-new.json"
    replace_once(
        mac_whats_new,
        r'("releases":\s*\[\s*\{\s*"version":\s*")[0-9.]+(")',
        rf"\g<1>{version}\2",
    )

    # 11. CHANGELOG.md (first release header)
    changelog = ROOT / "CHANGELOG.md"
    replace_once(
        changelog,
        r"(^## \[?)[0-9.]+(\]? - \d{4}-\d{2}-\d{2})",
        rf"\g<1>{version}\2",
        flags=re.MULTILINE,
    )

    print(f"Updated all manifests to version {version}")


def read_versions() -> dict[str, str]:
    pkg = json.loads((ROOT / "windows/package.json").read_text(encoding="utf-8"))["version"]
    pkg_lock_data = json.loads((ROOT / "windows/package-lock.json").read_text(encoding="utf-8"))
    pkg_lock = pkg_lock_data["version"]
    pkg_lock_root = pkg_lock_data.get("packages", {}).get("", {}).get("version", "")
    tauri = json.loads((ROOT / "windows/src-tauri/tauri.conf.json").read_text(encoding="utf-8"))["version"]
    cargo_toml = re.search(
        r'^version\s*=\s*"([^"]+)"',
        (ROOT / "windows/Cargo.toml").read_text(encoding="utf-8"),
        re.MULTILINE,
    ).group(1)
    cargo_lock_text = (ROOT / "windows/Cargo.lock").read_text(encoding="utf-8")
    m_main = re.search(r'name\s*=\s*"(?:aira|nala)"\s*\r?\nversion\s*=\s*"([^"]+)"', cargo_lock_text)
    cargo_lock_main = m_main.group(1) if m_main else ""
    m_hook = re.search(r'name\s*=\s*"(?:aira-hook|nala-hook)"\s*\r?\nversion\s*=\s*"([^"]+)"', cargo_lock_text)
    cargo_lock_hook = m_hook.group(1) if m_hook else ""
    pkgbuild = re.search(
        r"^pkgver=([0-9.]+)",
        (ROOT / "linux/arch/PKGBUILD").read_text(encoding="utf-8"),
        re.MULTILINE,
    ).group(1)
    mac_proj = re.search(
        r'CFBundleShortVersionString:\s*"([0-9.]+)"',
        (ROOT / "NotchBuddy/project.yml").read_text(encoding="utf-8"),
    ).group(1)
    mac_plist = re.search(
        r"<key>CFBundleShortVersionString</key>\s*<string>([0-9.]+)</string>",
        (ROOT / "NotchBuddy/Resources/Info.plist").read_text(encoding="utf-8"),
    ).group(1)
    win_whats_new = json.loads(
        (ROOT / "windows/src/whats-new/whats-new.json").read_text(encoding="utf-8")
    )["releases"][0]["version"]
    mac_whats_new = json.loads(
        (ROOT / "NotchBuddy/Resources/whats-new.json").read_text(encoding="utf-8")
    )["releases"][0]["version"]
    changelog_m = re.search(
        r"^## \[?(\d+\.\d+\.\d+)\]?",
        (ROOT / "CHANGELOG.md").read_text(encoding="utf-8"),
        re.MULTILINE,
    )
    changelog = changelog_m.group(1) if changelog_m else ""

    return {
        "windows/package.json": pkg,
        "windows/package-lock.json": pkg_lock,
        "windows/package-lock.json (root)": pkg_lock_root,
        "windows/src-tauri/tauri.conf.json": tauri,
        "windows/Cargo.toml": cargo_toml,
        "windows/Cargo.lock (app)": cargo_lock_main,
        "windows/Cargo.lock (hook)": cargo_lock_hook,
        "linux/arch/PKGBUILD": pkgbuild,
        "NotchBuddy/project.yml": mac_proj,
        "NotchBuddy/Resources/Info.plist": mac_plist,
        "windows/src/whats-new/whats-new.json": win_whats_new,
        "NotchBuddy/Resources/whats-new.json": mac_whats_new,
        "CHANGELOG.md": changelog,
    }


def main() -> None:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    if len(sys.argv) > 1:
        set_version(sys.argv[1])
    versions = read_versions()
    unique = set(versions.values())
    for label, v in versions.items():
        print(f"  {label:38s} -> {v}")
    if len(unique) != 1:
        print(f"\n✗ Version mismatch detected: {sorted(unique)}", file=sys.stderr)
        sys.exit(1)
    print(f"\n✓ All 13 version targets are synchronized at {next(iter(unique))}")


if __name__ == "__main__":
    main()
