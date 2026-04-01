use std::collections::{HashMap, HashSet};

use clap::Args;
use derive_more::Display;
use exn::{OptionExt as _, Result, ResultExt as _};
use imap::types::{Fetches, Uid};
use regex::Regex;

use crate::libs::{
    args,
    config::Config,
    imap::{Imap, ids_list_to_collapsed_sequence},
    render::{Renderer, new_renderer},
};

#[derive(Debug, Display)]
pub struct DuError(String);
impl std::error::Error for DuError {}

#[derive(Args, Debug, Clone)]
#[command(
    about = "Remove duplicate emails",
    long_about = "This will cleanup your mailboxes of duplicate emails.

It will search each mailbox and if a message with the same message id is found,
it will delete the duplicates."
)]
pub struct FindDups {
    #[clap(flatten)]
    config: args::Generic,
}

type MyExtra = serde_value::Value;

// "<*@*>"
const MIN_MESSAGE_ID_LEN: usize = 5;

// Define the regex as a static global variable
static MESSAGE_ID_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Regular expression to capture Message-ID values across line breaks
    #[expect(clippy::unwrap_used, reason = "re is correct")]
    Regex::new(r"(?i)Message-ID:\s*(<[^>]+>)")
        // We cannot bubble up the error here, so we unwrap(), but it's ok because
        // we wrote it and we know it is valid.
        .unwrap()
});

impl FindDups {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), DuError> {
        let config =
            Config::<MyExtra>::new(&self.config).or_raise(|| DuError("config".to_owned()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut renderer = new_renderer(
            if config.base.dry_run {
                "Mailbox Deduplication DRY-RUN"
            } else {
                "Mailbox Deduplication"
            },
            "[{0}] {1} {2}",
            &["Mailbox", "Dups", "Sequence"],
        )
        .or_raise(|| DuError("new renderer".to_owned()))?;

        let mut imap = Imap::connect(&config).or_raise(|| DuError("connect".to_owned()))?;

        for (mailbox, _result) in imap.list().or_raise(|| DuError("imap list".to_owned()))? {
            Self::process(&mut imap, &mut renderer, &mailbox, config.base.dry_run)
                .or_raise(|| DuError("process".to_owned()))?;
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(imap, renderer), err(level = "info"))
    )]
    fn process(
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer>,
        mailbox: &str,
        dry_run: bool,
    ) -> Result<(), DuError> {
        // Examine the mailbox in read only mode, so that we don't change any
        // "seen" flags if there are no duplicate messages
        let mbx = imap
            .session
            .examine(mailbox)
            .or_raise(|| DuError(format!("imap examine {mailbox:?} failed")))?;

        // If there are less than 2 messages, there cannot possible be
        // duplicates, stop here
        if mbx.exists < 2 {
            return Ok(());
        }

        // Fetch message headers to find duplicates
        let messages = imap
            .session
            .uid_fetch("1:*", "(BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)])")
            .or_raise(|| DuError("imap uid fetch failed".to_owned()))?;
        let duplicates =
            Self::find_duplicates(&messages).or_raise(|| DuError("find duplicates".to_owned()))?;

        // Delete duplicate messages
        if !duplicates.is_empty() {
            let duplicate_set = ids_list_to_collapsed_sequence(&duplicates);

            if !dry_run {
                // Re-open the mailbox in read-write mode
                imap.session
                    .select(mailbox)
                    .or_raise(|| DuError(format!("imap select {mailbox:?} failed")))?;

                imap.session
                    .uid_store(&duplicate_set, "+FLAGS (\\Deleted)")
                    .or_raise(|| DuError("imap uid store failed".to_owned()))?;

                imap.session
                    .close()
                    .or_raise(|| DuError("imap close failed".to_owned()))?;
            }

            renderer
                .add_row(&[&mailbox, &duplicates.len(), &duplicate_set])
                .or_raise(|| DuError("renderer add row".to_owned()))?;
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(messages), ret, , err(level = "info"), fields(messages = messages.len()))
    )]
    fn find_duplicates(messages: &Fetches) -> Result<HashSet<Uid>, DuError> {
        let mut message_ids: HashMap<String, Vec<Uid>> = HashMap::new();

        // Collect message IDs with sequence numbers
        for message in messages.iter() {
            if let Some(id) = Self::parse_message_id(message.header()) {
                message_ids
                    .entry(id)
                    .or_default()
                    .push(message.uid.ok_or_raise(|| DuError("The server does not support the UIDPLUS capability, and all our operations need UIDs for safety".to_owned()))?);
            }
        }

        // Identify duplicates
        let mut duplicates = HashSet::<Uid>::new();

        for ids in message_ids.values() {
            if ids.len() > 1 {
                // Sort ascending so the lowest UID (oldest message) is kept
                let mut sorted = ids.clone();
                sorted.sort_unstable();
                // Keep the oldest, mark the rest as duplicates
                #[expect(clippy::indexing_slicing, reason = "we just tested it's ok")]
                duplicates.extend(&sorted[1..]);
            }
        }

        Ok(duplicates)
    }

    #[cfg(test)]
    pub(super) fn parse_message_id_pub(header: Option<&[u8]>) -> Option<String> {
        Self::parse_message_id(header)
    }

    // Parses a Message-ID from the header
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(header), ret, fields(header = ?header.map(|h| std::str::from_utf8(h))))
    )]
    fn parse_message_id(header: Option<&[u8]>) -> Option<String> {
        let header_text = std::str::from_utf8(header?).ok()?;

        // Unfold RFC 2822 header continuation lines (CRLF or LF followed by whitespace)
        let cleaned_headers = header_text
            .replace("\r\n ", " ")
            .replace("\r\n\t", " ")
            .replace("\n ", " ")
            .replace("\n\t", " ");

        // Find and capture the Message-ID using the regex
        let s = MESSAGE_ID_REGEX
            .captures(&cleaned_headers)?
            .get(1)
            .map(|m| m.as_str().to_owned())?;
        // If the length of the message id is too short, say it's None

        (s.len() >= MIN_MESSAGE_ID_LEN).then_some(s)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use super::*;
    use crate::{
        libs::args,
        test_helpers::{MockExchange, MockServer, header_fetch_line, test_base},
    };

    #[test]
    fn parse_message_id_valid() {
        let header = b"Message-ID: <abc123@example.com>\r\n\r\n";
        let result = FindDups::parse_message_id_pub(Some(header));
        assert_eq!(result, Some("<abc123@example.com>".to_owned()));
    }

    #[test]
    fn parse_message_id_case_insensitive() {
        let header = b"message-id: <ABC@EXAMPLE.COM>\r\n\r\n";
        let result = FindDups::parse_message_id_pub(Some(header));
        assert_eq!(result, Some("<ABC@EXAMPLE.COM>".to_owned()));
    }

    #[test]
    fn parse_message_id_missing() {
        let header = b"Subject: hello\r\n\r\n";
        let result = FindDups::parse_message_id_pub(Some(header));
        assert_eq!(result, None);
    }

    #[test]
    fn parse_message_id_none_input() {
        assert_eq!(FindDups::parse_message_id_pub(None), None);
    }

    #[test]
    fn process_skips_mailbox_with_one_message() {
        // exists = 1 < 2 → returns immediately without UID FETCH
        let server = MockServer::start(&[], vec![MockExchange::ok(vec![
            "* 1 EXISTS\r\n".into(),
            "* 0 RECENT\r\n".into(),
        ])]);
        let base = test_base();
        let mut imap: Imap<serde_value::Value> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let find_dups = FindDups {
            config: args::Generic::default(),
        };
        let mut renderer = new_renderer("test", "{0}", &["col"]).expect("renderer");
        let result = FindDups::process(&mut imap, &mut renderer, "INBOX", false);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    #[test]
    fn process_dry_run_finds_duplicates() {
        // 3 messages: uid 2 and 3 share the same Message-ID
        let server = MockServer::start(&[], vec![
            // EXAMINE → 3 messages (examine is read-only)
            MockExchange::ok(vec!["* 3 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
            // UID FETCH BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)]
            MockExchange::ok(vec![
                header_fetch_line(1, 1, "<unique@example.com>"),
                header_fetch_line(2, 2, "<dup@example.com>"),
                header_fetch_line(3, 3, "<dup@example.com>"),
            ]),
        ]);
        let base = test_base();
        let mut imap: Imap<serde_value::Value> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let find_dups = FindDups {
            config: args::Generic {
                dry_run: true,
                ..Default::default()
            },
        };
        let mut renderer = new_renderer("test", "{0}", &["col"]).expect("renderer");
        let result = FindDups::process(&mut imap, &mut renderer, "INBOX", true);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }
}
