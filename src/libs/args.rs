use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct Generic {
    /// Path to the configuration file.
    #[arg(short = 'c', long, default_value = ".imap-tools.toml")]
    pub config: Option<PathBuf>,

    /// The server to connect to.
    #[arg(short = 's', long)]
    pub server: Option<String>,

    /// The username to use for the connection.
    #[arg(short = 'u', long)]
    pub username: Option<String>,

    /// The password to use for the connection.
    #[arg(short = 'p', long)]
    pub password: Option<String>,

    /// The command to use to get the password.
    #[arg(short = 'P', long)]
    pub password_command: Option<String>,

    /// Display all the IMAP commands sent and received.
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Do not actually do any changes to the server.
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}
