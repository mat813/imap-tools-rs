use async_imap::imap_proto::NameAttribute;
use clap::Args;
use exn::{Result, ResultExt as _};
use futures::TryStreamExt as _;
use regex::Regex;
use rust_i18n::t;

use crate::libs::{
    args,
    base_config::BaseConfig,
    imap::Imap,
    render::{Renderer, TableSpec, new_renderer},
};

#[derive(Debug, derive_more::Display)]
pub enum ImapListCommandError {
    #[display("{}", t!("error.shared.config"))]
    Config,
    #[display("{}", t!("error.shared.connect"))]
    Connect,
    #[display("{}", t!("error.shared.new_renderer"))]
    NewRenderer,
    #[display("{}", t!("error.shared.run_list"))]
    Run,
    #[display("{}", t!("error.shared.imap_close"))]
    ImapClose,
    #[display(
        "{}",
        t!("error.shared.imap_list_reference_pattern", reference = reference : {:?}, pattern = pattern : {:?})
    )]
    ImapList {
        reference: Option<String>,
        pattern: Option<String>,
    },
    #[display("{}", t!("error.imap_list.imap_list_collect"))]
    ImapListCollect,
    #[display("{}", t!("error.shared.renderer_add_row"))]
    RendererAddRow,
}
impl std::error::Error for ImapListCommandError {}

#[derive(Args, Debug, Clone)]
#[command(
    about = t!("cli.imap.list.about"),
    long_about = t!("cli.imap.list.long_about")
)]
pub struct List {
    #[clap(flatten)]
    config: args::Generic,

    /// Only include folder paths matching this regex
    #[arg(long, help = t!("cli.filter.include_re"))]
    pub include_re: Vec<Regex>,

    /// Exclude folder paths matching this regex
    #[arg(long, help = t!("cli.filter.exclude_re"))]
    pub exclude_re: Vec<Regex>,

    /// Include `NoSelect` "folders"
    #[arg(long, help = t!("cli.filter.no_select"))]
    pub no_select: bool,

    /// IMAP pattern
    #[clap(default_value = Some("*"), help = t!("cli.filter.pattern"))]
    pattern: Option<String>,

    /// IMAP reference list
    #[arg(help = t!("cli.filter.reference"))]
    reference: Option<String>,
}

static RENDERER_LEN: usize = 2;
static RENDERER_FORMAT: &[&str; RENDERER_LEN] = &[":<42", ""];
/// Stable, untranslated names of the columns, used by the machine-readable renderers.
static RENDERER_KEYS: &[&str; RENDERER_LEN] = &["Mailbox", "Attributes"];

fn table_spec() -> TableSpec<RENDERER_LEN> {
    TableSpec::new(
        t!("render.title.mailbox_list"),
        RENDERER_FORMAT,
        RENDERER_KEYS,
        [t!("render.header.mailbox"), t!("render.header.attributes")],
    )
}

impl List {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub async fn execute(&self) -> Result<(), ImapListCommandError> {
        let config = BaseConfig::new(&self.config).or_raise(|| ImapListCommandError::Config)?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> = Imap::connect_base(&config)
            .await
            .or_raise(|| ImapListCommandError::Connect)?;

        let mut renderer = new_renderer(config.renderer, table_spec())
            .or_raise(|| ImapListCommandError::NewRenderer)?;

        self.run(&mut imap, &mut renderer)
            .await
            .or_raise(|| ImapListCommandError::Run)?;
        imap.close()
            .await
            .or_raise(|| ImapListCommandError::ImapClose)?;
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, imap, renderer), err(level = "debug"))
    )]
    async fn run(
        &self,
        imap: &mut Imap<()>,
        renderer: &mut Box<dyn Renderer<RENDERER_LEN> + Send>,
    ) -> Result<(), ImapListCommandError> {
        let names: Vec<_> = {
            let stream = imap
                .session
                .list(self.reference.as_deref(), self.pattern.as_deref())
                .await
                .or_raise(|| ImapListCommandError::ImapList {
                    reference: self.reference.clone(),
                    pattern: self.pattern.clone(),
                })?;
            stream
                .try_collect()
                .await
                .or_raise(|| ImapListCommandError::ImapListCollect)?
        };

        for mailbox in names
            .iter()
            // Filter out folders that are marked as NoSelect, which are not mailboxes, only folders
            .filter(|mbx| self.no_select || !mbx.attributes().contains(&NameAttribute::NoSelect))
            // If we have an include regex, keep folders that match it
            // Otherwise, keep everything
            .filter(|mbx| {
                if self.include_re.is_empty() {
                    true
                } else {
                    self.include_re.iter().any(|re| re.is_match(mbx.name()))
                }
            })
            // If we have an exclude regex, filter out folders that match it
            // Otherwise, keep everything
            .filter(|mbx| {
                if self.exclude_re.is_empty() {
                    true
                } else {
                    self.exclude_re.iter().all(|re| !re.is_match(mbx.name()))
                }
            })
        {
            renderer
                .add_row(&[
                    &mailbox.name(),
                    &std::fmt::from_fn(|f| write!(f, "{:?}", mailbox.attributes())),
                ])
                .or_raise(|| ImapListCommandError::RendererAddRow)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, clippy::trivial_regex, reason = "tests")]

    use insta::assert_snapshot;

    use super::*;
    use crate::test_helpers::{MockExchange, MockServer, test_base};

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(), ret))]
    fn default_list() -> List {
        List {
            config: args::Generic {
                server: Some("127.0.0.1".to_owned()),
                username: Some("test".to_owned()),
                password: Some("test".to_owned()),
                ..Default::default()
            },
            include_re: vec![],
            exclude_re: vec![],
            no_select: false,
            pattern: Some("*".to_owned()),
            reference: None,
        }
    }

    #[tokio::test]
    async fn list_returns_all_regular_mailboxes() {
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST () \"/\" INBOX\r\n".into(),
            "* LIST () \"/\" Sent\r\n".into(),
        ])])
        .await;
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let cmd = default_list();
        let mut renderer = new_renderer(base.renderer, table_spec()).expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        INBOX,[]
        Sent,[]
        ");
    }

    #[tokio::test]
    async fn list_excludes_noselect_by_default() {
        // [Gmail] is NoSelect → filtered out; INBOX is kept
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST (\\Noselect) \"/\" [Gmail]\r\n".into(),
            "* LIST () \"/\" INBOX\r\n".into(),
        ])])
        .await;
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let cmd = default_list();
        let mut renderer = new_renderer(base.renderer, table_spec()).expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        INBOX,[]
        ");
    }

    #[tokio::test]
    async fn list_no_select_flag_includes_noselect_folders() {
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST (\\Noselect) \"/\" [Gmail]\r\n".into(),
            "* LIST () \"/\" INBOX\r\n".into(),
        ])])
        .await;
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let mut cmd = default_list();
        cmd.no_select = true;
        let mut renderer = new_renderer(base.renderer, table_spec()).expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        [Gmail],[NoSelect]
        INBOX,[]
        ");
    }

    #[tokio::test]
    async fn list_include_re_filters_mailboxes() {
        // include_re = "^INBOX$" → only INBOX is kept, Sent is discarded
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST () \"/\" INBOX\r\n".into(),
            "* LIST () \"/\" Sent\r\n".into(),
        ])])
        .await;
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let mut cmd = default_list();
        cmd.include_re = vec![Regex::new("^INBOX$").expect("valid regex")];
        let mut renderer = new_renderer(base.renderer, table_spec()).expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        INBOX,[]
        ");
    }

    #[tokio::test]
    async fn list_exclude_re_filters_mailboxes() {
        // exclude_re = "^Spam" → Spam/Junk is excluded, INBOX and Sent are kept
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST () \"/\" INBOX\r\n".into(),
            "* LIST () \"/\" Sent\r\n".into(),
            "* LIST () \"/\" Spam\r\n".into(),
        ])])
        .await;
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let mut cmd = default_list();
        cmd.exclude_re = vec![Regex::new("^Spam").expect("valid regex")];
        let mut renderer = new_renderer(base.renderer, table_spec()).expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        INBOX,[]
        Sent,[]
        ");
    }
}
