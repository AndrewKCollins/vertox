# Publishing Vertexy on GitHub

The repository is designed to be pushed directly to a new GitHub repository named `vertexy`.

## First public push

1. Create an empty GitHub repository named `vertexy` with `main` as the default branch.
2. Push this repository without replacing `LICENSE` or `NOTICE`.
3. In **Settings → Pages**, choose **GitHub Actions** as the Pages source.
4. In **Settings → Security**, enable private vulnerability reporting if it is available for the repository.
5. Add useful repository topics such as `solana`, `security`, `rust`, `sast`, `sbpf`, `reverse-engineering`, and `starlark`.
6. Add a concise GitHub description, for example: `Solana program security analysis and sBPF reverse-engineering toolkit.`

## Before the first release

Run locally:

```bash
python3 scripts/check_repo.py
cargo check --all-targets
cargo test --all-targets
```

The live Solana RPC tests are ignored by default. Run them explicitly when you want integration coverage:

```bash
cargo test -- --ignored
```

Because Vertexy is a binary application, commit the generated `Cargo.lock` after the first successful Cargo build.

## Create a release

Update `CHANGELOG.md`, commit the version bump, then create and push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds binaries and attaches them to the GitHub release created for the tag.

## Attribution

Vertexy is a modified work based on sol-azy by FuzzingLabs and contributors. Keep `LICENSE`, `NOTICE`, and the project-origin section of the README in public distributions.
