use self::list::List;
use anyhow::Result;
use clap::Subcommand;
mod list;

#[derive(Subcommand, Debug, Clone)]
pub enum Imap {
    #[command(aliases = &["ls"])]
    List(List),
}

impl Imap {
    pub fn execute(self) -> Result<()> {
        match self {
            Self::List(list) => list.execute(),
        }
    }
}
