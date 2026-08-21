# Foundry and Hardhat

Robinhood Chain accepts standard EVM deployments.

## Foundry

```toml
[rpc_endpoints]
robinhood = "${RH_RPC_URL}"
```

```bash
export RH_RPC_URL=https://rpc.mainnet.chain.robinhood.com
forge build
```

## Hardhat

```js
networks: {
  robinhood: {
    url: process.env.RH_RPC_URL,
    chainId: 4663,
    accounts: [process.env.PRIVATE_KEY]
  }
}
```

Never commit private keys or provider credentials.
