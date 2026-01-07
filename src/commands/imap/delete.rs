use crate::libs::{args, base_config::BaseConfig, imap::Imap};
use clap::Args;
use derive_more::Display;
use exn::{Result, ResultExt as _};

#[derive(Debug, Display)]
pub struct ImapDeleteCommandError(&'static str);
impl std::error::Error for ImapDeleteCommandError {}

#[derive(Args, Debug, Clone)]
#[command(
    about = "Create mailbox",
    long_about = "This command creates a mailbox."
)]
pub struct Delete {
    #[clap(flatten)]
    config: args::Generic,

    /// The mailbox to create
    mailbox: String,
}

impl Delete {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    #[expect(clippy::print_stdout, reason = "main")]
    pub fn execute(&self) -> Result<(), ImapDeleteCommandError> {
        let config = BaseConfig::new(&self.config).or_raise(|| ImapDeleteCommandError("config"))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> =
            Imap::connect_base(&config).or_raise(|| ImapDeleteCommandError("connect"))?;

        let mailbox = &self.mailbox;

        match imap.session.delete(mailbox) {
            Ok(()) => println!("The mailbox {mailbox} has been removed"),
            Err(imap::Error::No(no)) if no.information.contains("Mailbox doesn't exist") => {
                println!("Cannot remove {mailbox:?}, it does not exist: {no}");
            }
            Err(e) => println!("An error occured while removing the mailbox: {e:?}"),
        }

        Ok(())
    }
}
