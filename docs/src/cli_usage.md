# CLI usage

VERTOX groups its functionality into focused subcommands. Run `vertox <command> --help` for the complete option list.

When developing from a clone, replace `vertox` with `cargo run --` in the examples below.

## Commands

### [`build`](cli/build.md)

Build an Anchor or native SBF project for analysis.

```bash
vertox build --target-dir ./my-project --out-dir ./out
```

### [`recap`](cli/recap.md)

Generate an audit-oriented summary of an Anchor project.

```bash
vertox recap --target-dir ./my-anchor-project
```

### [`scan`](cli/scan.md)

Run source-level security analysis with bundled and optional custom Starlark rules.

```bash
vertox scan --target-dir ./my-project
```

With custom rules:

```bash
vertox scan --target-dir ./my-project --rules-dir ./rules
```

### [`fetch`](cli/fetch.md)

Fetch an on-chain program or account through Solana RPC.

```bash
vertox fetch --program-id <PROGRAM_ID> --out-dir ./out
```

### [`reverse`](cli/reverse.md)

Disassemble a compiled program and generate control-flow information.

```bash
vertox reverse \
  --mode both \
  --bytecodes-file ./program.so \
  --out-dir ./out \
  --labeling
```

### [`dotting`](reverse/dotting.md)

Reinsert selected functions into a reduced Graphviz CFG.

```bash
vertox dotting \
  --config functions.json \
  --reduced-dot-path cfg_reduced.dot \
  --full-dot-path cfg.dot
```

### [`ast`](cli/ast.md)

Print Rust AST data for custom-rule development.

```bash
vertox ast --file-path ./src/lib.rs --starlark-syn-ast
```

## Logging

Use `-v`, `-vv`, or `-vvv` for progressively more detail:

```bash
vertox -vv scan --target-dir ./my-project
```

You can also set `RUST_LOG` directly.

## Compatibility aliases

VERTOX keeps these aliases for users migrating from the upstream sol-azy CLI:

- `sast` → `scan`
- `fetcher` → `fetch`
- `ast-utils` → `ast`
