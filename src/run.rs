use crate::{commands::Commands, libs::error::OurResult};
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
    command: Commands,
}

/// Dispatch-run our commands
/// # Errors
/// forwards the errors from the commands to `main()`
pub fn run() -> OurResult<()> {
    let cli = MainArgs::parse();

    match cli.command {
        Commands::Archive(archive) => archive.execute(),
        Commands::Clean(clean) => clean.execute(),
        Commands::FindDups(find_dups) => find_dups.execute(),
        Commands::List(list) => list.execute(),
    }
}
