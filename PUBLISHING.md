# Publishing VERTOX

## Before the first Robinhood Chain release

1. Run `cargo fmt --check`.
2. Run `cargo clippy --all-targets --all-features -- -D warnings`.
3. Run `cargo test --all-targets`.
4. Run `python3 scripts/check_repo.py`.
5. Test `vertox network` and one testnet `vertox inspect` call.
6. Confirm README and docs match the current Robinhood Chain network configuration.
7. Commit `Cargo.lock` after the first successful dependency resolution.

## Release

Tag releases as `vMAJOR.MINOR.PATCH`, for example:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The release workflow builds platform archives and SHA-256 checksums.

The crate currently has `publish = false` because GitHub releases are the primary distribution path.
