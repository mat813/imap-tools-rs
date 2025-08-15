use crate::libs::{args, base_config::BaseConfig, imap::Imap};
use clap::Args;
use eyre::Result;

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
    pub fn execute(&self) -> Result<()> {
        let config = BaseConfig::new(&self.config)?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> = Imap::connect_base(&config)?;

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
