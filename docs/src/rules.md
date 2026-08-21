# Custom security rules

VERTOX rules are intentionally small TOML documents.

```toml
id = "example-rule"
title = "Example security pattern"
severity = "medium"
languages = ["solidity"]
pattern = 'delegatecall'
message = "Explain why this pattern deserves review."
recommendation = "Explain what the reviewer should verify."
```

Valid severities are `info`, `low`, `medium`, `high`, and `critical`. Languages are `solidity` and `vyper`.

Run custom rules with:

```bash
vertox scan ./contracts --rules-dir ./rules
```

Use `--no-builtin-rules` if you want only your own rules.
