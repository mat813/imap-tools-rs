use crate::commands::{clean::Clean, find_dups::FindDups, list::List};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(aliases = &["cleanup"])]
    Clean(Clean),

    #[command(aliases = &["find-dup", "findDup", "findDups"])]
    FindDups(FindDups),

    #[command(aliases = &["ls"])]
    List(List),
}
