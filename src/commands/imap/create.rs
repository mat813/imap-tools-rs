use crate::libs::{args, base_config::BaseConfig, imap::Imap};
use anyhow::Result;
use clap::Args;

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
    #[expect(clippy::print_stdout, reason = "main")]
    pub fn execute(&self) -> Result<()> {
        let config = BaseConfig::new(&self.config)?;

        let mut imap: Imap<()> = Imap::connect_base(&config)?;

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
