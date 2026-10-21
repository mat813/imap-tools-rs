use clap::Args;
use exn::Result;
use rust_i18n::t;

use super::mailbox::{MailboxCommandError, connect_and_run, report};
use crate::libs::{args, imap::Imap};

#[derive(Args, Clone, Debug)]
#[command(
    about = t!("cli.imap.delete.about"),
    long_about = t!("cli.imap.delete.long_about")
)]
pub struct Delete {
    #[clap(flatten)]
    config: args::Generic,

    /// The mailbox to delete
    #[arg(help = t!("cli.imap.delete.mailbox"))]
    mailbox: String,
}

impl Delete {
    pub async fn execute(&self) -> Result<(), MailboxCommandError> {
        connect_and_run(&self.config, "delete", async |imap, out| {
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
            imap.session.delete(mailbox).await,
            "Mailbox doesn't exist",
            t!("output.imap_delete.removed", mailbox = mailbox),
            |reason| t!("output.imap_delete.not_found", mailbox = mailbox : {:?}, reason = reason),
            MailboxCommandError::Delete {
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
    async fn delete_mailbox_success() {
        let server =
            MockServer::start(&[], vec![MockExchange::ok("DELETE \"OldFolder\"", vec![])]).await;
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let cmd = Delete {
            config: args::Generic::default(),
            mailbox: "OldFolder".to_owned(),
        };
        let mut output = Vec::<u8>::new();
        let result = cmd.run(&mut imap, &mut output).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let msg = String::from_utf8(output).expect("utf8");
        assert!(
            msg.contains("has been removed"),
            "unexpected output: {msg:?}"
        );
    }

    #[tokio::test]
    async fn delete_mailbox_not_found() {
        let server = MockServer::start(&[], vec![MockExchange::no(
            "DELETE \"Ghost\"",
            "Mailbox doesn't exist",
        )])
        .await;
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let cmd = Delete {
            config: args::Generic::default(),
            mailbox: "Ghost".to_owned(),
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
        assert!(msg.contains("does not exist"), "unexpected output: {msg:?}");
    }
}
