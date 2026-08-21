//! VERTOX CLI entry point.
//!
//! VERTOX is a Robinhood Chain security toolkit for Solidity/Vyper source scanning,
//! deployed-contract inspection, EVM bytecode analysis, and reverse engineering.

mod evm;
mod network;
mod project;
mod rpc;
mod scanner;

use anyhow::{bail, Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use evm::{
    build_cfg, cfg_to_dot, detect_eip1167, disassemble, discover_push4_selectors, eip1967_slot,
    function_selector, keccak256, normalize_storage_slot, read_bytecode_file,
    storage_word_to_address,
};
use network::Network;
use project::BuildTool;
use rpc::RpcClient;
use scanner::{scan_project, should_fail, Severity};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing_subscriber::fmt;

#[derive(Parser)]
#[command(
    name = "vertox",
    version,
    about = "Robinhood Chain smart-contract security analysis and EVM reverse-engineering toolkit",
    long_about = "VERTOX helps Robinhood Chain developers and security researchers scan Solidity/Vyper source, fetch and inspect deployed contracts, analyze EVM bytecode, recover function selectors, inspect storage, and generate control-flow graphs.",
    after_help = "Examples:\n  vertox scan ./src\n  vertox inspect 0x...\n  vertox fetch 0x... --out-dir ./out\n  vertox reverse ./out/contract.bin --mode both --out-dir ./analysis\n  vertox selectors --address 0x...\n  vertox storage 0x... --slot 0"
)]
struct Cli {
    /// Increase log verbosity. Repeat for trace-level logs.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan Solidity and Vyper source for security-sensitive patterns.
    Scan {
        /// File or directory to scan.
        target: PathBuf,

        /// Directory containing custom TOML rules.
        #[arg(long)]
        rules_dir: Option<PathBuf>,

        /// Disable bundled VERTOX rules.
        #[arg(long, default_value_t = false)]
        no_builtin_rules: bool,

        /// Output machine-readable JSON.
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Exit non-zero when this severity or higher is found.
        #[arg(long, value_name = "SEVERITY")]
        fail_on: Option<String>,
    },

    /// Build a Foundry or Hardhat project.
    Build {
        /// Project root.
        #[arg(short = 'd', long = "target-dir", default_value = ".")]
        target_dir: PathBuf,

        /// Build system to use.
        #[arg(long, value_enum, default_value = "auto")]
        tool: BuildTool,

        /// Optional directory to copy build artifacts into.
        #[arg(short = 'o', long = "out-dir")]
        out_dir: Option<PathBuf>,
    },

    /// Fetch deployed contract bytecode from Robinhood Chain.
    Fetch {
        /// Contract address.
        address: String,

        /// Directory for .bin and .hex output.
        #[arg(short = 'o', long = "out-dir", default_value = "./out")]
        out_dir: PathBuf,

        #[command(flatten)]
        rpc: RpcArgs,
    },

    /// Inspect a deployed contract, proxy slots, selectors, and bytecode identity.
    Inspect {
        /// Contract address.
        address: String,

        /// Output machine-readable JSON.
        #[arg(long, default_value_t = false)]
        json: bool,

        #[command(flatten)]
        rpc: RpcArgs,
    },

    /// Disassemble EVM bytecode and/or generate a static CFG.
    Reverse {
        /// Raw binary or 0x-prefixed/plain hexadecimal bytecode file.
        bytecode_file: PathBuf,

        /// Analysis mode.
        #[arg(long, value_enum, default_value = "both")]
        mode: ReverseMode,

        /// Output directory.
        #[arg(short = 'o', long = "out-dir", default_value = "./analysis")]
        out_dir: PathBuf,
    },

    /// Calculate function selectors or discover PUSH4 selectors in bytecode.
    Selectors {
        /// Function signature, for example transfer(address,uint256). Repeat as needed.
        #[arg(short = 's', long = "signature")]
        signatures: Vec<String>,

        /// Read selectors from a local bytecode file.
        #[arg(short = 'b', long = "bytecode-file")]
        bytecode_file: Option<PathBuf>,

        /// Fetch bytecode from this deployed contract before discovery.
        #[arg(short = 'a', long = "address")]
        address: Option<String>,

        #[command(flatten)]
        rpc: RpcArgs,
    },

    /// Read contract storage slots using eth_getStorageAt.
    Storage {
        /// Contract address.
        address: String,

        /// Storage slot in decimal or 0x-prefixed hex. Repeat as needed.
        /// If omitted, VERTOX reads the EIP-1967 implementation, admin, and beacon slots.
        #[arg(long = "slot")]
        slots: Vec<String>,

        #[command(flatten)]
        rpc: RpcArgs,
    },

    /// Make a raw JSON-RPC call against Robinhood Chain.
    Rpc {
        /// JSON-RPC method, for example eth_blockNumber.
        method: String,

        /// JSON array of parameters.
        #[arg(long, default_value = "[]")]
        params: String,

        #[command(flatten)]
        rpc: RpcArgs,
    },

    /// Print Robinhood Chain network configuration used by VERTOX.
    Network {
        #[arg(long, value_enum, default_value = "mainnet")]
        network: Network,
    },
}

#[derive(Clone, Args)]
struct RpcArgs {
    /// Robinhood Chain network.
    #[arg(long, value_enum, default_value = "mainnet")]
    network: Network,

    /// Override the default RPC endpoint.
    #[arg(long)]
    rpc_url: Option<String>,

    /// Skip Robinhood Chain ID verification. Useful only for local forks.
    #[arg(long, default_value_t = false)]
    no_chain_check: bool,
}

impl RpcArgs {
    fn endpoint(&self) -> String {
        self.rpc_url
            .clone()
            .unwrap_or_else(|| self.network.default_rpc().to_string())
    }

    async fn client(&self) -> Result<RpcClient> {
        let client = RpcClient::new(self.endpoint())?;
        if !self.no_chain_check {
            client.ensure_network(self.network).await?;
        }
        Ok(client)
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ReverseMode {
    Disass,
    Cfg,
    Both,
}

#[derive(Serialize)]
struct ContractReport {
    address: String,
    network: String,
    chain_id: u64,
    rpc_url: String,
    explorer_url: String,
    bytecode_size: usize,
    bytecode_keccak256: String,
    balance_wei_hex: String,
    transaction_count_hex: String,
    selectors: Vec<String>,
    delegatecall_present: bool,
    minimal_proxy_implementation: Option<String>,
    eip1967_implementation: Option<String>,
    eip1967_admin: Option<String>,
    eip1967_beacon: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let filter = match cli.verbose {
        0 => "vertox=warn",
        1 => "vertox=info",
        2 => "vertox=debug",
        _ => "vertox=trace",
    };
    fmt::Subscriber::builder()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| filter.into()))
        .with_target(false)
        .compact()
        .init();

    match cli.command {
        Commands::Scan {
            target,
            rules_dir,
            no_builtin_rules,
            json,
            fail_on,
        } => run_scan(&target, rules_dir.as_deref(), !no_builtin_rules, json, fail_on)?,
        Commands::Build {
            target_dir,
            tool,
            out_dir,
        } => {
            let used = project::build_project(&target_dir, tool, out_dir.as_deref())?;
            println!("VERTOX // BUILD COMPLETE");
            println!("tool       {used}");
            println!("project    {}", target_dir.display());
            if let Some(dir) = out_dir {
                println!("artifacts  {}", dir.display());
            }
        }
        Commands::Fetch {
            address,
            out_dir,
            rpc,
        } => run_fetch(&address, &out_dir, &rpc).await?,
        Commands::Inspect { address, json, rpc } => run_inspect(&address, json, &rpc).await?,
        Commands::Reverse {
            bytecode_file,
            mode,
            out_dir,
        } => run_reverse(&bytecode_file, mode, &out_dir)?,
        Commands::Selectors {
            signatures,
            bytecode_file,
            address,
            rpc,
        } => run_selectors(signatures, bytecode_file.as_deref(), address.as_deref(), &rpc).await?,
        Commands::Storage { address, slots, rpc } => run_storage(&address, slots, &rpc).await?,
        Commands::Rpc { method, params, rpc } => run_rpc(&method, &params, &rpc).await?,
        Commands::Network { network } => print_network(network),
    }

    Ok(())
}

fn run_scan(
    target: &Path,
    rules_dir: Option<&Path>,
    use_builtin: bool,
    json_output: bool,
    fail_on: Option<String>,
) -> Result<()> {
    let report = scan_project(target, rules_dir, use_builtin)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("VERTOX // SOURCE SECURITY SCAN");
        println!("files      {}", report.files_scanned);
        println!("rules      {}", report.rules_loaded);
        println!("findings   {}", report.findings.len());
        println!();
        for finding in &report.findings {
            println!(
                "[{}] {} ({})\n  {}:{}:{}\n  {}\n  {}\n  fix: {}\n",
                finding.severity,
                finding.title,
                finding.rule_id,
                finding.file,
                finding.line,
                finding.column,
                finding.snippet,
                finding.message,
                finding.recommendation
            );
        }
        if report.findings.is_empty() {
            println!("No bundled-rule matches found.");
        }
    }

    if let Some(value) = fail_on {
        let threshold = Severity::parse(&value)?;
        if should_fail(&report, threshold) {
            bail!("scan found one or more findings at {threshold} severity or higher");
        }
    }
    Ok(())
}

async fn run_fetch(address: &str, out_dir: &Path, rpc_args: &RpcArgs) -> Result<()> {
    rpc::validate_address(address)?;
    let client = rpc_args.client().await?;
    let code = client.get_code(address).await?;
    if code.is_empty() {
        bail!("address contains no deployed contract bytecode");
    }

    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let suffix = &address[address.len() - 8..];
    let bin_path = out_dir.join(format!("contract_{suffix}.bin"));
    let hex_path = out_dir.join(format!("contract_{suffix}.hex"));
    fs::write(&bin_path, &code)?;
    fs::write(&hex_path, format!("0x{}\n", hex::encode(&code)))?;

    println!("VERTOX // CONTRACT FETCHED");
    println!("network    {}", rpc_args.network.display_name());
    println!("address    {address}");
    println!("bytes      {}", code.len());
    println!("binary     {}", bin_path.display());
    println!("hex        {}", hex_path.display());
    println!("explorer   {}/address/{address}", rpc_args.network.explorer());
    Ok(())
}

async fn run_inspect(address: &str, json_output: bool, rpc_args: &RpcArgs) -> Result<()> {
    rpc::validate_address(address)?;
    let client = rpc_args.client().await?;
    let chain_id = client.chain_id().await?;
    let code = client.get_code(address).await?;
    if code.is_empty() {
        bail!("address contains no deployed contract bytecode");
    }

    let balance = client.get_balance(address).await?;
    let tx_count = client.get_transaction_count(address).await?;
    let instructions = disassemble(&code);
    let selectors: Vec<String> = discover_push4_selectors(&instructions).into_iter().collect();
    let implementation_slot = eip1967_slot("eip1967.proxy.implementation");
    let admin_slot = eip1967_slot("eip1967.proxy.admin");
    let beacon_slot = eip1967_slot("eip1967.proxy.beacon");
    let impl_word = client.get_storage_at(address, &implementation_slot).await?;
    let admin_word = client.get_storage_at(address, &admin_slot).await?;
    let beacon_word = client.get_storage_at(address, &beacon_slot).await?;

    let report = ContractReport {
        address: address.to_string(),
        network: rpc_args.network.display_name().to_string(),
        chain_id,
        rpc_url: client.url().to_string(),
        explorer_url: format!("{}/address/{address}", rpc_args.network.explorer()),
        bytecode_size: code.len(),
        bytecode_keccak256: format!("0x{}", hex::encode(keccak256(&code))),
        balance_wei_hex: balance,
        transaction_count_hex: tx_count,
        selectors,
        delegatecall_present: instructions.iter().any(|ins| ins.opcode == 0xf4),
        minimal_proxy_implementation: detect_eip1167(&code),
        eip1967_implementation: storage_word_to_address(&impl_word),
        eip1967_admin: storage_word_to_address(&admin_word),
        eip1967_beacon: storage_word_to_address(&beacon_word),
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("VERTOX // CONTRACT INTELLIGENCE");
    println!("network       {}", report.network);
    println!("chain id      {}", report.chain_id);
    println!("address       {}", report.address);
    println!("bytecode      {} bytes", report.bytecode_size);
    println!("code hash     {}", report.bytecode_keccak256);
    println!("selectors     {}", report.selectors.len());
    println!(
        "delegatecall  {}",
        if report.delegatecall_present { "detected" } else { "not detected" }
    );
    print_optional("eip1967 impl", report.eip1967_implementation.as_deref());
    print_optional("eip1967 admin", report.eip1967_admin.as_deref());
    print_optional("eip1967 beacon", report.eip1967_beacon.as_deref());
    print_optional("eip1167 impl", report.minimal_proxy_implementation.as_deref());
    println!("explorer      {}", report.explorer_url);

    if !report.selectors.is_empty() {
        println!();
        println!("PUSH4 selectors");
        for selector in report.selectors.iter().take(64) {
            println!("  {selector}");
        }
        if report.selectors.len() > 64 {
            println!("  ... {} more", report.selectors.len() - 64);
        }
    }
    Ok(())
}

fn print_optional(label: &str, value: Option<&str>) {
    if let Some(value) = value {
        println!("{label:<13} {value}");
    }
}

fn run_reverse(bytecode_file: &Path, mode: ReverseMode, out_dir: &Path) -> Result<()> {
    let code = read_bytecode_file(bytecode_file)?;
    let instructions = disassemble(&code);
    fs::create_dir_all(out_dir)?;

    if matches!(mode, ReverseMode::Disass | ReverseMode::Both) {
        let text = instructions
            .iter()
            .map(|ins| ins.display())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(out_dir.join("disassembly.txt"), format!("{text}\n"))?;
    }

    if matches!(mode, ReverseMode::Cfg | ReverseMode::Both) {
        let cfg = build_cfg(&instructions);
        fs::write(out_dir.join("cfg.dot"), cfg_to_dot(&cfg))?;
        fs::write(out_dir.join("cfg.json"), serde_json::to_vec_pretty(&cfg)?)?;
    }

    println!("VERTOX // EVM REVERSE ANALYSIS");
    println!("input         {}", bytecode_file.display());
    println!("bytecode      {} bytes", code.len());
    println!("instructions  {}", instructions.len());
    println!("output        {}", out_dir.display());
    Ok(())
}

async fn run_selectors(
    signatures: Vec<String>,
    bytecode_file: Option<&Path>,
    address: Option<&str>,
    rpc_args: &RpcArgs,
) -> Result<()> {
    if signatures.is_empty() && bytecode_file.is_none() && address.is_none() {
        bail!("provide --signature, --bytecode-file, or --address");
    }

    if !signatures.is_empty() {
        println!("VERTOX // FUNCTION SELECTORS");
        for signature in signatures {
            let selector = function_selector(&signature);
            println!("0x{}  {signature}", hex::encode(selector));
        }
    }

    let mut discovered = BTreeMap::<String, String>::new();
    if let Some(file) = bytecode_file {
        let code = read_bytecode_file(file)?;
        for selector in discover_push4_selectors(&disassemble(&code)) {
            discovered.insert(selector, format!("file: {}", file.display()));
        }
    }
    if let Some(address) = address {
        let client = rpc_args.client().await?;
        let code = client.get_code(address).await?;
        if code.is_empty() {
            bail!("address contains no deployed contract bytecode");
        }
        for selector in discover_push4_selectors(&disassemble(&code)) {
            discovered.insert(selector, format!("contract: {address}"));
        }
    }

    if !discovered.is_empty() {
        println!();
        println!("Discovered PUSH4 values");
        for (selector, origin) in discovered {
            println!("{selector}  {origin}");
        }
    }
    Ok(())
}

async fn run_storage(address: &str, slots: Vec<String>, rpc_args: &RpcArgs) -> Result<()> {
    rpc::validate_address(address)?;
    let client = rpc_args.client().await?;

    let requested = if slots.is_empty() {
        vec![
            ("implementation".to_string(), eip1967_slot("eip1967.proxy.implementation")),
            ("admin".to_string(), eip1967_slot("eip1967.proxy.admin")),
            ("beacon".to_string(), eip1967_slot("eip1967.proxy.beacon")),
        ]
    } else {
        slots
            .into_iter()
            .map(|slot| {
                let normalized = normalize_storage_slot(&slot)?;
                Ok((slot, normalized))
            })
            .collect::<Result<Vec<_>>>()?
    };

    println!("VERTOX // STORAGE");
    println!("address  {address}");
    for (label, slot) in requested {
        let value = client.get_storage_at(address, &slot).await?;
        println!("\n{label}");
        println!("  slot   {slot}");
        println!("  value  {value}");
        if let Some(addr) = storage_word_to_address(&value) {
            println!("  addr   {addr}");
        }
    }
    Ok(())
}

async fn run_rpc(method: &str, params: &str, rpc_args: &RpcArgs) -> Result<()> {
    let parsed: Value = serde_json::from_str(params).context("--params must be valid JSON")?;
    if !parsed.is_array() {
        bail!("--params must be a JSON array");
    }
    let client = rpc_args.client().await?;
    let result = client.call(method, parsed).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn print_network(network: Network) {
    println!("VERTOX // NETWORK");
    println!("name      {}", network.display_name());
    println!("chain id  {}", network.chain_id());
    println!("rpc       {}", network.default_rpc());
    println!("currency  ETH");
    println!("explorer  {}", network.explorer());
}
