# Security model

VERTOX is an analysis aid, not an oracle.

- Regex source rules can produce false positives and false negatives.
- `PUSH4` values are not guaranteed to be function selectors.
- Proxy storage can be intentionally non-standard.
- Static CFG recovery cannot resolve every dynamic jump.
- Runtime bytecode may differ from verified source.
- A clean VERTOX report does not prove a contract is secure.

Use the output to guide manual review and combine it with tests, compiler output, verified source, protocol context, and deeper analysis tools where appropriate.
