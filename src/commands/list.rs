use clap::Args;
use derive_more::Display;
use exn::{Result, ResultExt as _};

use crate::libs::{
    args,
    config::Config,
    imap::Imap,
    render::{Renderer, new_renderer},
};

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

        Self::run(&mut imap, &mut renderer)
    }

    fn run(imap: &mut Imap<MyExtra>, renderer: &mut Box<dyn Renderer>) -> Result<(), ListError> {
        for (mailbox, result) in imap.list().or_raise(|| ListError("list"))? {
            renderer
                .add_row(&[&mailbox, &format!("{:?}", result.extra)])
                .or_raise(|| ListError("renderer add row"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use super::*;
    use crate::test_helpers::{MockExchange, MockServer, test_base};

    #[test]
    fn list_renders_mailboxes() {
        let server = MockServer::start(&[], vec![MockExchange::ok(vec![
            "* LIST () \"/\" INBOX\r\n".into(),
            "* LIST () \"/\" Sent\r\n".into(),
        ])]);
        let base = test_base();
        let mut imap: Imap<MyExtra> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut renderer = new_renderer("test", "{0}", &["col"]).expect("renderer");
        let result = List::run(&mut imap, &mut renderer);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }
}
