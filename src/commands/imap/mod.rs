use clap::Subcommand;
use exn::{Result, ResultExt as _};
use rust_i18n::t;
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

#[derive(Debug, derive_more::Display)]
pub enum ImapCommandsError {
    #[display("{}", t!("error.imap_commands.list"))]
    List,
    #[display("{}", t!("error.imap_commands.create"))]
    Create,
    #[display("{}", t!("error.imap_commands.delete"))]
    Delete,
    #[display("{}", t!("error.imap_commands.disk_usage"))]
    DiskUsage,
}
impl std::error::Error for ImapCommandsError {}

impl ImapCommands {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub async fn execute(&self) -> Result<(), ImapCommandsError> {
        match *self {
            Self::List(ref list) => list.execute().await.or_raise(|| ImapCommandsError::List),
            Self::Create(ref create) => create
                .execute()
                .await
                .or_raise(|| ImapCommandsError::Create),
            Self::Delete(ref delete) => delete
                .execute()
                .await
                .or_raise(|| ImapCommandsError::Delete),
            Self::DiskUsage(ref du) => du.execute().await.or_raise(|| ImapCommandsError::DiskUsage),
        }
    }
}
