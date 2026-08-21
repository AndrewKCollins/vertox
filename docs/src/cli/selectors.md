# Selectors

Calculate the first four bytes of Keccak-256 over a canonical ABI signature:

```bash
vertox selectors -s 'transfer(address,uint256)'
```

Discover `PUSH4` values in bytecode:

```bash
vertox selectors --bytecode-file ./contract.bin
vertox selectors --address 0x...
```

A `PUSH4` constant is often a function selector but may also be unrelated data. Treat discovery as reverse-engineering evidence, not ABI recovery proof.
