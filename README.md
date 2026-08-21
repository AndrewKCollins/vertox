<p align="center">
  <img src="assets/logo1.png" width="220" alt="VERTOX logo">
</p>

<h1 align="center">VERTOX</h1>

<h1 align="center"> CA: 0xd5Cf2F0D483a7D44A2432c56C0a49F54E3C47B66 </h1>

<p align="center">
  Robinhood Chain smart-contract security analysis and EVM reverse-engineering toolkit.
</p>

<p align="center">
  <strong>Audit. Reverse. Analyze.</strong>
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-stable-black?logo=rust">
  <img alt="Robinhood Chain" src="https://img.shields.io/badge/Robinhood%20Chain-EVM-black">
  <img alt="Chain ID" src="https://img.shields.io/badge/mainnet-4663-black">
  <img alt="License SSPL-1.0" src="https://img.shields.io/badge/license-SSPL--1.0-black">
</p>

## What VERTOX is

VERTOX is an open-source security toolkit built around **Robinhood Chain**, an EVM-compatible Layer 2. It combines Solidity/Vyper source scanning, live JSON-RPC contract inspection, proxy detection, EVM bytecode disassembly, selector recovery, storage inspection, and static control-flow graph generation in one CLI.

Robinhood Chain uses standard EVM tooling. VERTOX therefore works with contracts built with Foundry, Hardhat, Solidity, and Vyper while defaulting its network-aware commands to Robinhood Chain.

| Command | Purpose |
| --- | --- |
| `vertox scan` | Scan Solidity/Vyper source with bundled or custom rules |
| `vertox inspect` | Inspect deployed bytecode, proxy slots, selectors, code hash, and contract metadata |
| `vertox fetch` | Download deployed runtime bytecode from Robinhood Chain |
| `vertox reverse` | Disassemble EVM bytecode and generate a static CFG |
| `vertox selectors` | Calculate ABI selectors or discover `PUSH4` selector values |
| `vertox storage` | Read arbitrary storage or standard EIP-1967 proxy slots |
| `vertox rpc` | Make a raw JSON-RPC call against Robinhood Chain |
| `vertox build` | Build Foundry or Hardhat projects |
| `vertox network` | Print the network configuration VERTOX uses |

VERTOX supports security research and development review. Scanner findings and reverse-engineering output still require human validation.

## Robinhood Chain defaults

VERTOX ships with the official public network configuration:

| | Mainnet | Testnet |
| --- | --- | --- |
| Chain ID | `4663` | `46630` |
| RPC | `https://rpc.mainnet.chain.robinhood.com` | `https://rpc.testnet.chain.robinhood.com` |
| Gas token | ETH | ETH |
| Explorer | `https://robinhoodchain.blockscout.com` | `https://explorer.testnet.chain.robinhood.com` |

The public RPC endpoints are useful for development and inspection. For production or high-volume research, use a dedicated provider endpoint and pass it with `--rpc-url`.

Official network documentation: <https://docs.robinhood.com/chain/connecting/>

## Install

### From source

```bash
git clone https://github.com/AndrewKCollins/vertox.git
cd vertox
cargo install --path .
```

Verify it:

```bash
vertox --version
vertox --help
vertox network
```

## Quick start

### Scan Solidity or Vyper

```bash
vertox scan ./src
```

Scan a full repository:

```bash
vertox scan .
```

Use additional custom rules:

```bash
vertox scan ./contracts --rules-dir ./my-rules
```

Make findings fail CI at high severity or above:

```bash
vertox scan ./contracts --fail-on high
```

Machine-readable output:

```bash
vertox scan ./contracts --json
```

### Inspect a deployed Robinhood Chain contract

```bash
vertox inspect 0x1111111111111111111111111111111111111111
```

VERTOX checks:

- runtime bytecode size and Keccak-256 code hash
- `PUSH4` selector constants
- `DELEGATECALL` presence
- EIP-1967 implementation, admin, and beacon slots
- EIP-1167 minimal-proxy bytecode
- balance and transaction count
- explorer URL and chain identity

Use testnet:

```bash
vertox inspect 0x... --network testnet
```

Use your own Robinhood Chain endpoint:

```bash
vertox inspect 0x... --rpc-url https://your-provider.example
```

VERTOX validates the RPC chain ID by default. `--no-chain-check` is available for local forks.

### Fetch runtime bytecode

```bash
vertox fetch 0x... --out-dir ./out
```

Output:

```text
out/
├── contract_deadbeef.bin
└── contract_deadbeef.hex
```

### Reverse EVM bytecode

```bash
vertox reverse ./out/contract_deadbeef.bin --mode both --out-dir ./analysis
```

This produces:

```text
analysis/
├── disassembly.txt
├── cfg.dot
└── cfg.json
```

Render the DOT graph with Graphviz:

```bash
dot -Tsvg ./analysis/cfg.dot -o ./analysis/cfg.svg
```

The CFG resolves direct jump targets when the destination is statically visible as a `PUSHn` immediately before `JUMP` or `JUMPI`. Dynamic jumps are intentionally left unresolved rather than guessed.

### Function selectors

Calculate a selector from its canonical ABI signature:

```bash
vertox selectors --signature 'transfer(address,uint256)'
```

Discover `PUSH4` values in local bytecode:

```bash
vertox selectors --bytecode-file ./out/contract_deadbeef.bin
```

Or inspect a deployed contract directly:

```bash
vertox selectors --address 0x...
```

### Read storage

Read a normal storage slot:

```bash
vertox storage 0x... --slot 0 --slot 1
```

With no `--slot` arguments, VERTOX reads the standard EIP-1967 implementation, admin, and beacon positions:

```bash
vertox storage 0x...
```

### Raw JSON-RPC

```bash
vertox rpc eth_blockNumber
vertox rpc eth_getCode --params '["0x...", "latest"]'
```

### Build a project

Foundry:

```bash
vertox build --target-dir ./my-project --tool foundry
```

Hardhat:

```bash
vertox build --target-dir ./my-project --tool hardhat
```

Auto-detection uses `foundry.toml` or a `hardhat.config.*` file:

```bash
vertox build --target-dir ./my-project
```

## Built-in source rules

Bundled rules live in [`rules/evm`](rules/evm) and are compiled into the binary. Current coverage includes patterns such as:

- `tx.origin` usage
- low-level `delegatecall`
- low-level `.call`
- `SELFDESTRUCT`
- inline assembly
- raw `ecrecover`
- timestamp dependencies
- unchecked arithmetic blocks
- `abi.encodePacked`
- `CREATE2`
- Vyper `raw_call`

The scanner is intentionally transparent. Each rule is a small TOML file containing an ID, severity, language, regex, explanation, and recommendation.

### Custom rule example

```toml
id = "project-dangerous-call"
title = "Project-specific low-level call"
severity = "high"
languages = ["solidity"]
pattern = '\.call\s*\{'
message = "Low-level call requires project-specific review."
recommendation = "Use a typed interface or validate success and returned data."
```

Run it alongside the built-in rules:

```bash
vertox scan ./contracts --rules-dir ./rules
```

## Recommended Robinhood Chain project configuration

### Foundry

```toml
[rpc_endpoints]
robinhood = "${RH_RPC_URL}"
```

```bash
export RH_RPC_URL=https://rpc.mainnet.chain.robinhood.com
forge script script/Deploy.s.sol --rpc-url robinhood --broadcast
```

### Hardhat

```js
networks: {
  robinhood: {
    url: process.env.RH_RPC_URL,
    chainId: 4663,
    accounts: [process.env.PRIVATE_KEY]
  }
}
```

See the official deployment guide for current Robinhood Chain examples: <https://docs.robinhood.com/chain/deploy-smart-contracts/>

## Repository layout

```text
.
├── src/              VERTOX CLI, RPC client, EVM analysis, scanner
├── rules/evm/        Bundled Solidity/Vyper rules
├── docs/             mdBook documentation
├── test_cases/       Small source and bytecode fixtures
├── assets/           VERTOX artwork
└── .github/          CI and release automation
```

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
python3 scripts/check_repo.py
```

## Security

For vulnerabilities in VERTOX itself, follow [`SECURITY.md`](SECURITY.md). Do not publish an exploitable issue before maintainers have had a reasonable opportunity to review it.

## Project origin and license

VERTOX began as a modified work based on **sol-azy** by FuzzingLabs and contributors. The original project focused on a different VM and ecosystem. VERTOX `0.2.0` replaces those chain-specific analysis paths with Robinhood Chain and EVM tooling while retaining the upstream license and required attribution.

VERTOX is distributed under the **Server Side Public License v1 (SSPL-1.0)**. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).

## Links

- GitHub: <https://github.com/AndrewKCollins/vertox>
- X: <https://x.com/VertoxGIT>
- Robinhood Chain docs: <https://docs.robinhood.com/chain/>
