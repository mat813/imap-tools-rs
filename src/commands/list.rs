use clap::Args;
use exn::{Result, ResultExt as _};
use rust_i18n::t;

use crate::libs::{
    args,
    config::Config,
    imap::Imap,
    render::{Renderer, TableSpec, new_renderer},
};

#[derive(Debug, derive_more::Display)]
pub enum ListError {
    #[display("{}", t!("error.shared.config"))]
    Config,
    #[display("{}", t!("error.shared.connect"))]
    Connect,
    #[display("{}", t!("error.shared.new_renderer"))]
    NewRenderer,
    #[display("{}", t!("error.shared.imap_close"))]
    ImapClose,
    #[display("{}", t!("error.shared.imap_list"))]
    ImapList,
    #[display("{}", t!("error.shared.run_list"))]
    Run,
    #[display("{}", t!("error.shared.renderer_add_row"))]
    RendererAddRow,
}
impl std::error::Error for ListError {}

#[derive(Args, Debug, Clone)]
#[command(
    about = t!("cli.list.about"),
    long_about = t!("cli.list.long_about")
)]
pub struct List {
    #[clap(flatten)]
    config: args::Generic,
}

type MyExtra = serde_value::Value;

static RENDERER_LEN: usize = 2;
static RENDERER_FORMAT: &[&str; RENDERER_LEN] = &[":<42", ""];
/// Stable, untranslated names of the columns, used by the machine-readable renderers.
static RENDERER_KEYS: &[&str; RENDERER_LEN] = &["Mailbox", "Mailbox extra"];

fn table_spec() -> TableSpec<RENDERER_LEN> {
    TableSpec::new(
        t!("render.title.mailbox_list"),
        RENDERER_FORMAT,
        RENDERER_KEYS,
        [
            t!("render.header.mailbox"),
            t!("render.header.mailbox_extra"),
        ],
    )
}

impl List {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub async fn execute(&self) -> Result<(), ListError> {
        let config = Config::<MyExtra>::new(&self.config).or_raise(|| ListError::Config)?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap = Imap::connect(&config)
            .await
            .or_raise(|| ListError::Connect)?;

        let mut renderer =
            new_renderer(config.base.renderer, table_spec()).or_raise(|| ListError::NewRenderer)?;

        Self::run(&mut imap, &mut renderer)
            .await
            .or_raise(|| ListError::Run)?;
        imap.close().await.or_raise(|| ListError::ImapClose)?;
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(imap, renderer), err(level = "debug"))
    )]
    async fn run(
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer<RENDERER_LEN> + Send>,
    ) -> Result<(), ListError> {
        for (mailbox, result) in imap.list().await.or_raise(|| ListError::ImapList)? {
            renderer
                .add_row(&[
                    &mailbox,
                    &std::fmt::from_fn(|f| write!(f, "{:?}", result.extra)),
                ])
                .or_raise(|| ListError::RendererAddRow)?;
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
        let mut renderer = new_renderer(base.renderer, table_spec()).expect("renderer");
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
