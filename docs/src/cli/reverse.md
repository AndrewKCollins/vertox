# Reverse

```bash
vertox reverse ./contract.bin --mode both --out-dir ./analysis
```

Modes are `disass`, `cfg`, and `both`.

The disassembler understands the standard EVM opcode families including PUSH, DUP, SWAP, LOG, PUSH0, transient storage opcodes, and common contract lifecycle opcodes.

The CFG is conservative. Static jump destinations are linked when a `PUSHn` immediately precedes `JUMP` or `JUMPI`. Dynamic jump destinations are not guessed.
