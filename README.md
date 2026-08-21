<p align="center">
  <img src="assets/logo.png" width="220" alt="VERTOX logo">
</p>

<h1 align="center">VERTOX</h1>

<h1 align="center"> $VERTOX - CA: TBA </h1>

<p align="center">
  Solana program security analysis and sBPF reverse-engineering toolkit.
</p>

<p align="center">
  <strong>Scan source. Review Anchor constraints. Fetch deployed programs. Reverse sBPF.</strong>
</p>

<p align="center">
  <img alt="Rust 2021" src="https://img.shields.io/badge/Rust-2021-black?logo=rust">
  <img alt="Solana" src="https://img.shields.io/badge/Solana-sBPF-black">
  <img alt="License SSPL-1.0" src="https://img.shields.io/badge/license-SSPL--1.0-black">
</p>

## What VERTOX does

VERTOX is a command-line toolkit for Solana developers, auditors, and researchers. It combines source-level static analysis with tooling for inspecting compiled and deployed programs.

| Capability | What it gives you |
| --- | --- |
| `VERTOX scan` | AST-based security checks using bundled or custom Starlark rules |
| `VERTOX recap` | Audit-friendly Anchor instruction and account-constraint summaries |
| `VERTOX fetch` | Program bytecode or raw account data from a Solana RPC endpoint |
| `VERTOX reverse` | sBPF disassembly, immediate-value tracking, and control-flow graphs |
| `VERTOX dotting` | Manual enrichment of reduced Graphviz control-flow graphs |
| `VERTOX ast` | JSON AST output for writing and debugging custom rules |
| `VERTOX build` | Build helpers for Anchor and native SBF projects |

VERTOX is intended to support security review. It does not prove that a program is secure, and findings still require human validation.

## Install

### Requirements

The core CLI requires Rust and Cargo. Some commands have extra requirements:

- `build`: Solana CLI and, for Anchor projects, Anchor CLI
- Graph visualization: Graphviz is useful for rendering generated `.dot` files

### From source

Clone this repository, then install the binary from its root:

```bash
cd VERTOX
cargo install --path .
```

Then verify the installation:

```bash
VERTOX --help
VERTOX --version
```

For local development without installing:

```bash
cargo run -- --help
```

## Quick start

### Scan a Solana project

Use VERTOX's bundled rules:

```bash
VERTOX scan --target-dir ./my-solana-project
```

Add your own Starlark rules:

```bash
VERTOX scan \
  --target-dir ./my-solana-project \
  --rules-dir ./rules
```

Use only external rules:

```bash
VERTOX scan \
  --target-dir ./my-solana-project \
  --rules-dir ./rules \
  --no-internal-rules
```

### Review an Anchor project

```bash
VERTOX recap --target-dir ./my-anchor-project
```

The recap extracts instruction-level audit context including signers, writable accounts, constraints, PDA seeds, and memory allocation hints.

### Fetch a deployed program or account

```bash
VERTOX fetch \
  --program-id <PROGRAM_ID> \
  --out-dir ./out
```

Use another RPC endpoint when needed:

```bash
VERTOX fetch \
  --program-id <PROGRAM_ID> \
  --out-dir ./out \
  --rpc-url https://your-rpc.example
```

For executable accounts, VERTOX writes `fetched_program.so`. Upgradeable programs are resolved through their ProgramData account and trimmed to the ELF header. Non-executable accounts are written unchanged as `fetched_account.bin`.

### Reverse engineer an sBPF binary

```bash
VERTOX reverse \
  --mode both \
  --bytecodes-file ./program.so \
  --out-dir ./out \
  --labeling \
  --reduced
```

Depending on the selected mode, VERTOX can generate:

- readable disassembly
- immediate-data information
- a full or reduced Graphviz CFG
- labels for known Solana syscalls and discovered functions

Render a generated graph with Graphviz:

```bash
dot -Tsvg ./out/cfg.dot -o ./out/cfg.svg
```

### Inspect the AST used by rules

```bash
VERTOX ast --file-path ./programs/demo/src/lib.rs --starlark-syn-ast
```

This is useful when developing custom Starlark detections.

## Built-in security rules

The bundled rule set currently includes checks for patterns such as:

- missing signer checks
- missing owner checks
- arbitrary CPI
- account reinitialization
- duplicate mutable accounts
- PDA sharing
- type cosplay
- unvalidated sysvar accounts
- missing bump seed canonicalization
- account-data reallocation
- risky account closing patterns
- saturating arithmetic usage
- `unwrap()` on checked arithmetic

Readable rule sources live under [`rules/syn_ast`](rules/syn_ast). The copies embedded into the compiled binary live under [`src/static/starlark_rules`](src/static/starlark_rules).

## Custom Starlark rules

VERTOX's source scanner is designed to be extended without recompiling the CLI. A custom rule can match prepared Rust syntax-tree data and emit a finding with its own metadata and severity.

Start with:

- [`docs/src/rules/format.md`](docs/src/rules/format.md)
- [`docs/src/rules/templates.md`](docs/src/rules/templates.md)
- [`docs/src/rules/example.md`](docs/src/rules/example.md)
- [`rules/syn_ast`](rules/syn_ast)

## Commands

```text
VERTOX build     Build an Anchor or native SBF project
VERTOX scan      Run source-level security analysis
VERTOX recap     Summarize an Anchor project's audit surface
VERTOX fetch     Fetch deployed program or account data
VERTOX reverse   Disassemble sBPF and generate CFGs
VERTOX dotting   Reinsert selected functions into a reduced CFG
VERTOX ast       Print Rust AST data for rule development
```

Legacy command aliases `sast`, `fetcher`, and `ast-utils` remain available for users migrating from the upstream project.

Use `-v`, `-vv`, or `-vvv` for additional logs. `RUST_LOG` can still be used for explicit filtering.

## Documentation

The complete mdBook source is in [`docs/`](docs/).

Run it locally with:

```bash
cargo install mdbook
mdbook serve docs --open
```

## Repository layout

```text
.
├── src/                  Rust CLI and analysis engines
├── rules/syn_ast/        Human-editable Starlark security rules
├── docs/                 mdBook documentation
├── test_cases/           Small Solana and sBPF fixtures
├── assets/               Project artwork
└── .github/              CI, release automation, and community templates
```

## Development

```bash
cargo check
cargo test
cargo fmt --check
```

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the development workflow and rule-contribution guidance.

## Security

Please do not publish a potentially exploitable issue in VERTOX itself before maintainers have had a chance to review it. See [`SECURITY.md`](SECURITY.md).

## Project origin and license

VERTOX is a modified work based on **sol-azy**, originally developed by FuzzingLabs and contributors. The upstream project is available at `https://github.com/FuzzingLabs/sol-azy`.

The project remains licensed under the **Server Side Public License v1 (SSPL-1.0)**. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE) for details about the upstream work and VERTOX modifications.

## Contributing

Bug reports, rule improvements, documentation fixes, and analysis-engine contributions are welcome. Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) before opening a pull request.
