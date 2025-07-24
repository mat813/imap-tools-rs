use anyhow::Result;
use clap::Subcommand;
mod create;
mod delete;
mod list;

#[derive(Subcommand, Debug, Clone)]
pub enum ImapCommands {
    #[command(aliases = &["ls"])]
    List(list::List),

    #[command(aliases = &["mkdir"])]
    Create(create::Create),

    #[command(aliases = &["rmdir"])]
    Delete(delete::Delete),
}

impl ImapCommands {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<()> {
        match *self {
            Self::List(ref list) => list.execute(),
            Self::Create(ref create) => create.execute(),
            Self::Delete(ref delete) => delete.execute(),
        }
    }
}
