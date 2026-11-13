#!/usr/bin/env python3
"""What's new after an update: every release must bring its entry.

Coucou shows, once after it updated itself, what the new version brings (an icon, a title and a
line per item, in every language the app speaks). The entries live in
  NotchBuddy/Resources/whats-new.json   (Mac, the 16 languages of Localizable.xcstrings)
  windows/src/whats-new/whats-new.json  (Windows and Linux, the languages of strings.json)
This test fails when the version the app is about to ship has no entry, or an entry is
incomplete. release.sh runs it too, so a release can't go out without its "What's new".

    python3 scripts/test-whats-new.py
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The icons both apps know (WhatsNew.swift maps them to SF Symbols, src/whats-new/icons.ts to SVG).
ICONS = {
    "update", "languages", "terminal", "checklist", "bell", "chat", "mic", "phone", "shield",
    "sparkles", "bolt", "puzzle", "paint", "eye", "keyboard", "heart", "clock", "download", "bug",
}

MAC_LANGS = ["en", "fr", "es", "pt-BR", "ru", "zh-Hans", "hi", "ar", "bn", "id", "de", "tr", "th", "zh-Hant", "ko", "ja"]

MAX_ITEMS = 4
MAX_TITLE = 48     # characters, any language: one line in the card
MAX_DETAIL = 110   # characters: two short lines at most

failures = []


def fail(msg):
    failures.append(msg)
    print(f"  ✗ {msg}")


def mac_version():
    text = (ROOT / "NotchBuddy/project.yml").read_text(encoding="utf-8")
    # The first target is the GitHub app (NotchBuddy); the App Store build has its own numbers.
    m = re.search(r'CFBundleShortVersionString:\s*"([0-9.]+)"', text)
    return m.group(1)


def windows_version():
    return json.loads((ROOT / "windows/package.json").read_text(encoding="utf-8"))["version"]


def check_mac_manifests(version):
    plist_text = (ROOT / "NotchBuddy/Resources/Info.plist").read_text(encoding="utf-8")
    m = re.search(r"<key>CFBundleShortVersionString</key>\s*<string>([0-9.]+)</string>", plist_text)
    plist_v = m.group(1) if m else ""
    if plist_v != version:
        fail(f"NotchBuddy/Resources/Info.plist says {plist_v!r}, expected {version!r}")
    changelog_text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    if not re.search(rf"^## \[?{re.escape(version)}\]?", changelog_text, re.MULTILINE):
        fail(f"CHANGELOG.md has no section for {version}")


def check_windows_manifests(version):
    tauri_v = json.loads((ROOT / "windows/src-tauri/tauri.conf.json").read_text(encoding="utf-8"))["version"]
    if tauri_v != version:
        fail(f"windows/src-tauri/tauri.conf.json says {tauri_v!r}, expected {version!r}")

    lock_data = json.loads((ROOT / "windows/package-lock.json").read_text(encoding="utf-8"))
    if lock_data.get("version") != version or lock_data.get("packages", {}).get("", {}).get("version") != version:
        fail(f"windows/package-lock.json does not match {version!r}")

    cargo_toml_text = (ROOT / "windows/Cargo.toml").read_text(encoding="utf-8")
    m_cargo = re.search(r'^version\s*=\s*"([^"]+)"', cargo_toml_text, re.MULTILINE)
    cargo_v = m_cargo.group(1) if m_cargo else ""
    if cargo_v != version:
        fail(f"windows/Cargo.toml says {cargo_v!r}, expected {version!r}")

    cargo_lock_text = (ROOT / "windows/Cargo.lock").read_text(encoding="utf-8")
    for crate_name in ("aira", "aira-hook", "nala", "nala-hook"):
        m_lock = re.search(rf'name\s*=\s*"{crate_name}"\s*\r?\nversion\s*=\s*"([^"]+)"', cargo_lock_text)
        if m_lock:
            lock_v = m_lock.group(1)
            if lock_v != version:
                fail(f"windows/Cargo.lock ({crate_name}) says {lock_v!r}, expected {version!r}")

    pkgbuild_text = (ROOT / "linux/arch/PKGBUILD").read_text(encoding="utf-8")
    m_pkg = re.search(r"^pkgver=([0-9.]+)", pkgbuild_text, re.MULTILINE)
    pkg_v = m_pkg.group(1) if m_pkg else ""
    if pkg_v != version:
        fail(f"linux/arch/PKGBUILD says {pkg_v!r}, expected {version!r}")

    changelog_text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    if not re.search(rf"^## \[?{re.escape(version)}\]?", changelog_text, re.MULTILINE):
        fail(f"CHANGELOG.md has no section for {version}")


def windows_langs():
    return json.loads((ROOT / "windows/src/i18n/strings.json").read_text(encoding="utf-8"))["languages"]


def vkey(v):
    return tuple(int(x) for x in v.split("."))


def check(label, path, version, langs):
    print(f"\n{label}: {path.relative_to(ROOT)} for {version}")
    before = len(failures)
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as e:  # noqa: BLE001
        fail(f"{path.name} does not read as JSON: {e}")
        return
    releases = data.get("releases")
    if not isinstance(releases, list) or not releases:
        fail("no releases")
        return
    versions = [r.get("version", "") for r in releases]
    if len(set(versions)) != len(versions):
        fail(f"a version appears twice: {versions}")
    if any(not re.fullmatch(r"\d+(\.\d+){1,3}", v) for v in versions):
        fail(f"a version is not numbers and dots: {versions}")
        return
    if versions != sorted(versions, key=vkey, reverse=True):
        fail(f"newest first, please: {versions}")
    if versions and versions[0] != version:
        fail(f"top entry in {path.relative_to(ROOT)} is {versions[0]}, expected shipping version {version}")
    current = next((r for r in releases if r["version"] == version), None)
    if current is None:
        fail(f"no entry for {version}: add it to {path.relative_to(ROOT)} (see CLAUDE.md, Releases)")
        return
    for r in releases:
        items = r.get("items")
        where = r["version"]
        if not isinstance(items, list) or not (1 <= len(items) <= MAX_ITEMS):
            fail(f"{where}: 1 to {MAX_ITEMS} items")
            continue
        for i, item in enumerate(items, 1):
            if item.get("icon") not in ICONS:
                fail(f"{where} item {i}: icon {item.get('icon')!r} is not one of {sorted(ICONS)}")
            for field, limit in (("title", MAX_TITLE), ("detail", MAX_DETAIL)):
                values = item.get(field)
                if not isinstance(values, dict):
                    fail(f"{where} item {i}: no {field}")
                    continue
                # Older entries may miss a language added later (English is shown instead);
                # the version being shipped must have them all.
                needed = langs if r is current else ["en"]
                for lang in needed:
                    v = values.get(lang)
                    if not isinstance(v, str) or not v.strip():
                        fail(f"{where} item {i}: {field} has no {lang}")
                    elif len(v) > limit:
                        fail(f"{where} item {i}: {field} in {lang} is {len(v)} characters (at most {limit})")
                for lang in values:
                    if lang not in langs:
                        fail(f"{where} item {i}: {field} has {lang!r}, which the app does not speak")
    if len(failures) == before:
        print(f"  ✓ {version}: {len(current['items'])} items in {len(langs)} languages")


def main():
    # Windows runners print in cp1252 by default: ✓ and the translations need UTF-8.
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    only = sys.argv[1] if len(sys.argv) > 1 else None
    if only in (None, "mac"):
        mv = mac_version()
        check_mac_manifests(mv)
        check("Mac", ROOT / "NotchBuddy/Resources/whats-new.json", mv, MAC_LANGS)
    if only in (None, "windows"):
        wv = windows_version()
        check_windows_manifests(wv)
        check("Windows and Linux", ROOT / "windows/src/whats-new/whats-new.json", wv, windows_langs())
    if failures:
        print(f"\n{len(failures)} failure(s)")
        sys.exit(1)
    print("\nWhat's new is ready")


if __name__ == "__main__":
    main()
