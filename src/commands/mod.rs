use clap::Subcommand;
use derive_more::Display;
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

#[derive(Debug, Display)]
pub struct MainCommandError(&'static str);

impl std::error::Error for MainCommandError {}

impl MainCommands {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), MainCommandError> {
        match *self {
            Self::Archive(ref archive) => {
                archive.execute().or_raise(|| MainCommandError("archive"))
            }
            Self::Clean(ref clean) => clean.execute().or_raise(|| MainCommandError("clean")),
            Self::FindDups(ref find_dups) => find_dups
                .execute()
                .or_raise(|| MainCommandError("find-dups")),
            Self::List(ref list) => list.execute().or_raise(|| MainCommandError("list")),
            Self::Imap(ref imap) => imap.execute().or_raise(|| MainCommandError("imap")),
        }
    }
}
