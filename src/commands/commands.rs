use crate::commands::list::List;
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(aliases = &["ls"])]
    List(List),
}
