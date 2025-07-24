use crate::commands::MainCommands;
use anyhow::Result;
use clap::Parser;

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

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", err(level = "info"))
)]
/// Dispatch-run our commands
/// # Errors
/// forwards the errors from the commands to `main()`
pub fn run() -> Result<()> {
    let cli = MainArgs::parse();

    cli.command.execute()
}
