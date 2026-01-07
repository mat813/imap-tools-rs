use crate::libs::{args, base_config::BaseConfig, imap::Imap};
use clap::Args;
use derive_more::Display;
use exn::{Result, ResultExt as _};

#[derive(Debug, Display)]
pub struct ImapCreateCommandError(&'static str);
impl std::error::Error for ImapCreateCommandError {}

#[derive(Args, Debug, Clone)]
#[command(
    about = "Create mailbox",
    long_about = "This command creates a mailbox."
)]
pub struct Create {
    #[clap(flatten)]
    config: args::Generic,

    /// The mailbox to create
    mailbox: String,
}

impl Create {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    #[expect(clippy::print_stdout, reason = "main")]
    pub fn execute(&self) -> Result<(), ImapCreateCommandError> {
        let config = BaseConfig::new(&self.config).or_raise(|| ImapCreateCommandError("config"))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> =
            Imap::connect_base(&config).or_raise(|| ImapCreateCommandError("connect"))?;

        let mailbox = &self.mailbox;

        match imap.session.create(mailbox) {
            Ok(()) => println!("The mailbox {mailbox} has been created"),
            Err(imap::Error::No(no)) if no.information.contains("Mailbox already exist") => {
                println!("Cannot create {mailbox:?}, it already exist: {no}");
            }
            Err(e) => println!("An error occured while creating the mailbox: {e:?}"),
        }

        Ok(())
    }
}
