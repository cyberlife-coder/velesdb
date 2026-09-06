#!/usr/bin/env python3
"""Fail the build when a removal deferred to a major version has been skipped.

# Why this exists

`WalBatchConfig` was to be removed "at the next major". The next major shipped
two days after that was written — `v6.0.0`, 2026-09-02 — and did not remove it.
Nothing was broken and nobody was careless: the constraint lived in a doc
comment and in an issue (#2174), and neither is read by the thing that bumps a
version.

A deferred removal is a promise with a due date. This is the alarm clock. When
the workspace major reaches the version a removal was promised for, every site
listed below must be gone, or CI refuses the release that bumped it.

# Why it fails the bump rather than warning before it

Warning "the next major is near" needs someone to be listening at the right
moment, which is the failure mode that produced this file. Failing *on* the bump
needs nobody: the release commit that raises the major cannot go green until the
removal lands. The check is only as good as its list, and the list is short on
purpose — an entry is added when a removal is deliberately postponed, which is
rare and always a decision someone wrote down.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

#: Removals promised for a future major, and every site that must be gone.
#:
#: A removal that leaves one site behind is not done, which is why sites are
#: listed individually rather than as a single symbol: `WalBatchConfig` also has
#: a field, an `ENGINE_SECTIONS` entry and an `ENV_SECTIONS` entry, and #2174
#: records that the last of those was missed by the issue text itself.
DEFERRED_REMOVALS: "list[dict]" = [
    {
        "what": "WalBatchConfig and its [wal_batch] config table",
        "remove_at_major": 7,
        "issue": 2174,
        "sites": [
            ("crates/velesdb-core/src/config.rs", "pub struct WalBatchConfig"),
            ("crates/velesdb-core/src/config.rs", "pub wal_batch:"),
            ("crates/velesdb-core/src/config.rs", '"wal_batch"'),
            # The compiler catches the Rust sites; it says nothing about a guide
            # that keeps documenting a table users can no longer set.
            ("docs/guides/CONFIGURATION.md", "wal_batch"),
            ("docs/CORE_WIRING_DEBT.md", "wal_batch"),
        ],
        # `docs/guides/MIGRATION_v6.0.0.md` is deliberately NOT a site. It
        # records that the table stayed in 6.0.0, which remains true after 7.0.0
        # removes it — a historical note is not stale merely because history
        # moved on, and forcing its deletion would falsify the record.
    },
]

#: Anchored inside `[workspace.package]`, and matching any version string
#: rather than a strict triple.
#:
#: Both details are borrowed from `check-version-sync.py`, which already carries
#: the scar: an unanchored search reads the FIRST line-initial `version =` in the
#: file, which a long-form `[workspace.dependencies.x]` table above the package
#: section would win. And `"7.0.0-rc.1"` is a version this repository ships —
#: `create-release-tag.sh` accepts a prerelease suffix — so a pattern demanding
#: a closing quote after the patch number would refuse to read a real manifest
#: and report it as a missing version.
_PACKAGE_SECTION = "[workspace.package]"
_VERSION_LINE = re.compile(r'^version\s*=\s*"([^"]+)"', re.MULTILINE)


def workspace_major(root: Path) -> int:
    """The major version `[workspace.package]` declares.

    Raises `RuntimeError` rather than guessing: a guard that cannot read the
    version it gates on must say so, not pass.
    """
    manifest = root / "Cargo.toml"
    try:
        text = manifest.read_text(encoding="utf-8")
    except OSError as exc:
        raise RuntimeError(f"cannot read {manifest}: {exc}") from exc
    section_at = text.find(_PACKAGE_SECTION)
    if section_at == -1:
        raise RuntimeError(f"no {_PACKAGE_SECTION} section in {manifest}")
    match = _VERSION_LINE.search(text, section_at)
    if match is None:
        raise RuntimeError(f"no version under {_PACKAGE_SECTION} in {manifest}")
    major, _, _ = match.group(1).partition(".")
    try:
        return int(major)
    except ValueError as exc:
        raise RuntimeError(f"unreadable major in version {match.group(1)!r}") from exc


def validate_entry(entry: "dict") -> None:
    """Refuses a malformed entry with `RuntimeError`, never a bare `TypeError`.

    The module promises that an unreadable tree answers 2 and never 1. An entry
    missing a key or carrying a string where an int belongs would raise out of
    `run` uncaught, and Python exits 1 on an uncaught exception — a refusal this
    guard never made, wearing the exit code of one it did.
    """
    for key in ("what", "remove_at_major", "issue", "sites"):
        if key not in entry:
            raise RuntimeError(f"deferred-removal entry is missing {key!r}: {entry}")
    if not isinstance(entry["remove_at_major"], int):
        raise RuntimeError(
            f"remove_at_major must be an int, got {entry['remove_at_major']!r}"
        )
    if not entry["sites"]:
        raise RuntimeError(
            f"deferred-removal entry {entry['what']!r} lists no sites, so it "
            "would pass the day its major arrives while the code is untouched"
        )


def surviving_sites(root: Path, entry: "dict") -> "list[str]":
    """Sites from `entry` that are still present in the tree."""
    survivors = []
    for relative, needle in entry["sites"]:
        path = root / relative
        try:
            text = path.read_text(encoding="utf-8")
        except FileNotFoundError:
            # The file itself is gone, so the site is too.
            continue
        except OSError as exc:
            raise RuntimeError(f"cannot read {path}: {exc}") from exc
        if needle in text:
            survivors.append(f"{relative}: {needle}")
    return survivors


def run(root: Path) -> int:
    major = workspace_major(root)
    overdue = []
    for entry in DEFERRED_REMOVALS:
        validate_entry(entry)
        if major < entry["remove_at_major"]:
            continue
        survivors = surviving_sites(root, entry)
        if survivors:
            overdue.append((entry, survivors))

    if not overdue:
        pending = [e for e in DEFERRED_REMOVALS if major < e["remove_at_major"]]
        print(
            f"PASSED: workspace major is {major}; "
            f"{len(pending)} deferred removal(s) not yet due, none overdue."
        )
        return 0

    print(
        f"FAILED: workspace major is {major} and "
        f"{len(overdue)} deferred removal(s) are overdue:",
        file=sys.stderr,
    )
    for entry, survivors in overdue:
        print(
            f"  - {entry['what']} was promised for {entry['remove_at_major']}.0.0 "
            f"(#{entry['issue']}), and these sites are still here:",
            file=sys.stderr,
        )
        for site in survivors:
            print(f"      {site}", file=sys.stderr)
    print(
        "\nRemove them, or move the entry's `remove_at_major` and say in the issue "
        "why the promise slipped. Editing this list silently is the failure this "
        "guard exists to stop.",
        file=sys.stderr,
    )
    return 1


def main(argv: "list[str] | None" = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", default=str(REPO_ROOT), help="repository root to scan")
    args = parser.parse_args(argv)
    # A tree this guard cannot read answers 2, never 1: a refusal it never made
    # must not look like a refusal.
    try:
        return run(Path(args.root).resolve())
    except (OSError, RuntimeError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
