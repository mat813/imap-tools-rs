use std::collections::{BTreeMap, HashSet};

use async_imap::{imap_proto::NameAttribute, types::Uid};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use clap::Args;
use derive_more::Display;
use exn::{OptionExt as _, Result, ResultExt as _, bail};
use futures::TryStreamExt as _;
use serde::{Deserialize, Serialize};

use crate::libs::{
    args,
    config::Config,
    imap::{Imap, ids_list_to_collapsed_sequence},
    render::{Renderer, new_renderer},
};

#[derive(Debug, Display)]
pub struct ArchiveError(String);
impl std::error::Error for ArchiveError {}

#[derive(Args, Debug, Clone)]
#[command(
    about = "Move old emails to \"archive\" folders",
    long_about = "This commands allows to archive old emails.

The destination mailbox can be configured, as well as the retention."
)]
pub struct Archive {
    #[clap(flatten)]
    config: args::Generic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MyExtra {
    format: String,
    days: u32, // TODO: should this be a float instead ?
}

static RENDERER_LEN: usize = 6;
static RENDERER_FORMAT: &[&str; RENDERER_LEN] = &[":<42", ":>5", ":<25", ":>5", ":>11", ""];
static RENDERER_HEADERS: &[&str; RENDERER_LEN] = &[
    "Mailbox",
    "Msgs",
    "Archive mbx",
    "Arc",
    "Cutoff date",
    "Sequence",
];

impl Archive {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub async fn execute(&self) -> Result<(), ArchiveError> {
        let config =
            Config::<MyExtra>::new(&self.config).or_raise(|| ArchiveError("config".to_owned()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut renderer = new_renderer(
            config.base.renderer,
            if config.base.dry_run {
                "Mailbox Archiving DRY-RUN"
            } else {
                "Mailbox Archiving"
            },
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .or_raise(|| ArchiveError("new renderer".to_owned()))?;

        let mut imap = Imap::connect(&config)
            .await
            .or_raise(|| ArchiveError("connect".to_owned()))?;

        for (mailbox, result) in imap
            .list()
            .await
            .or_raise(|| ArchiveError("imap list".to_owned()))?
        {
            match result.extra {
                Some(ref extra) => {
                    Self::archive(
                        &mut imap,
                        &mut renderer,
                        &mailbox,
                        extra,
                        config.base.dry_run,
                    )
                    .await
                    .or_raise(|| ArchiveError("archive".to_owned()))?;
                },
                None => bail!(ArchiveError(format!(
                    "Mailbox {mailbox} does not have an extra parameter"
                ))),
            }
        }

        imap.close()
            .await
            .or_raise(|| ArchiveError("imap close failed".to_owned()))?;

        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "archive handles dry-run/non-dry-run with MOVE/COPY paths"
    )]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(imap, renderer), err(level = "info"))
    )]
    async fn archive(
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer<RENDERER_LEN>>,
        mailbox: &str,
        extra: &MyExtra,
        dry_run: bool,
    ) -> Result<(), ArchiveError> {
        let mbx = imap
            .session
            .examine(mailbox)
            .await
            .or_raise(|| ArchiveError(format!("imap examine {mailbox:?} failed")))?;

        // If there are no messages, skip
        if mbx.exists == 0 {
            return Ok(());
        }

        let cutoff_date = Utc::now() - Duration::days(i64::from(extra.days));

        let cutoff_str = cutoff_date.format("%d-%b-%Y").to_string();

        // Search for messages older than the cutoff date and that are neither unread nor flagged
        let uids_to_move = imap
            .session
            .uid_search(format!("SEEN UNFLAGGED BEFORE {cutoff_str}"))
            .await
            .or_raise(|| ArchiveError("imap uid search failed".to_owned()))?;

        // Only delete if the rule applies based on mailbox size and message age
        if !uids_to_move.is_empty() {
            let uids_and_sequence_by_mailbox = Self::compute_destinations(
                imap,
                mailbox,
                extra,
                ids_list_to_collapsed_sequence(&uids_to_move),
            )
            .await?;

            if dry_run {
                for (archive_mailbox, (sequence, moving_msgs)) in uids_and_sequence_by_mailbox {
                    renderer
                        .add_row(&[
                            &mailbox,
                            &mbx.exists,
                            &archive_mailbox.replace(mailbox, "%MBX"),
                            &moving_msgs,
                            &cutoff_str,
                            &sequence,
                        ])
                        .or_raise(|| ArchiveError("renderer add row".to_owned()))?;
                }
            } else {
                imap.session
                    .select(mailbox)
                    .await
                    .or_raise(|| ArchiveError(format!("imap select {mailbox:?} failed")))?;

                for (archive_mailbox, (sequence, moving_msgs)) in uids_and_sequence_by_mailbox {
                    let quoted_mailbox =
                        if archive_mailbox.contains(' ') || archive_mailbox.contains('"') {
                            &format!(
                                "\"{}\"",
                                archive_mailbox.replace('\\', r"\\").replace('"', "\\\"")
                            )
                        } else {
                            &archive_mailbox
                        };

                    let names: Vec<_> = {
                        let s = imap
                            .session
                            .list(None, Some(quoted_mailbox))
                            .await
                            .or_raise(|| {
                                ArchiveError(format!("imap list pattern {quoted_mailbox:?} failed"))
                            })?;
                        s.try_collect().await.or_raise(|| {
                            ArchiveError(format!("imap list stream {quoted_mailbox:?} error"))
                        })?
                    };

                    // If archive mailbox does not exist, or is a simple folder that is not a mailbox, create it
                    if names.is_empty()
                        || names
                            .iter()
                            .all(|n| n.attributes().contains(&NameAttribute::NoSelect))
                    {
                        imap.session.create(&archive_mailbox).await.or_raise(|| {
                            ArchiveError(format!("imap create {archive_mailbox:?} failed"))
                        })?;
                    }

                    if imap
                        .has_capability("MOVE")
                        .await
                        .or_raise(|| ArchiveError("has capability".to_owned()))?
                    {
                        // MV does COPY / MARK \Deleted / EXPUNGE all in one go
                        imap.session
                            .uid_mv(&sequence, quoted_mailbox)
                            .await
                            .or_raise(|| {
                                ArchiveError(format!("imap move to {quoted_mailbox:?} failed"))
                            })?;
                    } else {
                        // If we don't have MV, do it the old fashion way.
                        imap.session
                            .uid_copy(&sequence, quoted_mailbox)
                            .await
                            .or_raise(|| {
                                ArchiveError(format!("imap copy to {quoted_mailbox:?} failed"))
                            })?;

                        {
                            let mut stream = imap
                                .session
                                .uid_store(&sequence, "+FLAGS (\\Deleted)")
                                .await
                                .or_raise(|| ArchiveError("imap store failed".to_owned()))?;
                            while stream
                                .try_next()
                                .await
                                .or_raise(|| ArchiveError("uid store stream error".to_owned()))?
                                .is_some()
                            {}
                        }
                    }

                    renderer
                        .add_row(&[
                            &mailbox,
                            &mbx.exists,
                            &archive_mailbox.replace(mailbox, "%MBX"),
                            &moving_msgs,
                            &cutoff_str,
                            &sequence,
                        ])
                        .or_raise(|| ArchiveError("renderer add row".to_owned()))?;
                }

                // Close the moved messages
                imap.session
                    .close()
                    .await
                    .or_raise(|| ArchiveError("imap close failed".to_owned()))?;
            }
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(imap), ret, err(level = "info"))
    )]
    async fn compute_destinations(
        imap: &mut Imap<MyExtra>,
        mailbox: &str,
        extra: &MyExtra,
        uid_set: String,
    ) -> Result<BTreeMap<String, (String, usize)>, ArchiveError> {
        // First group uids by archive mailbox
        let mut uids_by_mailbox = BTreeMap::<String, HashSet<Uid>>::new();

        {
            let mut stream = imap
                .session
                .uid_fetch(&uid_set, "INTERNALDATE")
                .await
                .or_raise(|| ArchiveError("imap uid fetch failed".to_owned()))?;

            while let Some(message) = stream
                .try_next()
                .await
                .or_raise(|| ArchiveError("uid fetch stream error".to_owned()))?
            {
                let mbx = Self::archive_mbx(
                    mailbox,
                    &extra.format,
                    message.internal_date().ok_or_raise(|| {
                        ArchiveError(format!(
                            "server did not return INTERNALDATE for UID {:?}",
                            message.uid
                        ))
                    })?,
                );

                let uid = message.uid.ok_or_raise(|| {
                    ArchiveError(
                        "The server does not support the UIDPLUS capability, and all our operations need UIDs for safety".to_owned(),
                    )
                })?;
                uids_by_mailbox.entry(mbx).or_default().insert(uid);
            }
        }

        // Then compute the emails sequence and length
        let mut uids_and_sequence_by_mailbox = BTreeMap::new();

        for (mbx, uids) in uids_by_mailbox {
            uids_and_sequence_by_mailbox
                .insert(mbx, (ids_list_to_collapsed_sequence(&uids), uids.len()));
        }

        Ok(uids_and_sequence_by_mailbox)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", ret))]
    fn archive_mbx(mailbox: &str, format_str: &str, date: DateTime<FixedOffset>) -> String {
        date.format(format_str).to_string().replace("%MBX", mailbox)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, clippy::indexing_slicing, reason = "tests")]

    use chrono::{FixedOffset, TimeZone as _};
    use insta::assert_snapshot;

    use super::*;
    use crate::test_helpers::{MockExchange, MockServer, test_base};

    #[tokio::test]
    async fn archive_non_dry_run_with_move() {
        // Non-dry-run path with MOVE capability: SELECT, LIST, UID MV, CLOSE
        let server = MockServer::start(&["MOVE"], vec![
            // EXAMINE INBOX → 5 messages
            MockExchange::ok("EXAMINE \"INBOX\"", vec![
                "* 5 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            // UID SEARCH → UIDs 1, 2, 3
            MockExchange::ok(
                r"/^UID SEARCH SEEN UNFLAGGED BEFORE \d\d-\w\w\w-\d\d\d\d$/",
                vec!["* SEARCH 1 2 3\r\n".into()],
            ),
            // UID FETCH INTERNALDATE → 3 messages all in Jan 2020 (same archive mailbox)
            MockExchange::ok("UID FETCH 1:3 INTERNALDATE", vec![
                "* 1 FETCH (UID 1 INTERNALDATE \"01-Jan-2020 10:00:00 +0000\")\r\n".into(),
                "* 2 FETCH (UID 2 INTERNALDATE \"02-Jan-2020 10:00:00 +0000\")\r\n".into(),
                "* 3 FETCH (UID 3 INTERNALDATE \"03-Jan-2020 10:00:00 +0000\")\r\n".into(),
            ]),
            // SELECT INBOX (read-write)
            MockExchange::ok("SELECT \"INBOX\"", vec![
                "* 5 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            // LIST to check archive mailbox existence → exists and selectable
            MockExchange::ok("LIST \"\" Archives/2020/01/INBOX", vec![
                "* LIST () \"/\" Archives/2020/01/INBOX\r\n".into(),
            ]),
            // UID MV
            MockExchange::ok("UID MOVE 1:3 \"Archives/2020/01/INBOX\"", vec![]),
            // CLOSE
            MockExchange::ok("CLOSE", vec![]),
        ])
        .await;
        let base = test_base();
        let mut imap: Imap<MyExtra> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let extra = MyExtra {
            format: "Archives/%Y/%m/%%MBX".to_owned(),
            days: 30,
        };
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Archiving",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = Archive::archive(&mut imap, &mut renderer, "INBOX", &extra, false).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let out: Vec<String> = renderer
            .output()
            .split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        assert_eq!(out.len(), 3);
        assert_snapshot!(out[0], @"Mailbox,Msgs,Archive mbx,Arc,Cutoff date,Sequence");
        assert!(
            regex::Regex::new(r"^INBOX,5,Archives/2020/01/%MBX,3,\d\d-\w\w\w-\d\d\d\d,1:3$")
                .expect("should parse")
                .is_match(&out[1])
        );
        assert!(out[2].is_empty());
    }

    #[tokio::test]
    async fn archive_non_dry_run_copy_delete_fallback() {
        // Non-dry-run path without MOVE capability: SELECT, LIST, UID COPY, UID STORE, CLOSE
        let server = MockServer::start(&[], vec![
            // EXAMINE INBOX → 5 messages
            MockExchange::ok("EXAMINE \"INBOX\"", vec![
                "* 5 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            // UID SEARCH → UIDs 1, 2, 3
            MockExchange::ok(
                r"/^UID SEARCH SEEN UNFLAGGED BEFORE \d\d-\w\w\w-\d\d\d\d$/",
                vec!["* SEARCH 1 2 3\r\n".into()],
            ),
            // UID FETCH INTERNALDATE → 3 messages all in Jan 2020
            MockExchange::ok("UID FETCH 1:3 INTERNALDATE", vec![
                "* 1 FETCH (UID 1 INTERNALDATE \"01-Jan-2020 10:00:00 +0000\")\r\n".into(),
                "* 2 FETCH (UID 2 INTERNALDATE \"02-Jan-2020 10:00:00 +0000\")\r\n".into(),
                "* 3 FETCH (UID 3 INTERNALDATE \"03-Jan-2020 10:00:00 +0000\")\r\n".into(),
            ]),
            // SELECT INBOX (read-write)
            MockExchange::ok("SELECT \"INBOX\"", vec![
                "* 5 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            // LIST to check archive mailbox existence → exists and selectable
            MockExchange::ok("LIST \"\" Archives/2020/01/INBOX", vec![
                "* LIST () \"/\" Archives/2020/01/INBOX\r\n".into(),
            ]),
            // UID COPY (fallback: no MOVE capability) — async-imap always quotes the mailbox
            MockExchange::ok("UID COPY 1:3 \"Archives/2020/01/INBOX\"", vec![]),
            // UID STORE +FLAGS (\Deleted)
            MockExchange::ok("UID STORE 1:3 +FLAGS (\\Deleted)", vec![]),
            // CLOSE
            MockExchange::ok("CLOSE", vec![]),
        ])
        .await;
        let base = test_base();
        let mut imap: Imap<MyExtra> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let extra = MyExtra {
            format: "Archives/%Y/%m/%%MBX".to_owned(),
            days: 30,
        };
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Archiving",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = Archive::archive(&mut imap, &mut renderer, "INBOX", &extra, false).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let out: Vec<String> = renderer
            .output()
            .split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        assert_eq!(out.len(), 3);
        assert_snapshot!(out[0], @"Mailbox,Msgs,Archive mbx,Arc,Cutoff date,Sequence");
        assert!(
            regex::Regex::new(r"^INBOX,5,Archives/2020/01/%MBX,3,\d\d-\w\w\w-\d\d\d\d,1:3$")
                .expect("should parse")
                .is_match(&out[1])
        );
        assert!(out[2].is_empty());
    }

    #[test]
    fn archive_mbx_date_format() {
        let date = FixedOffset::east_opt(0)
            .expect("valid offset")
            .with_ymd_and_hms(2020, 1, 15, 0, 0, 0)
            .single()
            .expect("valid date");
        // %% in chrono format produces a literal %, so %%MBX → %MBX → replaced with mailbox name
        assert_eq!(
            Archive::archive_mbx("INBOX", "Archives/%Y/%m/%%MBX", date),
            "Archives/2020/01/INBOX"
        );
        assert_eq!(
            Archive::archive_mbx("Sent", "Arch/%Y/%%MBX", date),
            "Arch/2020/Sent"
        );
    }

    #[tokio::test]
    async fn archive_skips_empty_mailbox() {
        let server = MockServer::start(&[], vec![
            // EXAMINE INBOX → 0 messages
            MockExchange::ok("EXAMINE \"INBOX\"", vec![
                "* 0 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
        ])
        .await;
        let base = test_base();
        let mut imap: Imap<MyExtra> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let extra = MyExtra {
            format: "Archives/%Y/%m/%%MBX".to_owned(),
            days: 30,
        };
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Archiving",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = Archive::archive(&mut imap, &mut renderer, "INBOX", &extra, false).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_snapshot!(renderer.output(), @"Mailbox,Msgs,Archive mbx,Arc,Cutoff date,Sequence");
    }

    #[tokio::test]
    async fn archive_dry_run_moves_old_messages() {
        let server = MockServer::start(&[], vec![
            // EXAMINE INBOX → 5 messages
            MockExchange::ok("EXAMINE \"INBOX\"", vec![
                "* 5 EXISTS\r\n".into(),
                "* 0 RECENT\r\n".into(),
            ]),
            // UID SEARCH → UIDs 1, 2, 3
            MockExchange::ok(
                r"/^UID SEARCH SEEN UNFLAGGED BEFORE \d\d-\w\w\w-\d\d\d\d$/",
                vec!["* SEARCH 1 2 3\r\n".into()],
            ),
            // UID FETCH INTERNALDATE → 3 old messages (all Jan 2020)
            MockExchange::ok("UID FETCH 1:3 INTERNALDATE", vec![
                "* 1 FETCH (UID 1 INTERNALDATE \"01-Jan-2020 10:00:00 +0000\")\r\n".into(),
                "* 2 FETCH (UID 2 INTERNALDATE \"02-Jan-2020 10:00:00 +0000\")\r\n".into(),
                "* 3 FETCH (UID 3 INTERNALDATE \"03-Jan-2020 10:00:00 +0000\")\r\n".into(),
            ]),
        ])
        .await;
        let base = test_base();
        let mut imap: Imap<MyExtra> = Imap::connect_base_on_port(&base, server.port)
            .await
            .expect("connect");
        let extra = MyExtra {
            format: "Archives/%Y/%m/%%MBX".to_owned(),
            days: 30,
        };
        let mut renderer = new_renderer(
            base.renderer,
            "Mailbox Archiving",
            RENDERER_FORMAT,
            RENDERER_HEADERS,
        )
        .expect("renderer");
        let result = Archive::archive(&mut imap, &mut renderer, "INBOX", &extra, true).await;
        let _ = imap.close().await;
        server.join().await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        let out: Vec<String> = renderer
            .output()
            .split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        assert_eq!(out.len(), 3);
        assert_snapshot!(out[0], @"Mailbox,Msgs,Archive mbx,Arc,Cutoff date,Sequence");
        assert!(
            regex::Regex::new(r"^INBOX,5,Archives/2020/01/%MBX,3,\d\d-\w\w\w-\d\d\d\d,1:3$")
                .expect("should parse")
                .is_match(&out[1])
        );
        assert!(out[2].is_empty());
    }
}
