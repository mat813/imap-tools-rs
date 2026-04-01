use clap::Args;
use derive_more::Display;
use exn::{Result, ResultExt as _, bail};

use crate::libs::{args, base_config::BaseConfig, imap::Imap};

#[derive(Debug, Display)]
pub struct ImapDeleteCommandError(String);
impl std::error::Error for ImapDeleteCommandError {}

#[derive(Args, Debug, Clone)]
#[command(
    about = "Delete mailbox",
    long_about = "This command deletes a mailbox."
)]
pub struct Delete {
    #[clap(flatten)]
    config: args::Generic,

    /// The mailbox to delete
    mailbox: String,
}

impl Delete {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), ImapDeleteCommandError> {
        let config = BaseConfig::new(&self.config)
            .or_raise(|| ImapDeleteCommandError("config".to_owned()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> = Imap::connect_base(&config)
            .or_raise(|| ImapDeleteCommandError("connect".to_owned()))?;

        self.run(&mut imap, &mut std::io::stdout())
    }

    fn run(
        &self,
        imap: &mut Imap<()>,
        out: &mut dyn std::io::Write,
    ) -> Result<(), ImapDeleteCommandError> {
        let mailbox = &self.mailbox;

        match imap.session.delete(mailbox) {
            Ok(()) => writeln!(out, "The mailbox {mailbox} has been removed")
                .or_raise(|| ImapDeleteCommandError("write output".to_owned()))?,
            Err(imap::Error::No(no)) if no.information.contains("Mailbox doesn't exist") => {
                writeln!(out, "Cannot remove {mailbox:?}, it does not exist: {no}")
                    .or_raise(|| ImapDeleteCommandError("write output".to_owned()))?;
            },
            Err(e) => bail!(ImapDeleteCommandError(format!(
                "imap delete {mailbox:?} failed: {e:?}"
            ))),
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
    fn delete_mailbox_success() {
        let server = MockServer::start(&[], vec![MockExchange::ok(vec![])]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let cmd = Delete {
            config: args::Generic::default(),
            mailbox: "OldFolder".to_owned(),
        };
        let mut output = Vec::<u8>::new();
        let result = cmd.run(&mut imap, &mut output);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let msg = String::from_utf8(output).expect("utf8");
        assert!(
            msg.contains("has been removed"),
            "unexpected output: {msg:?}"
        );
    }

    #[test]
    fn delete_mailbox_not_found() {
        let server = MockServer::start(&[], vec![MockExchange::no("Mailbox doesn't exist")]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let cmd = Delete {
            config: args::Generic::default(),
            mailbox: "Ghost".to_owned(),
        };
        let mut output = Vec::<u8>::new();
        let result = cmd.run(&mut imap, &mut output);
        drop(imap);
        server.join();
        assert!(
            result.is_ok(),
            "expected Ok even for NO response, got: {result:?}"
        );
        let msg = String::from_utf8(output).expect("utf8");
        assert!(msg.contains("does not exist"), "unexpected output: {msg:?}");
    }
}
