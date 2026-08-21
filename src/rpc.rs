use crate::network::Network;
use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Clone)]
pub struct RpcClient {
    url: String,
    http: Client,
}

impl RpcClient {
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent(concat!("vertox/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self {
            url: url.into(),
            http,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let response = self
            .http
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("RPC request failed: {method}"))?;

        let status = response.status();
        let value: Value = response
            .json()
            .await
            .with_context(|| format!("RPC returned non-JSON response for {method}"))?;

        if !status.is_success() {
            bail!("RPC HTTP error {status}: {value}");
        }

        if let Some(error) = value.get("error") {
            bail!("RPC error for {method}: {error}");
        }

        value
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow!("RPC response for {method} did not contain result"))
    }

    pub async fn chain_id(&self) -> Result<u64> {
        let value = self.call("eth_chainId", json!([])).await?;
        let raw = value
            .as_str()
            .ok_or_else(|| anyhow!("eth_chainId returned a non-string value"))?;
        parse_hex_u64(raw).context("invalid eth_chainId result")
    }

    pub async fn ensure_network(&self, network: Network) -> Result<u64> {
        let actual = self.chain_id().await?;
        let expected = network.chain_id();
        if actual != expected {
            bail!(
                "RPC chain mismatch: expected {} ({expected}), got chain ID {actual} from {}",
                network.display_name(),
                self.url
            );
        }
        Ok(actual)
    }

    pub async fn get_code(&self, address: &str) -> Result<Vec<u8>> {
        validate_address(address)?;
        let value = self
            .call("eth_getCode", json!([address, "latest"]))
            .await?;
        let raw = value
            .as_str()
            .ok_or_else(|| anyhow!("eth_getCode returned a non-string value"))?;
        decode_hex_data(raw).context("invalid contract bytecode")
    }

    pub async fn get_balance(&self, address: &str) -> Result<String> {
        validate_address(address)?;
        let value = self
            .call("eth_getBalance", json!([address, "latest"]))
            .await?;
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("eth_getBalance returned a non-string value"))
    }

    pub async fn get_transaction_count(&self, address: &str) -> Result<String> {
        validate_address(address)?;
        let value = self
            .call("eth_getTransactionCount", json!([address, "latest"]))
            .await?;
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("eth_getTransactionCount returned a non-string value"))
    }

    pub async fn get_storage_at(&self, address: &str, slot: &str) -> Result<String> {
        validate_address(address)?;
        let value = self
            .call("eth_getStorageAt", json!([address, slot, "latest"]))
            .await?;
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("eth_getStorageAt returned a non-string value"))
    }
}

pub fn validate_address(address: &str) -> Result<()> {
    if address.len() != 42 || !address.starts_with("0x") {
        bail!("invalid EVM address: expected 0x followed by 40 hex characters");
    }
    if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid EVM address: contains non-hex characters");
    }
    Ok(())
}

pub fn decode_hex_data(raw: &str) -> Result<Vec<u8>> {
    let stripped = raw.strip_prefix("0x").unwrap_or(raw);
    if stripped.is_empty() {
        return Ok(Vec::new());
    }
    if stripped.len() % 2 != 0 {
        bail!("hex data has odd length");
    }
    hex::decode(stripped).context("failed to decode hex data")
}

pub fn parse_hex_u64(raw: &str) -> Result<u64> {
    let stripped = raw.strip_prefix("0x").unwrap_or(raw);
    u64::from_str_radix(stripped, 16).context("failed to parse hexadecimal integer")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_evm_address() {
        assert!(validate_address("0x1111111111111111111111111111111111111111").is_ok());
    }

    #[test]
    fn rejects_bad_evm_address() {
        assert!(validate_address("1111").is_err());
        assert!(validate_address("0xzz11111111111111111111111111111111111111").is_err());
    }

    #[test]
    fn decodes_empty_code() {
        assert_eq!(decode_hex_data("0x").unwrap(), Vec::<u8>::new());
    }
}
