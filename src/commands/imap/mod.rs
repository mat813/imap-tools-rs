use crate::commands::imap::{create::Create, delete::Delete, list::List};
use anyhow::Result;
use clap::Subcommand;
mod create;
mod delete;
mod list;

#[derive(Subcommand, Debug, Clone)]
pub enum Imap {
    #[command(aliases = &["ls"])]
    List(List),

    #[command(aliases = &["mkdir"])]
    Create(Create),

    #[command(aliases = &["rmdir"])]
    Delete(Delete),
}

impl Imap {
    pub fn execute(self) -> Result<()> {
        match self {
            Self::List(list) => list.execute(),
            Self::Create(create) => create.execute(),
            Self::Delete(delete) => delete.execute(),
        }
    }
}
