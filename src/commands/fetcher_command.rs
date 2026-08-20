use crate::fetcher::fetch_to;
use anyhow::Context;
use log::debug;
use std::path::Path;

/// Fetch a Solana program or account and write it into the requested output directory.
pub async fn run(
    account: String,
    out_dir: String,
    rpc_url: Option<String>,
) -> anyhow::Result<()> {
    let out_path = Path::new(&out_dir);
    if !out_path.is_dir() {
        std::fs::create_dir_all(out_path)
            .with_context(|| format!("failed to create output directory '{out_dir}'"))?;
        debug!("Created output directory '{out_dir}'.");
    }

    fetch_to(&out_dir, rpc_url, &account)
        .await
        .with_context(|| format!("failed to fetch Solana account '{account}'"))?;

    Ok(())
}
