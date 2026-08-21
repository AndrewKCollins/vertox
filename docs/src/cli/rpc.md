# RPC

Use VERTOX as a thin Robinhood Chain JSON-RPC client:

```bash
vertox rpc eth_blockNumber
vertox rpc eth_getCode --params '["0x...", "latest"]'
```

`--params` must contain a JSON array.
