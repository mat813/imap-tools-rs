use std::collections::BTreeMap;

use chrono::{Duration, Utc};
use clap::Args;
use derive_more::Display;
use exn::{OptionExt as _, Result, ResultExt as _, bail};
use size::Size;

use crate::libs::{
    args,
    config::Config,
    imap::{Imap, ids_list_to_collapsed_sequence},
    render::{Renderer, new_renderer},
};

#[derive(Debug, Display)]
pub struct CleanError(String);
impl std::error::Error for CleanError {}

#[derive(Args, Debug, Clone)]
#[command(
    about = "Delete old messages",
    long_about = "This command allows to remove old message from mailboxes.

It can be configured to keep more messages if they don't take too much space."
)]
pub struct Clean {
    #[clap(flatten)]
    config: args::Generic,
}

type MyExtra = BTreeMap<Size, u64>;

/// Minimum number of messages in a mailbox before cleanup is considered.
const MIN_MESSAGE_COUNT: u32 = 300;

/// Minimum total mailbox size in bytes before cleanup is considered.
const MIN_TOTAL_SIZE_BYTES: i64 = 1_000_000;

static RENDERER_FORMAT: &str =
    "{0:<42} | {1:>5} | {2:>10} | {3:>4} | {4:>11} | {5:>11} | {6:>4} | {7}";
static RENDERER_HEADERS_LEN: usize = 8;
static RENDERER_HEADERS: &[&str; RENDERER_HEADERS_LEN] = &[
    "Mailbox",
    "Msgs",
    "Size",
    "Del",
    "First date",
    "Cutoff date",
    "Days",
    "Sequence",
];

impl Clean {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), CleanError> {
        let config =
            Config::<MyExtra>::new(&self.config).or_raise(|| CleanError("config".to_owned()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap = Imap::connect(&config).or_raise(|| CleanError("connect".to_owned()))?;

        let mut renderer = new_renderer(
            config.base.renderer,
            if config.base.dry_run {
                "Mailbox Cleaner DRY-RUN"
            } else {
                "Mailbox Cleaner"
            },
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .or_raise(|| CleanError("new renderer".to_owned()))?;

        for (mailbox, result) in imap.list().or_raise(|| CleanError("list".to_owned()))? {
            match result.extra {
                Some(ref extra) => {
                    Self::cleanup_mailbox(
                        &mut imap,
                        &mut renderer,
                        &mailbox,
                        extra,
                        config.base.dry_run,
                    )
                    .or_raise(|| CleanError("cleanup mailbox".to_owned()))?;
                },
                None => bail!(CleanError(format!(
                    "Mailbox {mailbox} does not have an extra parameter"
                ))),
            }
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(imap, renderer), err(level = "info"))
    )]
    fn cleanup_mailbox(
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer<RENDERER_HEADERS_LEN>>,
        mailbox: &str,
        extra: &MyExtra,
        dry_run: bool,
    ) -> Result<(), CleanError> {
        let mbx = imap
            .session
            .examine(mailbox)
            .or_raise(|| CleanError(format!("imap examine {mailbox:?} failed")))?;

        // If there are not enough messages, skip
        if mbx.exists <= MIN_MESSAGE_COUNT {
            return Ok(());
        }

        let messages = imap
            .session
            .uid_fetch("1:*", "(RFC822.SIZE INTERNALDATE)")
            .or_raise(|| CleanError("imap uid fetch failed".to_owned()))?;

        let total_size = messages
            .iter()
            .map(|m| i64::from(m.size.unwrap_or(0)))
            .sum::<i64>();

        let first_date = messages
            .iter()
            .next()
            .ok_or_raise(|| {
                CleanError("Could not find the first message where there should be one".to_owned())
            })?
            .internal_date()
            .unwrap_or_default();

        // If size is less than the minimum, skip
        if total_size <= MIN_TOTAL_SIZE_BYTES {
            return Ok(());
        }

        for (rule_size, rule_days) in extra {
            let cutoff_date =
                Utc::now() - Duration::days(i64::try_from(*rule_days).unwrap_or(i64::MAX));

            let cutoff_str = cutoff_date.format("%d-%b-%Y").to_string();

            // Search for messages older than the cutoff date
            let uids_to_delete = imap
                .session
                .uid_search(format!("SEEN UNFLAGGED BEFORE {cutoff_str}"))
                .or_raise(|| CleanError("imap uid search failed".to_owned()))?; // Search messages by cutoff date

            // Only delete if the rule applies based on mailbox size and message age
            if total_size > rule_size.bytes() && !uids_to_delete.is_empty() {
                // Mark messages for deletion

                let sequence = ids_list_to_collapsed_sequence(&uids_to_delete);

                if !dry_run {
                    imap.delete_uids(mailbox, &sequence)
                        .or_raise(|| CleanError("imap delete uids failed".to_owned()))?;
                }

                renderer
                    .add_row(&[
                        &mailbox,
                        &mbx.exists,
                        &Size::from_bytes(total_size).format(),
                        &uids_to_delete.len(),
                        &first_date.format("%d-%b-%Y"),
                        &cutoff_str,
                        &cutoff_date.signed_duration_since(first_date).num_days(),
                        &sequence,
                    ])
                    .or_raise(|| CleanError("add row".to_owned()))?;

                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, clippy::indexing_slicing, reason = "tests")]

    use insta::assert_snapshot;

    use super::*;
    use crate::test_helpers::{MockExchange, MockServer, test_base};

    fn test_extra() -> MyExtra {
        // Delete messages older than 30 days when mailbox exceeds 1 MB
        [(Size::from_bytes(1_000_000_i64), 30_u64)].into()
    }

    #[test]
    fn cleanup_skips_small_mailbox() {
        // exists = 50 ≤ 300 → should return immediately without UID FETCH
        let server = MockServer::start(&[], vec![MockExchange::ok("EXAMINE \"INBOX\"", vec![
            "* 50 EXISTS\r\n".into(),
            "* 0 RECENT\r\n".into(),
        ])]);
        let base = test_base();
        let mut imap: Imap<MyExtra> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Cleaner",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result =
            Clean::cleanup_mailbox(&mut imap, &mut renderer, "INBOX", &test_extra(), false);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"Mailbox,Msgs,Size,Del,First date,Cutoff date,Days,Sequence");
    }

    #[test]
    fn cleanup_skips_small_total_size() {
        // exists = 350 but total size < 1 MB → should skip
        let server = MockServer::start(
            &[],
            vec![
                // EXAMINE → 350 messages
                MockExchange::ok("EXAMINE \"INBOX\"",vec!["* 350 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
                // UID FETCH RFC822.SIZE INTERNALDATE → small messages
                MockExchange::ok("UID FETCH 1:* (RFC822.SIZE INTERNALDATE)",vec![
                    "* 1 FETCH (UID 1 RFC822.SIZE 1024 INTERNALDATE \"01-Jan-2020 10:00:00 +0000\")\r\n".into(),
                    "* 2 FETCH (UID 2 RFC822.SIZE 1024 INTERNALDATE \"02-Jan-2020 10:00:00 +0000\")\r\n".into(),
                ])
            ],
        );
        let base = test_base();
        let mut imap: Imap<MyExtra> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Cleaner",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result =
            Clean::cleanup_mailbox(&mut imap, &mut renderer, "INBOX", &test_extra(), false);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"Mailbox,Msgs,Size,Del,First date,Cutoff date,Days,Sequence");
    }

    #[test]
    fn cleanup_skips_when_no_old_messages() {
        // Large mailbox (> 1 MB) but UID SEARCH returns empty → no deletion
        let server = MockServer::start(
            &[],
            vec![
                // EXAMINE → 350 messages
                MockExchange::ok("EXAMINE \"INBOX\"",vec!["* 350 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
                // UID FETCH → 2 large messages (total > 1 MB)
                MockExchange::ok("UID FETCH 1:* (RFC822.SIZE INTERNALDATE)",vec![
                    "* 1 FETCH (UID 1 RFC822.SIZE 600000 INTERNALDATE \"01-Jan-2020 10:00:00 +0000\")\r\n".into(),
                    "* 2 FETCH (UID 2 RFC822.SIZE 600000 INTERNALDATE \"02-Jan-2020 10:00:00 +0000\")\r\n".into(),
                ]),
                // UID SEARCH → no results
                MockExchange::ok(r"/^UID SEARCH SEEN UNFLAGGED BEFORE \d\d-\w\w\w-\d\d\d\d$/", vec!["* SEARCH\r\n".into()])
            ],
        );
        let base = test_base();
        let mut imap: Imap<MyExtra> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Cleaner",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result =
            Clean::cleanup_mailbox(&mut imap, &mut renderer, "INBOX", &test_extra(), false);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"Mailbox,Msgs,Size,Del,First date,Cutoff date,Days,Sequence");
    }

    #[test]
    fn cleanup_multi_rule_first_skips_second_matches() {
        // Two rules: (500 KB, 1 day) and (1 MB, 365 days). BTreeMap iterates ascending by size,
        // so the 500 KB rule runs first. Total size is 1.2 MB so both thresholds are exceeded,
        // but the 1-day search returns nothing → first rule skipped. 365-day search finds old
        // messages → second rule matches. dry_run=true so no SELECT/STORE/CLOSE.
        let extra: MyExtra = [
            (Size::from_bytes(500_000_i64), 1_u64),
            (Size::from_bytes(1_000_000_i64), 365_u64),
        ]
        .into();
        let server = MockServer::start(
            &[],
            vec![
                // EXAMINE → 350 messages
                MockExchange::ok("EXAMINE \"INBOX\"",vec!["* 350 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
                // UID FETCH → 2 large messages (total 1.2 MB)
                MockExchange::ok("UID FETCH 1:* (RFC822.SIZE INTERNALDATE)",vec![
                    "* 1 FETCH (UID 1 RFC822.SIZE 600000 INTERNALDATE \"01-Jan-2020 10:00:00 +0000\")\r\n".into(),
                    "* 2 FETCH (UID 2 RFC822.SIZE 600000 INTERNALDATE \"02-Jan-2020 10:00:00 +0000\")\r\n".into(),
                ]),
                // UID SEARCH (1 day) → empty
                MockExchange::ok(r"/^UID SEARCH SEEN UNFLAGGED BEFORE \d\d-\w\w\w-\d\d\d\d$/", vec!["* SEARCH\r\n".into()]),
                // UID SEARCH (365 days) → UIDs 1 and 2
                MockExchange::ok(r"/^UID SEARCH SEEN UNFLAGGED BEFORE \d\d-\w\w\w-\d\d\d\d$/", vec!["* SEARCH 1 2\r\n".into()])
            ],
        );
        let base = test_base();
        let mut imap: Imap<MyExtra> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Cleaner",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = Clean::cleanup_mailbox(&mut imap, &mut renderer, "INBOX", &extra, true);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let out: Vec<String> = renderer
            .output()
            .split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        assert_eq!(out.len(), 3);
        assert_snapshot!(out[0], @"Mailbox,Msgs,Size,Del,First date,Cutoff date,Days,Sequence");
        assert!(
            regex::Regex::new(r"^INBOX,350,1.14 MiB,2,01-Jan-2020,\d\d-\w\w\w-\d\d\d\d,\d+,1:2$")
                .expect("should parse")
                .is_match(&out[1]),
            "not matching {:?}",
            out[1]
        );
        assert!(out[2].is_empty());
    }

    #[test]
    fn cleanup_dry_run_large_old_mailbox() {
        // exists = 350, total size > 1 MB, old messages → dry-run: no SELECT/STORE/CLOSE
        let server = MockServer::start(
            &[],
            vec![
                // EXAMINE → 350 messages
                MockExchange::ok("EXAMINE \"INBOX\"",vec!["* 350 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
                // UID FETCH → 2 large old messages (total > 1 MB)
                MockExchange::ok("UID FETCH 1:* (RFC822.SIZE INTERNALDATE)",vec![
                    "* 1 FETCH (UID 1 RFC822.SIZE 600000 INTERNALDATE \"01-Jan-2020 10:00:00 +0000\")\r\n".into(),
                    "* 2 FETCH (UID 2 RFC822.SIZE 600000 INTERNALDATE \"02-Jan-2020 10:00:00 +0000\")\r\n".into(),
                ]),
                // UID SEARCH → old messages to delete
                MockExchange::ok(r"/^UID SEARCH SEEN UNFLAGGED BEFORE \d\d-\w\w\w-\d\d\d\d$/",vec!["* SEARCH 1 2\r\n".into()])
            ],
        );
        let base = test_base();
        let mut imap: Imap<MyExtra> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Cleaner",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = Clean::cleanup_mailbox(&mut imap, &mut renderer, "INBOX", &test_extra(), true);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let out: Vec<String> = renderer
            .output()
            .split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        assert_eq!(out.len(), 3);
        assert_snapshot!(out[0], @"Mailbox,Msgs,Size,Del,First date,Cutoff date,Days,Sequence");
        assert!(
            regex::Regex::new(r"^INBOX,350,1.14 MiB,2,01-Jan-2020,\d\d-\w\w\w-\d\d\d\d,\d+,1:2$")
                .expect("should parse")
                .is_match(&out[1]),
            "not matching {:?}",
            out[1]
        );
        assert!(out[2].is_empty());
    }

    #[test]
    fn cleanup_destructive_large_old_mailbox() {
        // Same as dry_run test but with dry_run=false: expects SELECT + UID STORE + CLOSE
        let server = MockServer::start(
            &[],
            vec![
                // EXAMINE → 350 messages
                MockExchange::ok("EXAMINE \"INBOX\"",vec!["* 350 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
                // UID FETCH RFC822.SIZE INTERNALDATE → 2 large old messages (total > 1 MB)
                MockExchange::ok("UID FETCH 1:* (RFC822.SIZE INTERNALDATE)",vec![
                    "* 1 FETCH (UID 1 RFC822.SIZE 600000 INTERNALDATE \"01-Jan-2020 10:00:00 +0000\")\r\n".into(),
                    "* 2 FETCH (UID 2 RFC822.SIZE 600000 INTERNALDATE \"02-Jan-2020 10:00:00 +0000\")\r\n".into(),
                ]),
                // UID SEARCH → old messages to delete
                MockExchange::ok(r"/^UID SEARCH SEEN UNFLAGGED BEFORE \d\d-\w\w\w-\d\d\d\d$/",vec!["* SEARCH 1 2\r\n".into()]),
                // SELECT INBOX (read-write for deletion)
                MockExchange::ok("SELECT \"INBOX\"",vec!["* 350 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
                // UID STORE +FLAGS (\Deleted)
                MockExchange::ok("UID STORE 1:2 +FLAGS (\\Deleted)",vec![]),
                // CLOSE (expunge)
                MockExchange::ok("CLOSE",vec![]),
            ],
        );
        let base = test_base();
        let mut imap: Imap<MyExtra> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Cleaner",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result =
            Clean::cleanup_mailbox(&mut imap, &mut renderer, "INBOX", &test_extra(), false);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let out: Vec<String> = renderer
            .output()
            .split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        assert_eq!(out.len(), 3);
        assert_snapshot!(out[0], @"Mailbox,Msgs,Size,Del,First date,Cutoff date,Days,Sequence");
        assert!(
            regex::Regex::new(r"^INBOX,350,1.14 MiB,2,01-Jan-2020,\d\d-\w\w\w-\d\d\d\d,\d+,1:2$")
                .expect("should parse")
                .is_match(&out[1]),
            "not matching {:?}",
            out[1]
        );
        assert!(out[2].is_empty());
    }
}
