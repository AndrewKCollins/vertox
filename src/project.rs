use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use walkdir::WalkDir;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum BuildTool {
    Auto,
    Foundry,
    Hardhat,
}

pub fn build_project(root: &Path, requested: BuildTool, out_dir: Option<&Path>) -> Result<String> {
    if !root.is_dir() {
        bail!("project directory does not exist: {}", root.display());
    }

    let tool = match requested {
        BuildTool::Auto => detect_tool(root)?,
        other => other,
    };

    let (program, args, artifact_dir) = match tool {
        BuildTool::Foundry => ("forge", vec!["build"], root.join("out")),
        BuildTool::Hardhat => ("npx", vec!["hardhat", "compile"], root.join("artifacts")),
        BuildTool::Auto => unreachable!(),
    };

    let output = Command::new(program)
        .args(&args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("failed to execute {program}; is the build tool installed?"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!("build failed\n{stdout}\n{stderr}");
    }

    if let Some(destination) = out_dir {
        if artifact_dir.exists() {
            copy_tree(&artifact_dir, destination)?;
        }
    }

    Ok(match tool {
        BuildTool::Foundry => "Foundry".into(),
        BuildTool::Hardhat => "Hardhat".into(),
        BuildTool::Auto => unreachable!(),
    })
}

fn detect_tool(root: &Path) -> Result<BuildTool> {
    if root.join("foundry.toml").exists() {
        return Ok(BuildTool::Foundry);
    }

    let hardhat_configs = [
        "hardhat.config.js",
        "hardhat.config.cjs",
        "hardhat.config.mjs",
        "hardhat.config.ts",
    ];
    if hardhat_configs.iter().any(|name| root.join(name).exists()) {
        return Ok(BuildTool::Hardhat);
    }

    bail!(
        "could not detect Foundry or Hardhat project in {}; use --tool explicitly if needed",
        root.display()
    )
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    for entry in WalkDir::new(source) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(source)?;
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target).with_context(|| {
                format!("failed to copy {} to {}", entry.path().display(), target.display())
            })?;
        }
    }
    Ok(())
}
