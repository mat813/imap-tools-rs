use clap::Parser;
use exn::{Result, ResultExt as _};
use rust_i18n::t;

use crate::commands::MainCommands;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "imap-tools",
    version,
    about = t!("cli.root.about"),
    long_about = t!("cli.root.long_about")
)]
struct MainArgs {
    #[command(subcommand)]
    command: MainCommands,
}

#[derive(Debug, derive_more::Display)]
#[display("{}", t!("error.run.run_error"))]
pub struct RunError;

impl std::error::Error for RunError {}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", err(level = "info"))
)]
/// Dispatch-run our commands
/// # Errors
/// forwards the errors from the commands to `main()`
pub async fn run() -> Result<(), RunError> {
    let cli = MainArgs::parse();

    cli.command.execute().await.or_raise(|| RunError)
}
