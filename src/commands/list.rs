use crate::libs::{args, config::Config, imap::Imap, render::new_renderer};
use anyhow::Result;
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
    pub fn execute(&self) -> Result<()> {
        let config = Config::<MyExtra>::new(&self.config)?;

        let mut imap = Imap::connect(&config)?;

        #[expect(
            clippy::literal_string_with_formatting_args,
            reason = "We need it for later"
        )]
        let mut renderer =
            new_renderer("Mailbox List", "{0:<42} {1}", &["Mailbox", "Mailbox extra"])?;

        for (mailbox, result) in imap.list()? {
            renderer.add_row(&[&mailbox, &format!("{:?}", result.extra)])?;
        }

        Ok(())
    }
}
