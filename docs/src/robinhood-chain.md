# Robinhood Chain

VERTOX defaults to Robinhood Chain mainnet and supports the official testnet with `--network testnet`.

| | Mainnet | Testnet |
| --- | --- | --- |
| Chain ID | 4663 | 46630 |
| RPC | `https://rpc.mainnet.chain.robinhood.com` | `https://rpc.testnet.chain.robinhood.com` |
| Gas token | ETH | ETH |
| Explorer | `https://robinhoodchain.blockscout.com` | `https://explorer.testnet.chain.robinhood.com` |

Robinhood Chain is EVM compatible, so Solidity, Vyper, Foundry, Hardhat, ethers.js, viem, and standard Ethereum JSON-RPC methods work normally.

VERTOX verifies `eth_chainId` before network-dependent analysis unless `--no-chain-check` is supplied. This protects against accidentally inspecting the same `0x` address on the wrong chain.

Official documentation: <https://docs.robinhood.com/chain/connecting/>
