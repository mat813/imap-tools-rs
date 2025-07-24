use crate::libs::{
    args,
    config::Config,
    imap::{ids_list_to_collapsed_sequence, Imap},
    render::{new_renderer, Renderer},
};
use anyhow::{bail, Context as _, Result};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use clap::Args;
use imap::types::Uid;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

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
    pub fn execute(&self) -> Result<()> {
        let config = Config::<MyExtra>::new(&self.config)?;
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
        )?;

        let mut imap = Imap::connect(&config)?;

        for (mailbox, result) in imap.list()? {
            match result.extra {
                Some(ref extra) => {
                    self.archive(&mut imap, &mut renderer, &mailbox, extra)?;
                }
                None => bail!("Mailbox {mailbox} does not have an extra parameter"),
            }
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, imap, renderer), err(level = "info"))
    )]
    fn archive(
        &self,
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer>,
        mailbox: &str,
        extra: &MyExtra,
    ) -> Result<()> {
        let mbx = imap
            .session
            .examine(mailbox)
            .with_context(|| format!("imap examine {mailbox:?} failed"))?;

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
            .context("imap uid search failed")?;

        // Only delete if the rule applies based on mailbox size and message age
        if !uids_to_move.is_empty() {
            let uids_and_sequence_by_mailbox = Self::compute_destinations(
                imap,
                mailbox,
                extra,
                ids_list_to_collapsed_sequence(&uids_to_move),
            )?;

            if self.config.dry_run {
                for (archive_mailbox, (sequence, moving_msgs)) in uids_and_sequence_by_mailbox {
                    renderer.add_row(&[
                        &mailbox,
                        &mbx.exists,
                        &archive_mailbox.replace(mailbox, "%MBX"),
                        &moving_msgs,
                        &cutoff_str,
                        &sequence,
                    ])?;
                }
            } else {
                imap.session
                    .select(mailbox)
                    .with_context(|| format!("imap select {mailbox:?} failed"))?;

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

                    // If archive mailbox does not exist, create it
                    if imap
                        .session
                        .list(None, Some(quoted_mailbox))
                        .with_context(|| format!("imap list pattern {quoted_mailbox:?} failed"))?
                        .is_empty()
                    {
                        imap.session
                            .create(&archive_mailbox)
                            .with_context(|| format!("imap create {archive_mailbox:?} failed"))?;
                    }

                    if imap.has_capability("MOVE")? {
                        // MV does COPY / MARK \Deleted / EXPUNGE all in one go
                        imap.session
                            .uid_mv(&sequence, quoted_mailbox)
                            .with_context(|| format!("imap move to {quoted_mailbox:?} failed"))?;
                    } else {
                        // If we don't have MV, do it the old fashion way.
                        imap.session
                            .uid_copy(&sequence, quoted_mailbox)
                            .with_context(|| format!("imap copy to {quoted_mailbox:?} failed"))?;
                        imap.session
                            .uid_store(&sequence, "+FLAGS (\\Deleted)")
                            .context("imap store failed")?;
                    }

                    renderer.add_row(&[
                        &mailbox,
                        &mbx.exists,
                        &archive_mailbox.replace(mailbox, "%MBX"),
                        &moving_msgs,
                        &cutoff_str,
                        &sequence,
                    ])?;
                }

                // Close the moved messages
                imap.session.close().context("imap close failed")?;
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
    ) -> Result<BTreeMap<String, (String, usize)>> {
        let messages_to_move = imap
            .session
            .uid_fetch(uid_set, "INTERNALDATE")
            .context("imap uid fetch failed")?;

        // First group uids by archive mailbox
        let mut uids_by_mailbox = BTreeMap::<String, HashSet<Uid>>::new();

        for message in messages_to_move.iter() {
            let mbx = Self::archive_mbx(
                mailbox,
                &extra.format,
                message.internal_date().unwrap_or_default(),
            );

            uids_by_mailbox
                .entry(mbx)
                .or_default()
                .insert(message.uid.context("The server does not support the UIDPLUS capability, and all our operations need UIDs for safety")?);
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
