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
    pub async fn execute(&self) -> Result<(), ImapCreateCommandError> {
        let config = BaseConfig::new(&self.config)
            .or_raise(|| ImapCreateCommandError("config".to_owned()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> = Imap::connect_base(&config)
            .await
            .or_raise(|| ImapCreateCommandError("connect".to_owned()))?;

        let result = self.run(&mut imap, &mut std::io::stdout()).await;
        imap.close()
            .await
            .or_raise(|| ImapCreateCommandError("imap close failed".to_owned()))?;
        result
    }

    async fn run(
        &self,
        imap: &mut Imap<()>,
        out: &mut dyn std::io::Write,
    ) -> Result<(), ImapCreateCommandError> {
        let mailbox = &self.mailbox;

        match imap.session.create(mailbox).await {
            Ok(()) => writeln!(out, "The mailbox {mailbox} has been created")
                .or_raise(|| ImapCreateCommandError("write output".to_owned()))?,
            Err(async_imap::error::Error::No(reason))
                if reason.contains("Mailbox already exist") =>
            {
                writeln!(out, "Cannot create {mailbox:?}, it already exist: {reason}")
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
    use crate::test_helpers::{MockExchange, MockServer, test_base};

    #[tokio::test]
    async fn create_mailbox_success() {
        let server =
            MockServer::start(&[], vec![MockExchange::ok("CREATE \"NewFolder\"", vec![])]).await;
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let cmd = Create {
            config: args::Generic::default(),
            mailbox: "NewFolder".to_owned(),
        };
        let mut output = Vec::<u8>::new();
        let result = cmd.run(&mut imap, &mut output).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let msg = String::from_utf8(output).expect("utf8");
        assert!(
            msg.contains("has been created"),
            "unexpected output: {msg:?}"
        );
    }

    #[tokio::test]
    async fn create_mailbox_already_exists() {
        let server = MockServer::start(&[], vec![MockExchange::no(
            "CREATE \"INBOX\"",
            "Mailbox already exist",
        )])
        .await;
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let cmd = Create {
            config: args::Generic::default(),
            mailbox: "INBOX".to_owned(),
        };
        let mut output = Vec::<u8>::new();
        let result = cmd.run(&mut imap, &mut output).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(
            result.is_ok(),
            "expected Ok even for NO response, got: {result:?}"
        );
        let msg = String::from_utf8(output).expect("utf8");
        assert!(msg.contains("already exist"), "unexpected output: {msg:?}");
    }
}
