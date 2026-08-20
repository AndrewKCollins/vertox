//! Vertexy CLI entry point.
//!
//! Vertexy is a Solana program analysis toolkit for source-level security scanning,
//! Anchor project review, on-chain program retrieval, and sBPF reverse engineering.

mod commands;
mod dotting;
mod engines;
mod fetcher;
mod helpers;
mod parsers;
mod printers;
mod recap;
mod reverse;
mod state;

use crate::state::app_state::AppState;
use clap::{ArgAction, Parser, Subcommand};
use tracing_subscriber::fmt;

#[derive(Parser)]
#[command(
    name = "vertexy",
    version,
    about = "Solana program security analysis and sBPF reverse-engineering toolkit",
    long_about = "Vertexy helps auditors and Solana developers inspect source code, summarize Anchor account constraints, fetch deployed programs, and reverse engineer compiled sBPF binaries.",
    after_help = "Examples:\n  vertexy scan -d ./program\n  vertexy recap -d ./anchor-project\n  vertexy fetch -p <PROGRAM_ID> -o ./out\n  vertexy reverse --mode both --bytecodes-file ./program.so --out-dir ./out --labeling"
)]
pub struct Cli {
    /// Increase logging verbosity. Repeat for trace-level logging.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Build an Anchor or native SBF project for analysis.
    Build {
        /// Path to the Solana project.
        #[arg(short = 'd', long = "target-dir")]
        target_dir: String,

        /// Directory for build artifacts.
        #[arg(short = 'o', long = "out-dir")]
        out_dir: String,

        /// Allow Vertexy to switch the local Anchor version when required.
        #[arg(long = "unsafe-version-switch", default_value_t = false)]
        unsafe_version_switch: bool,
    },

    /// Scan Solana Rust source with built-in and custom Starlark security rules.
    #[command(name = "scan", visible_alias = "sast")]
    Scan {
        /// Project or directory to scan.
        #[arg(short = 'd', long = "target-dir")]
        target_dir: String,

        /// Directory containing custom .star rules.
        #[arg(short = 'r', long = "rules-dir")]
        rules_dir: Option<String>,

        /// Legacy compatibility flag retained for upstream CLI users.
        #[arg(long = "syn-scan-only", hide = true, default_value_t = false)]
        syn_scan_only: bool,

        /// Disable Vertexy's bundled rules. Requires --rules-dir.
        #[arg(long = "no-internal-rules", action = ArgAction::SetFalse, default_value_t = true)]
        use_internal_rules: bool,

        /// Recursively discover Solana projects below the target directory.
        #[arg(long = "recursive", default_value_t = false)]
        recursive: bool,
    },

    /// Disassemble an sBPF program and/or generate its control-flow graph.
    Reverse {
        /// Analysis output: disass, cfg, or both.
        #[arg(long = "mode", value_parser = ["disass", "cfg", "both"])]
        mode: String,

        /// Directory to write generated analysis files.
        #[arg(short = 'o', long = "out-dir")]
        out_dir: String,

        /// Path to a compiled Solana .so program.
        #[arg(short = 'b', long = "bytecodes-file")]
        bytecodes_file: String,

        /// Add resolved labels and syscall information where possible.
        #[arg(long = "labeling", action)]
        labeling: bool,

        /// Produce a reduced control-flow graph.
        #[arg(long = "reduced", action)]
        reduced: bool,

        /// Restrict CFG generation to the entrypoint subgraph.
        #[arg(long = "only-entrypoint", action)]
        only_entrypoint: bool,
    },

    /// Reinsert selected function clusters into a reduced Graphviz CFG.
    Dotting {
        /// JSON configuration describing functions to reinsert.
        #[arg(short = 'c', long = "config")]
        config: String,

        /// Path to the reduced .dot graph.
        #[arg(short = 'r', long = "reduced-dot-path")]
        reduced_dot_path: String,

        /// Path to the full .dot graph.
        #[arg(short = 'f', long = "full-dot-path")]
        full_dot_path: String,
    },

    /// Fetch a deployed Solana program or account from RPC.
    #[command(name = "fetch", visible_alias = "fetcher")]
    Fetch {
        /// Solana program or account address.
        #[arg(short = 'p', long = "program-id")]
        program_id: String,

        /// Directory for the fetched binary.
        #[arg(short = 'o', long = "out-dir")]
        out_dir: String,

        /// Solana RPC endpoint. Defaults to mainnet-beta.
        #[arg(short = 'r', long = "rpc-url")]
        rpc_url: Option<String>,
    },

    /// Print a Rust syntax tree as JSON for rule development.
    #[command(name = "ast", visible_alias = "ast-utils")]
    Ast {
        /// Rust source file to parse.
        #[arg(short = 'f', long = "file-path")]
        file_path: String,

        /// Emit the AST shape prepared for Vertexy Starlark rules.
        #[arg(short = 's', long = "starlark-syn-ast", default_value_t = false)]
        starlark_syn_ast: bool,
    },

    /// Summarize instructions, account constraints, signers, PDA seeds, and memory hints in an Anchor project.
    Recap {
        /// Root of an Anchor project. Defaults to the current directory.
        #[arg(short = 'd', long = "target-dir")]
        anchor_path: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let default_filter = match cli.verbose {
        0 => "vertexy=warn",
        1 => "vertexy=info",
        2 => "vertexy=debug",
        _ => "vertexy=trace",
    };

    fmt::Subscriber::builder()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| default_filter.into()))
        .with_target(false)
        .compact()
        .init();

    let mut app = AppState {
        cli,
        build_states: vec![],
        sast_states: vec![],
    };

    app.run_cli().await
}
