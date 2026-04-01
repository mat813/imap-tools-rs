use std::str::FromStr;

use clap::Args;
use derive_more::Display;
use exn::{Result, ResultExt as _};
use imap_proto::NameAttribute;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use size::Size;

use crate::libs::{
    args,
    base_config::BaseConfig,
    imap::Imap,
    render::{Renderer, new_renderer},
};

#[derive(Debug, Display)]
pub struct ImapDuCommandError(String);
impl std::error::Error for ImapDuCommandError {}

#[derive(Debug, Clone, Default)]
pub enum Sort {
    #[default]
    Name,
    Size,
}

impl FromStr for Sort {
    type Err = ImapDuCommandError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "size" => Ok(Self::Size),
            "name" => Ok(Self::Name),
            _ => Err(ImapDuCommandError(format!("Invalid sort {s}"))),
        }
    }
}

#[derive(Args, Debug, Clone)]
#[command(
    about = "List mailboxes",
    long_about = "This command allows to list mailboxes."
)]
pub struct DiskUsage {
    #[clap(flatten)]
    config: args::Generic,

    /// Only include folder paths matching this re
    #[arg(long)]
    pub include_re: Vec<Regex>,

    /// Exclude folder paths matching this re
    #[arg(long)]
    pub exclude_re: Vec<Regex>,

    /// sort results by `name` or `size`
    #[arg(long, default_value = "name")]
    pub sort: Sort,

    /// Show progress bar
    #[arg(long)]
    pub progress: bool,

    /// Imap pattern
    #[clap(default_value = Some("*"))]
    pattern: Option<String>,

    /// Imap reference list
    reference: Option<String>,
}

impl DiskUsage {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), ImapDuCommandError> {
        let config =
            BaseConfig::new(&self.config).or_raise(|| ImapDuCommandError("config".to_owned()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> =
            Imap::connect_base(&config).or_raise(|| ImapDuCommandError("connect".to_owned()))?;

        #[expect(
            clippy::literal_string_with_formatting_args,
            reason = "We need it for later"
        )]
        let mut renderer = new_renderer("Mailbox Size", "{0:<42} {1}", &["Mailbox", "Attributes"])
            .or_raise(|| ImapDuCommandError("new renderer".to_owned()))?;

        self.run(&mut imap, &mut renderer)
    }

    fn run(
        &self,
        imap: &mut Imap<()>,
        renderer: &mut Box<dyn Renderer>,
    ) -> Result<(), ImapDuCommandError> {
        let mut result: Vec<(String, u64)> = vec![];

        let mailboxes = imap
            .session
            .list(self.reference.as_deref(), self.pattern.as_deref())
            .or_raise(|| {
                ImapDuCommandError(format!(
                    "imap list failed with ref:{:?} and pattern:{:?}",
                    self.reference, self.pattern
                ))
            })?;

        let mailboxes: Vec<_> = mailboxes
            .iter()
            // Filter out folders that are marked as NoSelect, which are not mailboxes, only folders
            .filter(|mbx| !mbx.attributes().contains(&NameAttribute::NoSelect))
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
            .collect();

        let len_mbox = u64::try_from(mailboxes.len())
            .or_raise(|| ImapDuCommandError("parse length".to_owned()))?;

        let bar = self.progress.then(|| ProgressBar::new(len_mbox));

        if let Some(ref b) = bar {
            b.set_style(
                ProgressStyle::with_template(
                    "[{elapsed_precise}/{duration_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                ).or_raise(|| ImapDuCommandError("new progress style".to_owned()))?
                .progress_chars("##-"),
            );
        }

        for mailbox in mailboxes {
            if let Some(ref b) = bar {
                b.inc(1);
                b.set_message(mailbox.name().to_owned());
            }

            let mbx = imap
                .session
                .examine(mailbox.name())
                .or_raise(|| ImapDuCommandError("imap examine".to_owned()))?;

            if mbx.exists == 0 {
                result.push((mailbox.name().to_owned(), 0));
                continue;
            }

            let total: u64 = imap
                .session
                .uid_fetch("1:*", "(RFC822.SIZE)")
                .or_raise(|| ImapDuCommandError("imap uid fetch".to_owned()))?
                .iter()
                .map(|m| u64::from(m.size.unwrap_or(0)))
                .sum();

            result.push((mailbox.name().to_owned(), total));
        }

        if let Some(b) = bar {
            b.finish();
        }

        result.sort_by(|a, b| match self.sort {
            Sort::Name => a.0.cmp(&b.0),
            Sort::Size => a.1.cmp(&b.1),
        });

        for (mbx, total) in result {
            renderer
                .add_row(&[&mbx, &Size::from_bytes(total).format()])
                .or_raise(|| ImapDuCommandError("renderer add row".to_owned()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use super::*;
    use crate::test_helpers::{MockExchange, MockServer, test_base};

    fn default_du() -> DiskUsage {
        DiskUsage {
            config: args::Generic {
                server: Some("127.0.0.1".to_owned()),
                username: Some("test".to_owned()),
                password: Some("test".to_owned()),
                ..Default::default()
            },
            include_re: vec![],
            exclude_re: vec![],
            sort: Sort::Name,
            progress: false,
            pattern: Some("*".to_owned()),
            reference: None,
        }
    }

    #[test]
    fn disk_usage_empty_mailbox() {
        // INBOX has 0 messages → size reported as 0, no UID FETCH needed
        let server = MockServer::start(&[], vec![
            // LIST
            MockExchange::ok(vec!["* LIST () \"/\" INBOX\r\n".into()]),
            // EXAMINE INBOX → 0 messages
            MockExchange::ok(vec!["* 0 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
        ]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let cmd = default_du();
        let mut renderer = new_renderer("test", "{0}", &["col"]).expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    #[test]
    fn disk_usage_sums_message_sizes() {
        // INBOX has 2 messages of 1024 + 2048 bytes = 3072 bytes total
        let server = MockServer::start(&[], vec![
            // LIST
            MockExchange::ok(vec!["* LIST () \"/\" INBOX\r\n".into()]),
            // EXAMINE INBOX → 2 messages
            MockExchange::ok(vec!["* 2 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
            // UID FETCH 1:* (RFC822.SIZE)
            MockExchange::ok(vec![
                "* 1 FETCH (UID 1 RFC822.SIZE 1024)\r\n".into(),
                "* 2 FETCH (UID 2 RFC822.SIZE 2048)\r\n".into(),
            ]),
        ]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let cmd = default_du();
        let mut renderer = new_renderer("test", "{0}", &["col"]).expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    #[test]
    fn disk_usage_skips_noselect_folders() {
        // [Gmail] is NoSelect → filtered out; only INBOX is examined
        let server = MockServer::start(&[], vec![
            // LIST → NoSelect folder + real mailbox
            MockExchange::ok(vec![
                "* LIST (\\Noselect) \"/\" [Gmail]\r\n".into(),
                "* LIST () \"/\" INBOX\r\n".into(),
            ]),
            // EXAMINE INBOX (only INBOX is examined, [Gmail] is skipped)
            MockExchange::ok(vec!["* 0 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
        ]);
        let base = test_base();
        let mut imap: Imap<()> = Imap::connect_base_on_port(&base, server.port).expect("connect");
        let cmd = default_du();
        let mut renderer = new_renderer("test", "{0}", &["col"]).expect("renderer");
        let result = cmd.run(&mut imap, &mut renderer);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }
}
