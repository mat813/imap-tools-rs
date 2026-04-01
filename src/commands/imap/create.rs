use clap::Args;
use derive_more::Display;
use exn::{Result, ResultExt as _, bail};

use crate::libs::{args, base_config::BaseConfig, imap::Imap};

#[derive(Debug, Display)]
pub struct ImapCreateCommandError(String);
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
    pub fn execute(&self) -> Result<(), ImapCreateCommandError> {
        let config = BaseConfig::new(&self.config)
            .or_raise(|| ImapCreateCommandError("config".to_owned()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> = Imap::connect_base(&config)
            .or_raise(|| ImapCreateCommandError("connect".to_owned()))?;

        self.run(&mut imap, &mut std::io::stdout())
    }

    fn run(
        &self,
        imap: &mut Imap<()>,
        out: &mut dyn std::io::Write,
    ) -> Result<(), ImapCreateCommandError> {
        let mailbox = &self.mailbox;

        match imap.session.create(mailbox) {
            Ok(()) => writeln!(out, "The mailbox {mailbox} has been created")
                .or_raise(|| ImapCreateCommandError("write output".to_owned()))?,
            Err(imap::Error::No(no)) if no.information.contains("Mailbox already exist") => {
                writeln!(out, "Cannot create {mailbox:?}, it already exist: {no}")
                    .or_raise(|| ImapCreateCommandError("write output".to_owned()))?;
            },
            Err(e) => bail!(ImapCreateCommandError(format!(
                "imap create {mailbox:?} failed: {e:?}"
            ))),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use super::*;
    use crate::test_helpers::{MockExchange, MockServer};

    fn test_base() -> BaseConfig {
        BaseConfig::new(&args::Generic {
            server: Some("127.0.0.1".to_owned()),
            username: Some("test".to_owned()),
            password: Some("test".to_owned()),
            ..Default::default()
        })
        .expect("test base config")
    }

    #[test]
    fn create_mailbox_success() {
        let server = MockServer::start(&[], vec![MockExchange::ok(vec![])]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let cmd = Create {
            config: args::Generic::default(),
            mailbox: "NewFolder".to_owned(),
        };
        let mut output = Vec::<u8>::new();
        let result = cmd.run(&mut imap, &mut output);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let msg = String::from_utf8(output).expect("utf8");
        assert!(
            msg.contains("has been created"),
            "unexpected output: {msg:?}"
        );
    }

    #[test]
    fn create_mailbox_already_exists() {
        let server = MockServer::start(&[], vec![MockExchange::no("Mailbox already exist")]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let cmd = Create {
            config: args::Generic::default(),
            mailbox: "INBOX".to_owned(),
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
        assert!(msg.contains("already exist"), "unexpected output: {msg:?}");
    }
}
