use clap::Subcommand;
mod archive;
mod clean;
mod find_dups;
mod imap;
mod list;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(aliases = &["move"])]
    Archive(archive::Archive),

    #[command(aliases = &["cleanup"])]
    Clean(clean::Clean),

    #[command(aliases = &["find-dup", "findDup", "findDups", "finddup", "finddups"])]
    FindDups(find_dups::FindDups),

    #[command(aliases = &["ls"])]
    List(list::List),

    #[command(subcommand)]
    Imap(imap::Imap),
}
