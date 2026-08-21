# Storage

```bash
vertox storage 0x... --slot 0 --slot 1
```

Slots may be decimal or `0x`-prefixed hexadecimal.

With no explicit slots, VERTOX reads the EIP-1967 implementation, admin, and beacon positions:

```bash
vertox storage 0x...
```
