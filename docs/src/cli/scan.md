# `scan` Command

The `scan` command performs source-level security analysis on Solana Rust projects using Vertexy's Starlark rule engine.

## Basic usage

Run the bundled rules against one Anchor or native SBF project:

```bash
vertexy scan --target-dir ./my-project
```

Add custom rules from a directory:

```bash
vertexy scan \
  --target-dir ./my-project \
  --rules-dir ./rules
```

Use only your external rules:

```bash
vertexy scan \
  --target-dir ./my-project \
  --rules-dir ./rules \
  --no-internal-rules
```

To discover and scan multiple Solana projects beneath a directory, opt into recursion:

```bash
vertexy scan \
  --target-dir ./workspace \
  --recursive
```

## How it works

The scanner:

1. detects Anchor or native SBF project roots;
2. parses Rust source with `syn`;
3. enriches the syntax tree with source positions;
4. prepares the tree for the Starlark matching helpers;
5. applies bundled and optional external rules;
6. prints findings with rule metadata and source locations.

By default, `scan` analyzes only the project given by `--target-dir`. This avoids duplicate findings when an Anchor workspace contains nested program crates.

## Custom rules

Start with the rule documentation:

- [Rule Format](../rules/format.md)
- [Writing Templates](../rules/templates.md)
- [Detection Example](../rules/example.md)

The `ast` command is useful when you need to inspect the exact AST structure a rule will receive.

## Related

- [AST command](ast.md)
- [SAST engine architecture](../architecture/sast_engine.md)
