use crate::commands::{archive::Archive, clean::Clean, find_dups::FindDups, list::List};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(aliases = &["move"])]
    Archive(Archive),

    #[command(aliases = &["cleanup"])]
    Clean(Clean),

    #[command(aliases = &["find-dup", "findDup", "findDups", "finddup", "finddups"])]
    FindDups(FindDups),

    #[command(aliases = &["ls"])]
    List(List),
}
