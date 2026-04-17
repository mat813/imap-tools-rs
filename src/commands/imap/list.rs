use clap::Args;
use derive_more::Display;
use exn::{Result, ResultExt as _};
use imap_proto::NameAttribute;
use regex::Regex;

use crate::libs::{
    args,
    base_config::BaseConfig,
    imap::Imap,
    render::{Renderer, new_renderer},
};

#[derive(Debug, Display)]
pub struct ImapListCommandError(String);
impl std::error::Error for ImapListCommandError {}

#[derive(Args, Debug, Clone)]
#[command(
    about = "List mailboxes",
    long_about = "This command allows to list mailboxes."
)]
pub struct List {
    #[clap(flatten)]
    config: args::Generic,

    /// Only include folder paths matching this re
    #[arg(long)]
    pub include_re: Vec<Regex>,

    /// Exclude folder paths matching this re
    #[arg(long)]
    pub exclude_re: Vec<Regex>,

    /// Include `NoSelect` "folders"
    #[arg(long)]
    pub no_select: bool,

    /// Imap pattern
    #[clap(default_value = Some("*"))]
    pattern: Option<String>,

    /// Imap reference list
    reference: Option<String>,
}

static RENDERER_FORMAT: &str = "{0:<42} {1}";
static RENDERER_HEADERS_LEN: usize = 2;
static RENDERER_HEADERS: &[&str; RENDERER_HEADERS_LEN] = &["Mailbox", "Attributes"];

impl List {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), ImapListCommandError> {
        let config =
            BaseConfig::new(&self.config).or_raise(|| ImapListCommandError("config".into()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> =
            Imap::connect_base(&config).or_raise(|| ImapListCommandError("connect".into()))?;

        let mut renderer = new_renderer(
            config.renderer,
            "Mailbox List",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .or_raise(|| ImapListCommandError("new renderer".to_owned()))?;

        self.run(&mut imap, &mut renderer)
    }

    fn run(
        &self,
        imap: &mut Imap<()>,
        renderer: &mut Box<dyn Renderer<RENDERER_HEADERS_LEN>>,
    ) -> Result<(), ImapListCommandError> {
        for mailbox in imap
            .session
            .list(self.reference.as_deref(), self.pattern.as_deref())
            .or_raise(|| {
                ImapListCommandError(format!(
                    "imap list failed with ref:{:?} and pattern:{:?}",
                    self.reference, self.pattern
                ))
            })?
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
                .or_raise(|| ImapListCommandError("renderer add row".to_owned()))?;
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

    #[test]
    fn list_returns_all_regular_mailboxes() {
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST () \"/\" INBOX\r\n".into(),
            "* LIST () \"/\" Sent\r\n".into(),
        ])]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let cmd = default_list();
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox List",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        INBOX,[]
        Sent,[]
        ");
    }

    #[test]
    fn list_excludes_noselect_by_default() {
        // [Gmail] is NoSelect → filtered out; INBOX is kept
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST (\\Noselect) \"/\" [Gmail]\r\n".into(),
            "* LIST () \"/\" INBOX\r\n".into(),
        ])]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let cmd = default_list();
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox List",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        INBOX,[]
        ");
    }

    #[test]
    fn list_no_select_flag_includes_noselect_folders() {
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST (\\Noselect) \"/\" [Gmail]\r\n".into(),
            "* LIST () \"/\" INBOX\r\n".into(),
        ])]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut cmd = default_list();
        cmd.no_select = true;
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox List",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        [Gmail],[NoSelect]
        INBOX,[]
        ");
    }

    #[test]
    fn list_include_re_filters_mailboxes() {
        // include_re = "^INBOX$" → only INBOX is kept, Sent is discarded
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST () \"/\" INBOX\r\n".into(),
            "* LIST () \"/\" Sent\r\n".into(),
        ])]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut cmd = default_list();
        cmd.include_re = vec![Regex::new("^INBOX$").expect("valid regex")];
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox List",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        INBOX,[]
        ");
    }

    #[test]
    fn list_exclude_re_filters_mailboxes() {
        // exclude_re = "^Spam" → Spam/Junk is excluded, INBOX and Sent are kept
        let server = MockServer::start(&[], vec![MockExchange::ok("LIST \"\" *", vec![
            "* LIST () \"/\" INBOX\r\n".into(),
            "* LIST () \"/\" Sent\r\n".into(),
            "* LIST () \"/\" Spam\r\n".into(),
        ])]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut cmd = default_list();
        cmd.exclude_re = vec![Regex::new("^Spam").expect("valid regex")];
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox List",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Attributes
        INBOX,[]
        Sent,[]
        ");
    }
}
