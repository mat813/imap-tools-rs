use anyhow::Result;
use clap::Subcommand;
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

impl MainCommands {
    pub fn execute(&self) -> Result<()> {
        match *self {
            Self::Archive(ref archive) => archive.execute(),
            Self::Clean(ref clean) => clean.execute(),
            Self::FindDups(ref find_dups) => find_dups.execute(),
            Self::List(ref list) => list.execute(),
            Self::Imap(ref imap) => imap.execute(),
        }
    }
}
