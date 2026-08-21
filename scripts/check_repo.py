#!/usr/bin/env python3
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
errors = []

required = [
    ROOT / "Cargo.toml",
    ROOT / "README.md",
    ROOT / "LICENSE",
    ROOT / "NOTICE",
    ROOT / "assets" / "logo1.png",
    ROOT / "rules" / "evm",
]
for path in required:
    if not path.exists():
        errors.append(f"missing required path: {path.relative_to(ROOT)}")

cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
if cargo["package"]["name"] != "vertox":
    errors.append("Cargo package must be named vertox")
if "Robinhood Chain" not in cargo["package"]["description"]:
    errors.append("Cargo description should identify Robinhood Chain")

network = (ROOT / "src" / "network.rs").read_text()
for expected in ["4663", "46630", "rpc.mainnet.chain.robinhood.com", "rpc.testnet.chain.robinhood.com"]:
    if expected not in network:
        errors.append(f"network.rs missing {expected}")

for rule_path in sorted((ROOT / "rules" / "evm").glob("*.toml")):
    try:
        rule = tomllib.loads(rule_path.read_text())
    except Exception as exc:
        errors.append(f"invalid TOML {rule_path.relative_to(ROOT)}: {exc}")
        continue
    for key in ["id", "title", "severity", "languages", "pattern", "message", "recommendation"]:
        if key not in rule:
            errors.append(f"{rule_path.relative_to(ROOT)} missing {key}")
    try:
        re.compile(rule.get("pattern", ""))
    except re.error as exc:
        errors.append(f"invalid regex {rule_path.relative_to(ROOT)}: {exc}")

# Stale runtime/product wording should not survive the chain conversion.
# Legal attribution in NOTICE/README is intentionally excluded.
scan_paths = [ROOT / "src", ROOT / "docs", ROOT / "rules", ROOT / "test_cases"]
stale = [
    (re.compile(r"\bAnchor\b", re.I), "Anchor"),
    (re.compile(r"\bsBPF\b", re.I), "sBPF"),
    (re.compile(r"solana-program", re.I), "solana-program"),
]
for base in scan_paths:
    for path in base.rglob("*"):
        if not path.is_file() or path.suffix.lower() in {".png", ".jpg", ".jpeg", ".gif", ".ico", ".svg"}:
            continue
        text = path.read_text(errors="ignore")
        for pattern, label in stale:
            if pattern.search(text):
                errors.append(f"stale {label} reference: {path.relative_to(ROOT)}")

if errors:
    print("Repository checks failed:")
    for error in errors:
        print(f" - {error}")
    sys.exit(1)

print("VERTOX repository checks passed.")
