# Scan

```bash
vertox scan ./contracts
vertox scan ./contracts --json
vertox scan ./contracts --fail-on high
vertox scan ./contracts --rules-dir ./custom-rules
```

VERTOX scans `.sol` and `.vy` files. Common dependency/build directories such as `node_modules`, `lib`, `out`, `artifacts`, `cache`, and `target` are skipped.

A match means “review this location,” not “this contract is exploitable.”
