# Contributing to VERTOX

VERTOX welcomes focused contributions to Robinhood Chain and EVM security analysis.

## Development setup

Install stable Rust, then run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
python3 scripts/check_repo.py
```

Network-dependent commands should not be required for unit tests. Keep tests deterministic and use local fixtures where possible.

## Source rules

Bundled rules live under `rules/evm/`. A rule should identify a security-sensitive pattern that benefits from human review. Avoid presenting regex matches as proof of exploitation.

Each rule must include:

```toml
id = "unique-id"
title = "Short title"
severity = "medium"
languages = ["solidity"]
pattern = 'regex'
message = "Why the match matters."
recommendation = "What should be reviewed or changed."
```

Supported languages are `solidity` and `vyper`.

## Pull requests

Keep changes scoped. Explain the security or developer problem being solved, add tests for analysis logic, update docs when CLI behavior changes, and do not include private keys, API keys, production secrets, or live exploit material.
