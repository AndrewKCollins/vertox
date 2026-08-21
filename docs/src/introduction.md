# VERTOX

<p align="center"><img src="images/vertox-logo.png" width="220" alt="VERTOX logo"></p>

VERTOX is an open-source Robinhood Chain security toolkit for source review, deployed-contract inspection, and EVM reverse engineering.

Its core workflow is deliberately simple:

```text
source -> scan
address -> inspect/fetch -> reverse
bytecode -> disassemble -> CFG -> review
```

VERTOX targets developers, auditors, and researchers who want a transparent CLI rather than a hosted black box.
