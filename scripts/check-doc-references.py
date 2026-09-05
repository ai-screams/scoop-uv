#!/usr/bin/env python3
"""Verify hand-copied facts in the reference docs still match the code.

`llms.txt`, `llms-full.txt` and `docs/src/llms.md` are three hand-maintained
copies of the same reference material, so a fact updated in one routinely
goes stale in the others. Every check below compares a doc against the code
that owns the fact -- never a doc against another doc.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
failures = []


def read(rel):
    return (ROOT / rel).read_text(encoding="utf-8")


def check(label, ok, detail=""):
    print(f"{'✅' if ok else '❌'} {label}")
    if not ok:
        failures.append(f"{label}\n   {detail}" if detail else label)


# --- 1. MSRV: Cargo.toml is the source of truth -------------------------
msrv = re.search(r'^rust-version = "(.+?)"', read("Cargo.toml"), re.M).group(1)
check(
    f"llms-full.txt states MSRV {msrv}",
    f"MSRV {msrv}" in read("llms-full.txt"),
    f"expected 'MSRV {msrv}'; update llms-full.txt",
)

# --- 2. Version samples: Cargo.toml is the source of truth ---------------
version = re.search(r'^version = "(.+?)"', read("Cargo.toml"), re.M).group(1)
for rel in ("README.md", "docs/src/installation.md", "docs/src/api.md"):
    stale = [
        m.group(0)
        for m in re.finditer(r"scuv (\d+\.\d+\.\d+)", read(rel))
        if m.group(1) != version
    ]
    check(
        f"{rel} version samples say {version}",
        not stale,
        f"stale: {', '.join(sorted(set(stale)))} -- Cargo.toml is {version}",
    )

# --- 3. Reserved names: src/validate.rs is the source of truth -----------
names = re.findall(
    r'"([a-z]+)"',
    re.search(
        r"const RESERVED_NAMES: &\[&str\] = &\[(.*?)\n\];", read("src/validate.rs"), re.S
    ).group(1),
)
expected = ", ".join(names)
for rel in ("llms.txt", "docs/src/llms.md"):
    check(
        f"{rel} lists all {len(names)} reserved names",
        f"Reserved words: {expected}" in read(rel),
        f"expected: Reserved words: {expected}",
    )

# --- 4. Translation keys: locales/app.yml is the source of truth ---------
keys = len(re.findall(r"^[a-z_][a-zA-Z0-9_.]*:$", read("locales/app.yml"), re.M))
for rel in ("llms-full.txt", "CONTRIBUTING.md", "docs/src/development/translation.md"):
    body = read(rel)
    wrong = [n for n in re.findall(r"(\d+) keys", body) if int(n) != keys]
    check(
        f"{rel} states {keys} keys",
        not wrong,
        f"stale counts: {', '.join(sorted(set(wrong)))} -- app.yml has {keys}",
    )

# --- 5. uv floor: src/uv/version.rs is the source of truth ----------------
# The floor named 0.5.14 for months while the code required 0.5.19, so
# `scuv doctor` passed installations that then failed on the first command
# that lists Python versions. Keep every copy pointing at the constant.
#
# Scan whole lines, not a regex window around "uv": the qualifying word
# ("Minimum", ">=", "or newer") often sits before the version, and an
# earlier version of this check matched too narrowly to notice.
floor = ".".join(
    re.search(
        r"MIN_VERSION: \(u32, u32, u32\) = \((\d+), (\d+), (\d+)\)", read("src/uv/version.rs")
    ).groups()
)
FLOOR_LINE = re.compile(r"\buv\b", re.I)
QUALIFIER = re.compile(r"minimum|>=|or newer", re.I)
for rel in ("README.md", "CLAUDE.md", "docs/src/installation.md"):
    stale = []
    for line in read(rel).splitlines():
        if not (FLOOR_LINE.search(line) and QUALIFIER.search(line)):
            continue
        stale += [v for v in re.findall(r"\b(\d+\.\d+\.\d+)\b", line) if v != floor]
    check(
        f"{rel} states uv floor {floor}",
        not stale,
        f"stale: {', '.join(sorted(set(stale)))} -- MIN_VERSION is {floor}",
    )

if failures:
    print("\n".join(["", "Stale references found:"] + [f" - {f}" for f in failures]))
    sys.exit(1)
print("\nAll reference docs are in sync with the code.")
