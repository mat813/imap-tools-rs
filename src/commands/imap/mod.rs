use clap::Subcommand;
use derive_more::Display;
use exn::{Result, ResultExt as _};
mod create;
mod delete;
mod disk_usage;
mod list;

#[derive(Subcommand, Debug, Clone)]
pub enum ImapCommands {
    #[command(aliases = &["ls"])]
    List(list::List),

    #[command(aliases = &["mkdir"])]
    Create(create::Create),

    #[command(aliases = &["rmdir"])]
    Delete(delete::Delete),

    #[command(aliases = &["du"])]
    DiskUsage(disk_usage::DiskUsage),
}

#[derive(Debug, Display)]
pub struct ImapCommandsError(&'static str);
impl std::error::Error for ImapCommandsError {}

impl ImapCommands {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), ImapCommandsError> {
        match *self {
            Self::List(ref list) => list.execute().or_raise(|| ImapCommandsError("list")),
            Self::Create(ref create) => create.execute().or_raise(|| ImapCommandsError("create")),
            Self::Delete(ref delete) => delete.execute().or_raise(|| ImapCommandsError("delete")),
            Self::DiskUsage(ref du) => du.execute().or_raise(|| ImapCommandsError("du")),
        }
    }
}
