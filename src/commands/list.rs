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

static RENDERER_LEN: usize = 2;
static RENDERER_FORMAT: &[&str; RENDERER_LEN] = &[":<42", ""];
static RENDERER_HEADERS: &[&str; RENDERER_LEN] = &["Mailbox", "Mailbox extra"];

impl List {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub async fn execute(&self) -> Result<(), ListError> {
        let config = Config::<MyExtra>::new(&self.config).or_raise(|| ListError("config"))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap = Imap::connect(&config)
            .await
            .or_raise(|| ListError("connect"))?;

        let mut renderer = new_renderer(
            config.base.renderer,
            "Mailbox List",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .or_raise(|| ListError("new renderer"))?;

        let result = Self::run(&mut imap, &mut renderer).await;
        imap.close()
            .await
            .or_raise(|| ListError("imap close failed"))?;
        result
    }

    async fn run(
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer<RENDERER_LEN>>,
    ) -> Result<(), ListError> {
        for (mailbox, result) in imap.list().await.or_raise(|| ListError("list"))? {
            renderer
                .add_row(&[
                    &mailbox,
                    &std::fmt::from_fn(|f| write!(f, "{:?}", result.extra)),
                ])
                .or_raise(|| ListError("renderer add row"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use insta::assert_snapshot;

    use super::*;
    use crate::test_helpers::{MockExchange, MockServer, test_base};

    #[tokio::test]
    async fn list_renders_mailboxes() {
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST () \"/\" INBOX\r\n".into(),
            "* LIST () \"/\" Sent\r\n".into(),
        ])])
        .await;
        let base = test_base();
        let mut imap: Imap<MyExtra> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox List",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = List::run(&mut imap, &mut renderer).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Mailbox extra
        INBOX,None
        Sent,None
        ");
    }
}
