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

# `--fix` rewrites the version samples (check 2) and nothing else.
#
# That check is the only one whose source of truth moves without a human
# touching the docs: release-plz bumps `version` in Cargo.toml when it opens a
# release PR, which strands the `scuv <version>` samples in prose and fails the
# Lint job on every release. The other checks answer to code a person just
# edited -- MSRV, RESERVED_NAMES, translation keys, MIN_VERSION -- so a stale
# doc there is a real omission in that PR and must stay a human decision.
FIX = "--fix" in sys.argv[1:]
fixed = []

# Two of the four files carry the sample inside the mdBook (installation.md and
# api.md), so rewriting them changes the gettext msgids and strands ko.po --
# which docs.yml then rejects. The release automation would fix one gate by
# breaking the other, every release. The samples are version strings, not
# prose, so the fix is mechanical and stays here rather than pulling mdbook and
# msgmerge into the release job.
PO_FILES = ("docs/po/ko.po",)


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
# `scuv 0.15.3` covers `--version` output samples; `scuv Version: 0.15.3`
# covers api.md's footer stamp, which the narrower pattern silently skipped —
# the file was checked, matched nothing, and passed while stale.
SAMPLE = re.compile(r"scuv (?:Version:\**\s*)?(\d+\.\d+\.\d+)")
for rel in ("README.md", "CLAUDE.md", "docs/src/installation.md", "docs/src/api.md"):
    if FIX:
        body = read(rel)
        # Rewrite only the captured version, keeping whatever prefix matched
        # (`scuv ` or api.md's `scuv Version:** `) exactly as it was.
        patched = SAMPLE.sub(
            lambda m: m.group(0)[: m.start(1) - m.start(0)] + version, body
        )
        if patched != body:
            (ROOT / rel).write_text(patched, encoding="utf-8")
            fixed.append(rel)
    found = [m.group(1) for m in SAMPLE.finditer(read(rel))]
    stale = sorted({v for v in found if v != version})
    check(
        f"{rel} version samples say {version}",
        found and not stale,
        f"stale: {', '.join(stale)} -- Cargo.toml is {version}"
        if stale
        else "no version sample found; the pattern no longer matches this file",
    )

if FIX:
    # Entry-scoped, not a blanket substitution. A `.po` entry is a blank-line
    # separated block; only blocks whose msgid carries a `scuv <version>`
    # sample are touched, and inside those the bare version string is replaced
    # wherever it appears. That last part matters: api.md's msgid reads
    # `**scuv Version:** 0.15.4` while its msgstr reads `**scuv 버전:** 0.15.4`,
    # so a SAMPLE-regex substitution would update the English side and leave
    # the Korean page advertising the previous release.
    #
    # Obsolete entries (`#~`) are left alone -- they describe text that no
    # longer exists in the source, so rewriting them would invent history.
    for rel in PO_FILES:
        body = read(rel)
        blocks = body.split("\n\n")
        touched = False
        for i, block in enumerate(blocks):
            if block.startswith("#~"):
                continue
            msgid_versions = {
                m.group(1)
                for line in block.splitlines()
                if line.startswith(("msgid", '"'))
                for m in SAMPLE.finditer(line)
            }
            for old_version in msgid_versions - {version}:
                blocks[i] = blocks[i].replace(old_version, version)
                touched = True
        if touched:
            (ROOT / rel).write_text("\n\n".join(blocks), encoding="utf-8")
            fixed.append(rel)

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
for rel in (
    "llms-full.txt",
    "CONTRIBUTING.md",
    "docs/src/development/translation.md",
    "docs/src/development/docs-translation.md",
):
    body = read(rel)
    wrong = [n for n in re.findall(r"(\d+) keys", body) if int(n) != keys]
    check(
        f"{rel} states {keys} keys",
        not wrong,
        f"stale counts: {', '.join(sorted(set(wrong)))} -- app.yml has {keys}",
    )

# --- 5. Supported languages: src/i18n.rs is the source of truth ----------
# Every doc that enumerates the locales must name each code in SUPPORTED_LANGS.
# The Spanish locale (#190) landed in code with all of these still saying
# "(en, ko, ja, pt-BR)" and nothing failed.
langs_block = re.search(r"SUPPORTED_LANGS[^=]*=\s*&\[(.*?)\];", read("src/i18n.rs"), re.S)
lang_codes = re.findall(r'\("([A-Za-z-]+)",\s*"', langs_block.group(1)) if langs_block else []
check("src/i18n.rs SUPPORTED_LANGS parsed", len(lang_codes) >= 2, "could not parse SUPPORTED_LANGS")
for rel in (
    "README.md",
    "CLAUDE.md",
    "llms.txt",
    "llms-full.txt",
    "context7.json",
    "docs/src/commands/lang.md",
    "docs/src/commands/README.md",
    "docs/src/llms.md",
    "docs/src/development/architecture.md",
    "docs/src/development/contributing.md",
    "docs/src/development/translation.md",
):
    body = read(rel)

    def has(code, text):
        return re.search(rf"(?<![\w-]){re.escape(code)}(?![\w-])", text) is not None

    missing = [c for c in lang_codes if not has(c, body)]
    # A line that already enumerates the list (it names both ko and pt-BR)
    # must name every code -- one stale "(en, ko, ja, pt-BR)" in a file that
    # mentions the new code elsewhere would otherwise slip through.
    stale = [
        n
        for n, line in enumerate(body.splitlines(), 1)
        if has("ko", line) and has("pt-BR", line) and any(not has(c, line) for c in lang_codes)
    ]
    check(
        f"{rel} lists every supported language",
        not missing and not stale,
        f"missing: {', '.join(missing) or '-'}; stale enumeration on line(s): "
        f"{', '.join(map(str, stale)) or '-'} -- SUPPORTED_LANGS has {', '.join(lang_codes)}",
    )

# --- 6. uv floor: src/uv/version.rs is the source of truth ----------------
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

if fixed:
    print(f"\nRewrote version samples to {version} in: {', '.join(fixed)}")

if failures:
    print("\n".join(["", "Stale references found:"] + [f" - {f}" for f in failures]))
    if not FIX:
        print("\nVersion samples alone can be fixed with: "
              "python3 scripts/check-doc-references.py --fix")
    sys.exit(1)
print("\nAll reference docs are in sync with the code.")
