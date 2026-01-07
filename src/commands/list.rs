use crate::libs::{args, config::Config, imap::Imap, render::new_renderer};
use clap::Args;
use derive_more::Display;
use exn::{Result, ResultExt as _};

#[derive(Debug, Display)]
pub struct ListError(&'static str);
impl std::error::Error for ListError {}

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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), ListError> {
        let config = Config::<MyExtra>::new(&self.config).or_raise(|| ListError("config"))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap = Imap::connect(&config).or_raise(|| ListError("connect"))?;

        #[expect(
            clippy::literal_string_with_formatting_args,
            reason = "We need it for later"
        )]
        let mut renderer =
            new_renderer("Mailbox List", "{0:<42} {1}", &["Mailbox", "Mailbox extra"])
                .or_raise(|| ListError("new renderer"))?;

        for (mailbox, result) in imap.list().or_raise(|| ListError("list"))? {
            renderer
                .add_row(&[&mailbox, &format!("{:?}", result.extra)])
                .or_raise(|| ListError("renderer add row"))?;
        }

        Ok(())
    }
}
