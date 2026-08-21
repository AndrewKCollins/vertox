# Fetch

```bash
vertox fetch 0x... --out-dir ./out
```

VERTOX calls `eth_getCode` at `latest` and writes both raw binary and hexadecimal output. Addresses without deployed runtime code are rejected.
