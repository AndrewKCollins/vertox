use crate::state::build_state::BuildState;
use crate::state::sast_state::SastState;
use crate::{commands, Cli, Commands};
use anyhow::Context;
use log::info;

/// Global application state and command dispatcher.
pub struct AppState {
    pub cli: Cli,
    pub build_states: Vec<BuildState>,
    pub sast_states: Vec<SastState>,
}

impl AppState {
    /// Execute the selected CLI command.
    ///
    /// Errors are propagated to `main` so shell scripts and CI receive a non-zero exit code.
    pub async fn run_cli(&mut self) -> anyhow::Result<()> {
        let command = self.cli.command.clone();
        match &command {
            Commands::Reverse {
                mode,
                out_dir,
                bytecodes_file,
                labeling,
                reduced,
                only_entrypoint,
            } => self.run_reverse(
                mode.clone(),
                out_dir.clone(),
                bytecodes_file.clone(),
                *labeling,
                *reduced,
                *only_entrypoint,
            ),
            Commands::Dotting {
                config,
                reduced_dot_path,
                full_dot_path,
            } => self.run_dotting(
                config.clone(),
                reduced_dot_path.clone(),
                full_dot_path.clone(),
            ),
            Commands::Fetch {
                program_id,
                out_dir,
                rpc_url,
            } => {
                self.run_fetcher(program_id.clone(), out_dir.clone(), rpc_url.clone())
                    .await
            }
            cmd @ Commands::Recap { .. } => {
                self.run_recap(&commands::recap_command::RecapCmd::new_from_clap(cmd))
            }
            cmd @ Commands::Build { .. } => {
                self.build_project(&commands::build_command::BuildCmd::new_from_clap(cmd))
            }
            cmd @ Commands::Scan { .. } => {
                self.run_sast(&commands::sast_command::SastCmd::new_from_clap(cmd))
            }
            cmd @ Commands::Ast { .. } => {
                self.run_ast_utils(&commands::ast_utils_command::AstUtilsCmd::new_from_clap(cmd))
                    .await
            }
        }
    }

    fn build_project(&mut self, cmd: &commands::build_command::BuildCmd) -> anyhow::Result<()> {
        let state = commands::build_command::run(cmd)
            .with_context(|| format!("failed to build {}", cmd.target_dir))?;
        self.build_states.push(state);
        Ok(())
    }

    fn run_sast(&mut self, cmd: &commands::sast_command::SastCmd) -> anyhow::Result<()> {
        let states = commands::sast_command::run(cmd)
            .with_context(|| format!("failed to scan {}", cmd.target_dir))?;
        self.sast_states.extend(states);
        Ok(())
    }

    fn run_reverse(
        &mut self,
        mode: String,
        out_dir: String,
        bytecodes_file: String,
        labeling: bool,
        reduced: bool,
        only_entrypoint: bool,
    ) -> anyhow::Result<()> {
        commands::reverse_command::run(
            mode,
            out_dir,
            bytecodes_file,
            labeling,
            reduced,
            only_entrypoint,
        )?;
        info!("Reverse analysis completed.");
        Ok(())
    }

    fn run_dotting(
        &mut self,
        config: String,
        reduced_dot_path: String,
        full_dot_path: String,
    ) -> anyhow::Result<()> {
        commands::dotting_command::run(config, reduced_dot_path, full_dot_path)?;
        info!("CFG editing completed.");
        Ok(())
    }

    async fn run_fetcher(
        &mut self,
        program_id: String,
        output_path: String,
        rpc_url: Option<String>,
    ) -> anyhow::Result<()> {
        commands::fetcher_command::run(program_id, output_path.clone(), rpc_url)
            .await
            .with_context(|| format!("failed to fetch account into {output_path}"))?;
        info!("Fetch completed: {output_path}");
        Ok(())
    }

    async fn run_ast_utils(
        &mut self,
        cmd: &commands::ast_utils_command::AstUtilsCmd,
    ) -> anyhow::Result<()> {
        commands::ast_utils_command::run(cmd)?;
        Ok(())
    }

    fn run_recap(&mut self, cmd: &commands::recap_command::RecapCmd) -> anyhow::Result<()> {
        commands::recap_command::run(cmd)?;
        Ok(())
    }
}
