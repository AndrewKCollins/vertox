# Installation

Vertexy is distributed as a Rust command-line application.

## Requirements

Install Rust and Cargo with [rustup](https://rustup.rs/).

Some commands need additional tools:

| Command | Additional requirement |
| --- | --- |
| `build` on native SBF projects | Solana CLI |
| `build` on Anchor projects | Solana CLI and Anchor CLI |
| rendering generated CFGs | Graphviz |

Verify your Rust toolchain:

```bash
rustc --version
cargo --version
```

## Build from source

```bash
git clone <your-vertexy-repository-url>
cd vertexy
cargo build --release
```

The binary will be written to:

```text
target/release/vertexy
```

Install it into Cargo's binary directory with:

```bash
cargo install --path .
```

Confirm the CLI is available:

```bash
vertexy --help
vertexy --version
```

## Run without installing

During development, prefix Vertexy arguments with `cargo run --`:

```bash
cargo run -- scan --target-dir ./my-project
```

For large scans or reverse-engineering jobs, use a release build:

```bash
cargo run --release -- scan --target-dir ./my-project
```

## Documentation

Install mdBook if you want to browse the documentation locally:

```bash
cargo install mdbook --locked
mdbook serve docs --open
```

## Troubleshooting

If `vertexy build` fails, first check that the project can be built directly with the same Solana or Anchor toolchain. Solana build tooling can select a Rust toolchain that differs from your system toolchain, so version mismatches in the target project should be resolved at the Solana or Anchor level rather than by editing lockfile formats by hand.

For reverse engineering, confirm the input is an ELF shared object produced for Solana SBF. For network failures in `fetch`, retry with an explicit RPC endpoint using `--rpc-url`.
