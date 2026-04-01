use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Duration, FixedOffset, Utc};
use clap::Args;
use derive_more::Display;
use exn::{OptionExt as _, Result, ResultExt as _, bail};
use imap::types::Uid;
use imap_proto::NameAttribute;
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

impl Archive {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), ArchiveError> {
        let config =
            Config::<MyExtra>::new(&self.config).or_raise(|| ArchiveError("config".to_owned()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        #[expect(
            clippy::literal_string_with_formatting_args,
            reason = "We need it for later"
        )]
        let mut renderer = new_renderer(
            if config.base.dry_run {
                "Mailbox Archiving DRY-RUN"
            } else {
                "Mailbox Archiving"
            },
            "{0:<42} | {1:>5} | {2:<25} | {3:>5} | {4:>11} | {5}",
            &[
                "Mailbox",
                "Msgs",
                "Archive mbx",
                "Arc",
                "Cutoff date",
                "Sequence",
            ],
        )
        .or_raise(|| ArchiveError("new renderer".to_owned()))?;

        let mut imap = Imap::connect(&config).or_raise(|| ArchiveError("connect".to_owned()))?;

        for (mailbox, result) in imap
            .list()
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
                    .or_raise(|| ArchiveError("archive".to_owned()))?;
                },
                None => bail!(ArchiveError(format!(
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
    fn archive(
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer>,
        mailbox: &str,
        extra: &MyExtra,
        dry_run: bool,
    ) -> Result<(), ArchiveError> {
        let mbx = imap
            .session
            .examine(mailbox)
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
            .or_raise(|| ArchiveError("imap uid search failed".to_owned()))?;

        // Only delete if the rule applies based on mailbox size and message age
        if !uids_to_move.is_empty() {
            let uids_and_sequence_by_mailbox = Self::compute_destinations(
                imap,
                mailbox,
                extra,
                ids_list_to_collapsed_sequence(&uids_to_move),
            )?;

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

                    let names = imap.session.list(None, Some(quoted_mailbox)).or_raise(|| {
                        ArchiveError(format!("imap list pattern {quoted_mailbox:?} failed"))
                    })?;

                    // If archive mailbox does not exist, or is a simple folder that is not a mailbox, create it
                    if names.is_empty()
                        || names
                            .iter()
                            .all(|n| n.attributes().contains(&NameAttribute::NoSelect))
                    {
                        imap.session.create(&archive_mailbox).or_raise(|| {
                            ArchiveError(format!("imap create {archive_mailbox:?} failed"))
                        })?;
                    }
                    drop(names);

                    if imap
                        .has_capability("MOVE")
                        .or_raise(|| ArchiveError("has capability".to_owned()))?
                    {
                        // MV does COPY / MARK \Deleted / EXPUNGE all in one go
                        imap.session
                            .uid_mv(&sequence, quoted_mailbox)
                            .or_raise(|| {
                                ArchiveError(format!("imap move to {quoted_mailbox:?} failed"))
                            })?;
                    } else {
                        // If we don't have MV, do it the old fashion way.
                        imap.session
                            .uid_copy(&sequence, quoted_mailbox)
                            .or_raise(|| {
                                ArchiveError(format!("imap copy to {quoted_mailbox:?} failed"))
                            })?;
                        imap.session
                            .uid_store(&sequence, "+FLAGS (\\Deleted)")
                            .or_raise(|| ArchiveError("imap store failed".to_owned()))?;
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
                    .or_raise(|| ArchiveError("imap close failed".to_owned()))?;
            }
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(imap), ret, err(level = "info"))
    )]
    fn compute_destinations(
        imap: &mut Imap<MyExtra>,
        mailbox: &str,
        extra: &MyExtra,
        uid_set: String,
    ) -> Result<BTreeMap<String, (String, usize)>, ArchiveError> {
        let messages_to_move = imap
            .session
            .uid_fetch(uid_set, "INTERNALDATE")
            .or_raise(|| ArchiveError("imap uid fetch failed".to_owned()))?;

        // First group uids by archive mailbox
        let mut uids_by_mailbox = BTreeMap::<String, HashSet<Uid>>::new();

        for message in messages_to_move.iter() {
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

            uids_by_mailbox
                .entry(mbx)
                .or_default()
                .insert(message.uid.ok_or_raise(||ArchiveError("The server does not support the UIDPLUS capability, and all our operations need UIDs for safety".to_owned()))?);
        }

        // Then compute the emails sequence and length
        let mut uids_and_sequence_by_mailbox = BTreeMap::new();

        for (mailbox, uids) in uids_by_mailbox {
            uids_and_sequence_by_mailbox
                .insert(mailbox, (ids_list_to_collapsed_sequence(&uids), uids.len()));
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
    #![expect(clippy::expect_used, reason = "tests")]

    use chrono::{FixedOffset, TimeZone as _};

    use super::*;
    use crate::{
        libs::args,
        test_helpers::{MockExchange, MockServer, test_base},
    };

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

    #[test]
    fn archive_skips_empty_mailbox() {
        let server = MockServer::start(&[], vec![
            // EXAMINE INBOX → 0 messages
            MockExchange::ok(vec!["* 0 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
        ]);
        let base = test_base();
        let mut imap: Imap<MyExtra> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let extra = MyExtra {
            format: "Archives/%Y/%m/%%MBX".to_owned(),
            days: 30,
        };
        let archive = Archive {
            config: args::Generic::default(),
        };
        let mut renderer = new_renderer("test", "{0}", &["col"]).expect("renderer");
        let result = Archive::archive(&mut imap, &mut renderer, "INBOX", &extra, false);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    #[test]
    fn archive_dry_run_moves_old_messages() {
        let server = MockServer::start(&[], vec![
            // EXAMINE INBOX → 5 messages
            MockExchange::ok(vec!["* 5 EXISTS\r\n".into(), "* 0 RECENT\r\n".into()]),
            // UID SEARCH → UIDs 1, 2, 3
            MockExchange::ok(vec!["* SEARCH 1 2 3\r\n".into()]),
            // UID FETCH INTERNALDATE → 3 old messages (all Jan 2020)
            MockExchange::ok(vec![
                "* 1 FETCH (UID 1 INTERNALDATE \"01-Jan-2020 10:00:00 +0000\")\r\n".into(),
                "* 2 FETCH (UID 2 INTERNALDATE \"02-Jan-2020 10:00:00 +0000\")\r\n".into(),
                "* 3 FETCH (UID 3 INTERNALDATE \"03-Jan-2020 10:00:00 +0000\")\r\n".into(),
            ]),
        ]);
        let base = test_base();
        let mut imap: Imap<MyExtra> =
            Imap::connect_base_on_port(&base, server.port).expect("connect");
        let extra = MyExtra {
            format: "Archives/%Y/%m/%%MBX".to_owned(),
            days: 30,
        };
        let archive = Archive {
            config: args::Generic {
                dry_run: true,
                ..Default::default()
            },
        };
        let mut renderer = new_renderer("test", "{0}", &["col"]).expect("renderer");
        let result = Archive::archive(&mut imap, &mut renderer, "INBOX", &extra, true);
        drop(imap);
        server.join();
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }
}
