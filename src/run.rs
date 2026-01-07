use crate::commands::MainCommands;
use clap::Parser;
use derive_more::Display;
use exn::{Result, ResultExt as _};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "imap-tools",
    version,
    about = "A collection of tools to manipulate IMAP mailboxes",
    long_about = "These commands will help you curate your IMAP mailboxes.
	
You can remove duplicate emails, clean old emails, or archive them."
)]
struct MainArgs {
    #[command(subcommand)]
    command: MainCommands,
}

#[derive(Debug, Display)]
pub struct RunError;

impl std::error::Error for RunError {}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", err(level = "info"))
)]
/// Dispatch-run our commands
/// # Errors
/// forwards the errors from the commands to `main()`
pub fn run() -> Result<(), RunError> {
    let cli = MainArgs::parse();

    cli.command.execute().or_raise(|| RunError)
}
