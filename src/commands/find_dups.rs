use std::collections::{HashMap, HashSet};

use async_imap::types::Uid;
use clap::Args;
use derive_more::Display;
use exn::{OptionExt as _, Result, ResultExt as _};
use futures::TryStreamExt as _;
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

static RENDERER_LEN: usize = 3;
static RENDERER_FORMAT: &[&str; RENDERER_LEN] = &[":<42", "", ""];
static RENDERER_HEADERS: &[&str; RENDERER_LEN] = &["Mailbox", "Dups", "Sequence"];

impl FindDups {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub async fn execute(&self) -> Result<(), DuError> {
        let config =
            Config::<MyExtra>::new(&self.config).or_raise(|| DuError("config".to_owned()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut renderer = new_renderer(
            config.base.renderer,
            if config.base.dry_run {
                "Mailbox Deduplication DRY-RUN"
            } else {
                "Mailbox Deduplication"
            },
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .or_raise(|| DuError("new renderer".to_owned()))?;

        let mut imap = Imap::connect(&config)
            .await
            .or_raise(|| DuError("connect".to_owned()))?;

        for (mailbox, _result) in imap
            .list()
            .await
            .or_raise(|| DuError("imap list".to_owned()))?
        {
            Self::process(&mut imap, &mut renderer, &mailbox, config.base.dry_run)
                .await
                .or_raise(|| DuError("process".to_owned()))?;
        }

        imap.close()
            .await
            .or_raise(|| DuError("imap close failed".to_owned()))?;

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(imap, renderer), err(level = "info"))
    )]
    async fn process(
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer<RENDERER_LEN>>,
        mailbox: &str,
        dry_run: bool,
    ) -> Result<(), DuError> {
        // Examine the mailbox in read only mode, so that we don't change any
        // "seen" flags if there are no duplicate messages
        let mbx = imap
            .session
            .examine(mailbox)
            .await
            .or_raise(|| DuError(format!("imap examine {mailbox:?} failed")))?;

        // If there are less than 2 messages, there cannot possible be
        // duplicates, stop here
        if mbx.exists < 2 {
            return Ok(());
        }

        // Fetch message headers to find duplicates
        let mut message_ids: HashMap<String, Vec<Uid>> = HashMap::new();

        {
            let mut stream = imap
                .session
                .uid_fetch("1:*", "(BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)])")
                .await
                .or_raise(|| DuError("imap uid fetch failed".to_owned()))?;

            while let Some(message) = stream
                .try_next()
                .await
                .or_raise(|| DuError("uid fetch stream error".to_owned()))?
            {
                if let Some(id) = Self::parse_message_id(message.header()) {
                    let uid = message.uid.ok_or_raise(|| {
                        DuError("The server does not support the UIDPLUS capability, and all our operations need UIDs for safety".to_owned())
                    })?;
                    message_ids.entry(id).or_default().push(uid);
                }
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

        // Delete duplicate messages
        if !duplicates.is_empty() {
            let duplicate_set = ids_list_to_collapsed_sequence(&duplicates);

            if !dry_run {
                imap.delete_uids(mailbox, &duplicate_set)
                    .await
                    .or_raise(|| DuError("imap delete uids failed".to_owned()))?;
            }

            renderer
                .add_row(&[&mailbox, &duplicates.len(), &duplicate_set])
                .or_raise(|| DuError("renderer add row".to_owned()))?;
        }

        Ok(())
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

    use insta::assert_snapshot;

    use super::*;
    use crate::test_helpers::{MockExchange, MockServer, header_fetch_line, test_base};

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
    fn parse_message_id_folded_header() {
        // RFC 2822 continuation: CRLF followed by a space unfolds to a single line
        let header = b"Message-ID:\r\n <folded@example.com>\r\n\r\n";
        let result = FindDups::parse_message_id_pub(Some(header));
        assert_eq!(result, Some("<folded@example.com>".to_owned()));
    }

    #[test]
    fn parse_message_id_too_short() {
        // "<x@y>" is 5 chars (MIN_MESSAGE_ID_LEN), "<x@>" is 4 chars → None
        let header = b"Message-ID: <x@>\r\n\r\n";
        let result = FindDups::parse_message_id_pub(Some(header));
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn process_no_duplicates() {
        // 2 messages with different Message-IDs → no deletions
        let server = MockServer::start(&[], vec![
            MockExchange::ok("EXAMINE \"INBOX\"", vec![
                "* 2 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            MockExchange::ok(
                "UID FETCH 1:* (BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)])",
                vec![
                    header_fetch_line(1, 1, "<msg1@example.com>"),
                    header_fetch_line(2, 2, "<msg2@example.com>"),
                ],
            ),
        ])
        .await;
        let base = test_base();
        let mut imap: Imap<serde_value::Value> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Deduplication",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = FindDups::process(&mut imap, &mut renderer, "INBOX", false).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"Mailbox,Dups,Sequence");
    }

    #[tokio::test]
    async fn process_keeps_oldest_with_three_duplicates() {
        // 3 messages sharing the same Message-ID: UIDs 1, 3, 5.
        // The oldest (lowest UID = 1) is kept; UIDs 3 and 5 are deleted.
        let server = MockServer::start(&[], vec![
            MockExchange::ok("EXAMINE \"INBOX\"", vec![
                "* 3 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            MockExchange::ok(
                "UID FETCH 1:* (BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)])",
                vec![
                    header_fetch_line(1, 1, "<dup@example.com>"),
                    header_fetch_line(2, 3, "<dup@example.com>"),
                    header_fetch_line(3, 5, "<dup@example.com>"),
                ],
            ),
            // SELECT INBOX
            MockExchange::ok("SELECT \"INBOX\"", vec![
                "* 3 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            // UID STORE +FLAGS (\Deleted) for UIDs 3 and 5
            MockExchange::ok("UID STORE 3,5 +FLAGS (\\Deleted)", vec![]),
            // CLOSE
            MockExchange::ok("CLOSE", vec![]),
        ])
        .await;
        let base = test_base();
        let mut imap: Imap<serde_value::Value> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Deduplication",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = FindDups::process(&mut imap, &mut renderer, "INBOX", false).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @r#"
        Mailbox,Dups,Sequence
        INBOX,2,"3,5"
        "#);
    }

    #[tokio::test]
    async fn process_skips_mailbox_with_one_message() {
        // exists = 1 < 2 → returns immediately without UID FETCH
        let server = MockServer::start(&[], vec![MockExchange::ok("EXAMINE \"INBOX\"", vec![
            "* 1 EXISTS\r\n".into(),
            "* 0 RECENT\r\n".into(),
        ])])
        .await;
        let base = test_base();
        let mut imap: Imap<serde_value::Value> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Deduplication",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = FindDups::process(&mut imap, &mut renderer, "INBOX", false).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"Mailbox,Dups,Sequence");
    }

    #[tokio::test]
    async fn process_dry_run_finds_duplicates() {
        // 3 messages: uid 2 and 3 share the same Message-ID
        let server = MockServer::start(&[], vec![
            // EXAMINE → 3 messages (examine is read-only)
            MockExchange::ok("EXAMINE \"INBOX\"", vec![
                "* 3 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            // UID FETCH BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)]
            MockExchange::ok(
                "UID FETCH 1:* (BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)])",
                vec![
                    header_fetch_line(1, 1, "<unique@example.com>"),
                    header_fetch_line(2, 2, "<dup@example.com>"),
                    header_fetch_line(3, 3, "<dup@example.com>"),
                ],
            ),
        ])
        .await;
        let base = test_base();
        let mut imap: Imap<serde_value::Value> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Deduplication",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = FindDups::process(&mut imap, &mut renderer, "INBOX", true).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Dups,Sequence
        INBOX,1,3
        ");
    }

    #[tokio::test]
    async fn process_destructive_deletes_duplicates() {
        // Same as dry_run test but with dry_run=false: expects SELECT + UID STORE + CLOSE
        let server = MockServer::start(&[], vec![
            // EXAMINE → 3 messages
            MockExchange::ok("EXAMINE \"INBOX\"", vec![
                "* 3 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            // UID FETCH headers
            MockExchange::ok(
                "UID FETCH 1:* (BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)])",
                vec![
                    header_fetch_line(1, 1, "<unique@example.com>"),
                    header_fetch_line(2, 2, "<dup@example.com>"),
                    header_fetch_line(3, 3, "<dup@example.com>"),
                ],
            ),
            // SELECT INBOX (read-write for deletion)
            MockExchange::ok("SELECT \"INBOX\"", vec![
                "* 3 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            // UID STORE +FLAGS (\Deleted)
            MockExchange::ok("UID STORE 3 +FLAGS (\\Deleted)", vec![]),
            // CLOSE (expunge)
            MockExchange::ok("CLOSE", vec![]),
        ])
        .await;
        let base = test_base();
        let mut imap: Imap<serde_value::Value> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Deduplication",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = FindDups::process(&mut imap, &mut renderer, "INBOX", false).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"
        Mailbox,Dups,Sequence
        INBOX,1,3
        ");
    }
}
