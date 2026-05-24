use clap::Subcommand;
use exn::{Result, ResultExt as _};
mod archive;
mod clean;
mod find_dups;
mod imap;
mod list;

#[derive(Subcommand, Debug, Clone)]
pub enum MainCommands {
    #[command(aliases = &["move"])]
    Archive(archive::Archive),

    #[command(aliases = &["cleanup"])]
    Clean(clean::Clean),

    #[command(aliases = &["find-dup", "findDup", "findDups", "finddup", "finddups"])]
    FindDups(find_dups::FindDups),

    #[command(aliases = &["ls"])]
    List(list::List),

    #[command(subcommand)]
    Imap(imap::ImapCommands),
}

#[derive(Debug, derive_more::Display)]
pub enum MainCommandError {
    #[display("Running {command} command")]
    Command { command: &'static str },
}

impl std::error::Error for MainCommandError {}

impl MainCommands {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub async fn execute(&self) -> Result<(), MainCommandError> {
        match *self {
            Self::Archive(ref archive) => archive
                .execute()
                .await
                .or_raise(|| MainCommandError::Command { command: "archive" }),
            Self::Clean(ref clean) => clean
                .execute()
                .await
                .or_raise(|| MainCommandError::Command { command: "clean" }),
            Self::FindDups(ref find_dups) => {
                find_dups
                    .execute()
                    .await
                    .or_raise(|| MainCommandError::Command {
                        command: "find-dups",
                    })
            },
            Self::List(ref list) => list
                .execute()
                .await
                .or_raise(|| MainCommandError::Command { command: "list" }),
            Self::Imap(ref imap) => imap
                .execute()
                .await
                .or_raise(|| MainCommandError::Command { command: "imap" }),
        }
    }
}
