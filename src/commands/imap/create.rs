use clap::Args;
use exn::Result;
use rust_i18n::t;

use super::mailbox::{MailboxCommandError, connect_and_run, report};
use crate::libs::{args, imap::Imap};

#[derive(Args, Clone, Debug)]
#[command(
    about = t!("cli.imap.create.about"),
    long_about = t!("cli.imap.create.long_about")
)]
pub struct Create {
    #[clap(flatten)]
    config: args::Generic,

    /// The mailbox to create
    #[arg(help = t!("cli.imap.create.mailbox"))]
    mailbox: String,
}

impl Create {
    pub async fn execute(&self) -> Result<(), MailboxCommandError> {
        connect_and_run(&self.config, "create", async |imap, out| {
            self.run(imap, out).await
        })
        .await
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, imap, out), err(level = "debug"))
    )]
    async fn run(
        &self,
        imap: &mut Imap<()>,
        out: &mut (dyn std::io::Write + Send),
    ) -> Result<(), MailboxCommandError> {
        let mailbox = &self.mailbox;
        report(
            out,
            imap.session.create(mailbox).await,
            "Mailbox already exist",
            t!("output.imap_create.created", mailbox = mailbox),
            |reason| t!("output.imap_create.already_exists", mailbox = mailbox : {:?}, reason = reason),
            MailboxCommandError::Create {
                mailbox: mailbox.clone(),
            },
        )
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
