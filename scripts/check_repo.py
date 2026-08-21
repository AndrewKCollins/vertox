#!/usr/bin/env python3
"""Lightweight repository consistency checks for VERTOX."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def check_rule_mirrors(errors: list[str]) -> None:
    public = ROOT / "rules" / "syn_ast"
    embedded = ROOT / "src" / "static" / "starlark_rules" / "syn_ast"

    public_files = {p.name: p for p in public.glob("*.star")}
    embedded_files = {p.name: p for p in embedded.glob("*.star")}

    if public_files.keys() != embedded_files.keys():
        missing_embedded = sorted(public_files.keys() - embedded_files.keys())
        missing_public = sorted(embedded_files.keys() - public_files.keys())
        if missing_embedded:
            errors.append(f"rules missing from embedded copy: {', '.join(missing_embedded)}")
        if missing_public:
            errors.append(f"embedded rules missing from public copy: {', '.join(missing_public)}")

    for name in sorted(public_files.keys() & embedded_files.keys()):
        if public_files[name].read_bytes() != embedded_files[name].read_bytes():
            errors.append(f"rule copies differ: {name}")


def check_markdown_links(errors: list[str]) -> None:
    markdown_files = [ROOT / "README.md", *sorted((ROOT / "docs" / "src").rglob("*.md"))]
    link_pattern = re.compile(r"(?<!!)\[[^\]]*\]\(([^)]+)\)")
    scheme_pattern = re.compile(r"^[a-zA-Z][a-zA-Z0-9+.-]*:")

    for path in markdown_files:
        text = path.read_text(encoding="utf-8")
        for match in link_pattern.finditer(text):
            raw_target = match.group(1).strip()
            target = raw_target.split("#", 1)[0].strip()
            if not target or scheme_pattern.match(target):
                continue
            target = target.split()[0]
            resolved = (path.parent / target).resolve()
            if not resolved.exists():
                errors.append(f"broken relative Markdown link in {path.relative_to(ROOT)}: {raw_target}")


def check_branding(errors: list[str]) -> None:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    main = (ROOT / "src" / "main.rs").read_text(encoding="utf-8")
    if 'name = "vertox"' not in cargo:
        errors.append("Cargo package/binary is not named vertox")
    if 'name = "vertox"' not in main:
        errors.append("Clap application is not named vertox")


def main() -> int:
    errors: list[str] = []
    check_rule_mirrors(errors)
    check_markdown_links(errors)
    check_branding(errors)

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print("VERTOX repository checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
