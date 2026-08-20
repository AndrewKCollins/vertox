use crate::helpers::{check_binary_installed, create_dir_if_not_exists, get_project_type, ProjectType};
use crate::state::build_state::BuildState;
use crate::{helpers, Commands};
use anyhow::Context;
use log::{debug, warn};
use std::fs;
use std::path::Path;

pub struct BuildCmd {
    pub target_dir: String,
    pub out_dir: String,
    pub unsafe_version_switch: bool,
}

impl BuildCmd {
    pub fn new_from_clap(cmd: &Commands) -> Self {
        match cmd {
            Commands::Build {
                target_dir,
                out_dir,
                unsafe_version_switch,
            } => Self {
                target_dir: target_dir.clone(),
                out_dir: out_dir.clone(),
                unsafe_version_switch: *unsafe_version_switch,
            },
            _ => unreachable!(),
        }
    }
}

fn require_tool(binary: &str, purpose: &str) -> anyhow::Result<()> {
    if check_binary_installed(binary) {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "required tool `{binary}` was not found in PATH ({purpose})"
        ))
    }
}

fn preflight(cmd: &BuildCmd, project_type: ProjectType) -> anyhow::Result<()> {
    let target = Path::new(&cmd.target_dir);
    if !target.is_dir() {
        return Err(anyhow::anyhow!(
            "target directory does not exist: {}",
            target.display()
        ));
    }

    if !create_dir_if_not_exists(&cmd.out_dir) {
        return Err(anyhow::anyhow!(
            "output directory could not be created: {}",
            cmd.out_dir
        ));
    }

    require_tool("cargo", "building Rust projects")?;
    require_tool("solana", "Solana SBF toolchain")?;

    if project_type == ProjectType::Anchor {
        require_tool("anchor", "building Anchor projects")?;
    }

    Ok(())
}

/// Build an Anchor or native SBF project and copy compiled `.so` files into `out_dir`.
pub fn run(cmd: &BuildCmd) -> anyhow::Result<BuildState> {
    debug!("Starting build process for {}", cmd.target_dir);

    let project_type = get_project_type(&cmd.target_dir);
    if project_type == ProjectType::Unknown {
        return Err(anyhow::anyhow!(
            "unable to detect a Solana project at {}",
            cmd.target_dir
        ));
    }

    preflight(cmd, project_type)?;

    match project_type {
        ProjectType::Anchor => build_anchor_project(cmd)?,
        ProjectType::Sbf => build_sbf_project(cmd)?,
        ProjectType::Unknown => unreachable!(),
    }

    copy_built_programs(cmd)?;

    Ok(BuildState {
        name: String::new(),
        target_dir: cmd.target_dir.clone(),
        out_dir: cmd.out_dir.clone(),
    })
}

fn build_anchor_project(cmd: &BuildCmd) -> anyhow::Result<()> {
    debug!("Building Anchor project {}", cmd.target_dir);

    if let Some(version) = helpers::get_anchor_version(Path::new(&cmd.target_dir))? {
        debug!("Detected Anchor version {}", version);
        if cmd.unsafe_version_switch {
            let spinner = helpers::spinner::get_new_spinner(format!(
                "Switching Anchor to {}...",
                version
            ));
            helpers::switch_anchor_version(&version)?;
            spinner.finish_with_message(format!("Switched Anchor to {}", version));
        }
    }

    run_in_project(&cmd.target_dir, "cargo", &["clean"], vec![])?;

    let spinner = helpers::spinner::get_new_spinner(format!(
        "Running `anchor build` in {}",
        cmd.target_dir
    ));
    let result = run_in_project(
        &cmd.target_dir,
        "anchor",
        &["build", "--skip-lint"],
        vec![(
            "RUSTFLAGS",
            "--emit=asm,llvm-bc,llvm-ir,obj,metadata,link,dep-info,mir",
        )],
    );
    match &result {
        Ok(_) => spinner.finish_with_message("Built Anchor project"),
        Err(_) => spinner.finish_with_message("Anchor build failed"),
    }
    result.map(|_| ())
}

fn build_sbf_project(cmd: &BuildCmd) -> anyhow::Result<()> {
    debug!("Building native SBF project {}", cmd.target_dir);

    run_in_project(&cmd.target_dir, "cargo", &["clean"], vec![])?;

    let spinner = helpers::spinner::get_new_spinner(format!(
        "Running `cargo build-sbf` in {}",
        cmd.target_dir
    ));
    let result = run_in_project(
        &cmd.target_dir,
        "cargo",
        &["build-sbf"],
        vec![(
            "RUSTFLAGS",
            "--emit=asm,llvm-bc,llvm-ir,obj,metadata,link,dep-info,mir",
        )],
    );
    match &result {
        Ok(_) => spinner.finish_with_message("Built SBF project"),
        Err(_) => spinner.finish_with_message("SBF build failed"),
    }
    result.map(|_| ())
}

fn run_in_project(
    target_dir: &str,
    command: &str,
    args: &[&str],
    env_vars: Vec<(&str, &str)>,
) -> anyhow::Result<String> {
    let original_dir = std::env::current_dir().context("failed to read current directory")?;
    std::env::set_current_dir(target_dir)
        .with_context(|| format!("failed to enter project directory {target_dir}"))?;

    let result = helpers::run_command(command, args, env_vars);
    let restore_result = std::env::set_current_dir(&original_dir)
        .with_context(|| format!("failed to restore directory {}", original_dir.display()));

    restore_result?;
    result
}

fn copy_built_programs(cmd: &BuildCmd) -> anyhow::Result<()> {
    let deploy_dir = Path::new(&cmd.target_dir).join("target").join("deploy");
    if !deploy_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "build completed but target/deploy was not found at {}",
            deploy_dir.display()
        ));
    }

    let out_dir = Path::new(&cmd.out_dir);
    let mut copied = 0usize;

    for entry in fs::read_dir(&deploy_dir)
        .with_context(|| format!("failed to read {}", deploy_dir.display()))?
    {
        let entry = entry?;
        let source = entry.path();
        let is_shared_object = source
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("so"));

        if !is_shared_object {
            continue;
        }

        let filename = source
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("invalid build artifact path: {}", source.display()))?;
        let destination = out_dir.join(filename);
        fs::copy(&source, &destination).with_context(|| {
            format!(
                "failed to copy {} to {}",
                source.display(),
                destination.display()
            )
        })?;
        copied += 1;
    }

    if copied == 0 {
        warn!(
            "Build completed, but no .so files were found in {}",
            deploy_dir.display()
        );
    }

    Ok(())
}
