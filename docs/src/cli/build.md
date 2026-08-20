# `build` Command

The `build` command compiles an Anchor or native Solana SBF project and copies the resulting program `.so` files into a directory you choose.

## Usage

```bash
vertexy build \
  --target-dir ./my-project \
  --out-dir ./out
```

## Project detection

Vertexy detects the build path from the project root:

- `Anchor.toml` → `anchor build`
- `Cargo.toml` with a `solana-program` dependency → `cargo build-sbf`

## Requirements

Both project types require `cargo` and the Solana CLI. Anchor projects also require the `anchor` CLI.

Vertexy checks only the tools needed for the detected project type. A native SBF project does not require Anchor.

## Anchor version switching

If an Anchor project declares an Anchor version in `Anchor.toml`, Vertexy can switch the local Anchor CLI before building:

```bash
vertexy build \
  --target-dir ./my-anchor-project \
  --out-dir ./out \
  --unsafe-version-switch
```

This flag can reinstall a different Anchor CLI version on the machine, so use it only when that behavior is intentional.

## Output

After a successful build, Vertexy copies every `.so` file from the project's `target/deploy/` directory into `--out-dir`.

It deliberately does not copy deployment keypair files.

## Example

```bash
vertexy build \
  --target-dir test_cases/base_sbf_addition_checker \
  --out-dir ./out
```

You can then pass the copied `.so` file directly to [`vertexy reverse`](reverse.md).
