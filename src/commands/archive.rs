use crate::libs::{
    args,
    config::Config,
    error::{OurError, OurResult},
    imap::{ids_list_to_collapsed_sequence, Imap},
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use clap::Args;
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
    pub fn execute(&self) -> OurResult<()> {
        let config = Config::<MyExtra>::new_with_args(&self.config)?;

        let mut imap = Imap::connect(&config)?;

        for (mailbox, result) in imap.list()? {
            match result.extra {
                Some(ref extra) => {
                    self.archive(&mut imap, &mailbox, extra)?;
                }
                None => {
                    return Err(OurError::config(format!(
                        "Mailbox {mailbox} does not have an extra parameter"
                    )))
                }
            }
        }

        Ok(())
    }

    fn archive(&self, imap: &mut Imap<MyExtra>, mailbox: &str, extra: &MyExtra) -> OurResult<()> {
        let mbx = imap.session.examine(mailbox)?;

        // If there are no messages, skip
        if mbx.exists == 0 {
            return Ok(());
        }

        let cutoff_date = Utc::now() - Duration::days(i64::from(extra.days));

        let cutoff_str = cutoff_date.format("%d-%b-%Y").to_string();

        // Search for messages older than the cutoff date and that are neither unread nor flagged
        let uids_to_move = imap
            .session
            .uid_search(format!("SEEN UNFLAGGED BEFORE {cutoff_str}"))?;

        // Only delete if the rule applies based on mailbox size and message age
        if !uids_to_move.is_empty() {
            if self.config.dry_run {
                println!(
                    "{mailbox:<42} | {cur_msgs:>5} | {moving_msgs:>5} | {cutoff_str:>11} | {all:?}",
                    cur_msgs = mbx.exists,
                    moving_msgs = uids_to_move.len(),
                    all = ids_list_to_collapsed_sequence(&uids_to_move),
                );
            } else {
                let messages_to_move = imap.session.uid_fetch(
                    ids_list_to_collapsed_sequence(&uids_to_move),
                    "INTERNALDATE",
                )?;

                let mut uids_by_mailbox = BTreeMap::<String, HashSet<u32>>::new();

                for message in &messages_to_move {
                    let mbx = archive_mbx(
                        mailbox,
                        &extra.format,
                        message.internal_date().unwrap_or_default(),
                    );

                    uids_by_mailbox
                        .entry(mbx)
                        .or_default()
                        .insert(message.uid.ok_or(OurError::Uidplus)?);
                }

                imap.session.select(mailbox)?;

                for (archive_mailbox, uids) in uids_by_mailbox {
                    let sequence = ids_list_to_collapsed_sequence(&uids);

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
                    if imap.session.list(None, Some(quoted_mailbox))?.is_empty() {
                        imap.session.create(&archive_mailbox)?;
                    }

                    if imap.has_capability("MOVE")? {
                        // MV does COPY / MARK \Deleted / EXPUNGE all in one go
                        imap.session.uid_mv(&sequence, quoted_mailbox)?;
                    } else {
                        // If we don't have MV, do it the old fashion way.
                        imap.session.uid_copy(&sequence, quoted_mailbox)?;
                        imap.session.uid_store(&sequence, "+FLAGS (\\Deleted)")?;
                    }

                    println!(
                        "{mailbox:<42} | {cur_msgs:>5} | {archive_mailbox:<25} | {moving_msgs:>5} | {cutoff_str:>11} | {sequence}",
                        archive_mailbox = archive_mailbox.replace(mailbox, "%MBX"),
                        cur_msgs = mbx.exists,
                        moving_msgs = uids.len(),
                    );
                }

                // Close the moved messages
                imap.session.close()?;
            }
        }

        Ok(())
    }
}

fn archive_mbx(mailbox: &str, format_str: &str, date: DateTime<FixedOffset>) -> String {
    date.format(format_str).to_string().replace("%MBX", mailbox)
}
