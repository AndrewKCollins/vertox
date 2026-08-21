# Build

Auto-detect a project:

```bash
vertox build -d .
```

Force Foundry or Hardhat:

```bash
vertox build -d . --tool foundry
vertox build -d . --tool hardhat
```

Use `--out-dir` to copy the resulting `out/` or `artifacts/` tree to another directory after a successful build.
