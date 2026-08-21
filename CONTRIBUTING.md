# Contributing to VERTOX

Thanks for helping improve VERTOX. Contributions are useful when they are focused, testable, and clear about the security assumption being changed.

## Before you start

For larger changes, open an issue first and describe the problem, expected behavior, and proposed approach. Small fixes can go directly to a pull request.

Please use `SECURITY.md` instead of a public issue for vulnerabilities in VERTOX itself.

## Development setup

Install Rust using rustup, then clone the repository and run:

```bash
cargo check
cargo test
```

For changes to the documentation:

```bash
cargo install mdbook
mdbook serve docs
```

## Pull requests

Before opening a pull request:

```bash
cargo fmt --check
cargo check
cargo test
```

Keep changes scoped. If a change modifies CLI behavior, update the README and the relevant page under `docs/src/cli/`.

If Cargo generates or updates `Cargo.lock`, include it in the pull request. VERTOX is an application, so the lockfile should be committed for reproducible builds.

## Adding a security rule

Source-analysis rules are Starlark files under `rules/syn_ast/`. Bundled runtime copies are embedded from `src/static/starlark_rules/syn_ast/`.

A rule contribution should include:

1. a narrow description of the unsafe pattern;
2. metadata that explains the finding clearly;
3. at least one positive example that should trigger;
4. at least one negative example that should not trigger;
5. documentation when the rule introduces a new matching pattern or helper.

Avoid broad patterns that produce large numbers of findings without actionable context.

## Reverse-engineering changes

Changes to disassembly, syscall labeling, immediate tracking, or CFG generation should be exercised against fixtures under `test_cases/`. If output changes intentionally, include a short before-and-after example in the pull request.

## Style

Prefer straightforward Rust, explicit errors, and small functions. Do not silently discard errors that should affect the CLI exit status.

## License

By contributing, you agree that your contribution will be distributed under the repository's SSPL-1.0 license.
