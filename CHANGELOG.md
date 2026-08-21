# Changelog

All notable changes to VERTOX are documented here.

## 0.2.0 - 2026-08-21

### Robinhood Chain conversion

- Rebuilt VERTOX around Robinhood Chain and standard EVM JSON-RPC.
- Added mainnet chain ID `4663` and testnet chain ID `46630` defaults.
- Added Solidity and Vyper source scanning with bundled TOML rules.
- Added custom rule loading and CI severity thresholds.
- Added deployed-contract bytecode fetching with `.bin` and `.hex` output.
- Added contract intelligence with code hashes, selector discovery, `DELEGATECALL` detection, and explorer links.
- Added EIP-1967 implementation/admin/beacon inspection.
- Added EIP-1167 minimal-proxy detection.
- Added EVM disassembly and static control-flow graph generation.
- Added function selector calculation and `PUSH4` discovery.
- Added arbitrary storage reads and raw JSON-RPC access.
- Added Foundry and Hardhat build integration.
- Replaced the previous chain-specific rules, fixtures, documentation, and architecture.
- Kept the VERTOX name, monochrome identity, and existing logo artwork unchanged.

## 0.1.0 - 2026-08-20

- Initial VERTOX public repository based on the upstream sol-azy project.
