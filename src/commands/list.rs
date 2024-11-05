use crate::libs::{args, config::Config, error::OurResult, imap::Imap};
use clap::Args;

#[derive(Args, Debug, Clone)]
#[command(
    about = "List mailboxes as per filters",
    long_about = "This command allows to list mailboxes as per filters.

It can be used to debug filters before running commands that have a destructive
effect on the mailboxes."
)]
pub struct List {
    #[clap(flatten)]
    config: args::Generic,
}

type MyExtra = serde_value::Value;

impl List {
    pub fn execute(&self) -> OurResult<()> {
        let config = Config::<MyExtra>::new_with_args(&self.config)?;

        let mut imap = Imap::connect(&config)?;

        for (mailbox, result) in imap.list()? {
            if self.config.dry_run {
                println!("Mailbox: {mailbox} {result:?}");
            } else {
                println!("Mailbox: {mailbox}");
            }
        }

        Ok(())
    }
}
