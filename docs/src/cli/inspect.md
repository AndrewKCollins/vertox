# Inspect

```bash
vertox inspect 0x...
vertox inspect 0x... --network testnet
vertox inspect 0x... --json
```

Inspection includes runtime bytecode size, Keccak-256 code hash, selector constants, `DELEGATECALL` presence, EIP-1967 storage, EIP-1167 minimal-proxy detection, balance, transaction count, and explorer URL.

Use `--rpc-url` for a provider endpoint. VERTOX checks the chain ID by default.
