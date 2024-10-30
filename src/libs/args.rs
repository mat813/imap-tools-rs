use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct Generic {
    #[arg(short = 'c', long, default_value = ".imap-tools.toml")]
    pub config: Option<PathBuf>,

    #[arg(short = 's', long)]
    pub server: Option<String>,

    #[arg(short = 'u', long)]
    pub username: Option<String>,

    #[arg(short = 'p', long)]
    pub password: Option<String>,

    #[arg(short = 'P', long)]
    pub password_command: Option<String>,

    #[arg(short = 'd', long)]
    pub debug: bool,

    #[arg(short = 'n', long)]
    pub dry_run: bool,
}
