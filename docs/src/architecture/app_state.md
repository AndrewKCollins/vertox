# `AppState` architecture

`AppState` is the small runtime dispatcher behind Vertexy's CLI. It owns the parsed command and the build or scan state collected during execution.

## Location

```text
src/state/app_state.rs
```

## Structure

```rust
pub struct AppState {
    pub cli: Cli,
    pub build_states: Vec<BuildState>,
    pub sast_states: Vec<SastState>,
}
```

The command-line parser is defined in `src/main.rs`. `AppState::run_cli()` then dispatches the selected command into the command modules under `src/commands/`.

## Error behavior

`run_cli()` returns `anyhow::Result<()>`. Command errors are propagated to `main` instead of being logged and discarded. This matters for automation because a failed Vertexy command produces a non-zero process exit status.

Conceptually:

```rust
match &self.cli.command {
    Commands::Build { .. } => self.build_project(...),
    Commands::Scan { .. } => self.run_sast(...),
    Commands::Reverse { .. } => self.run_reverse(...),
    Commands::Fetch { .. } => self.run_fetcher(...).await,
    // ...
}
```

## State collection

The build and source-analysis paths retain their resulting `BuildState` and `SastState` values. Reverse engineering, fetch, AST, recap, and dotting commands complete their work directly and return success or failure.

## Related

- [CLI usage](../cli_usage.md)
- [Architecture overview](../architecture.md)
