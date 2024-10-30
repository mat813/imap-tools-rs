use crate::commands::Commands;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "imap-tools",
    version,
    about = "A collection of tools to manipulate IMAP mailboxes",
    long_about = "These commands will help you curate your IMAP mailboxes."
)]
struct MainArgs {
    #[command(subcommand)]
    command: Commands,
}

pub fn run() {
    let cli = MainArgs::parse();

    if let Err(error) = match cli.command {
        Commands::List(list) => list.execute(),
    } {
        eprintln!("{error}");
    }
}
