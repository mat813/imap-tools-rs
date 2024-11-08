use crate::libs::{
    args,
    config::Config,
    error::{OurError, OurResult},
    imap::{ids_list_to_collapsed_sequence, Imap},
};
use chrono::{Duration, Utc};
use clap::Args;
use size::Size;
use std::collections::BTreeMap;

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

impl Clean {
    pub fn execute(&self) -> OurResult<()> {
        let config = Config::<MyExtra>::new_with_args(&self.config)?;

        let mut imap = Imap::connect(&config)?;

        for (mailbox, result) in imap.list()? {
            match result.extra {
                Some(ref extra) => {
                    self.cleanup_mailbox(&mut imap, &mailbox, extra)?;
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

    fn cleanup_mailbox(
        &self,
        imap: &mut Imap<MyExtra>,
        mailbox: &str,
        extra: &MyExtra,
    ) -> OurResult<()> {
        let mbx = imap.session.examine(mailbox)?;

        // If there are not enough messages, skip
        if mbx.exists <= 300 {
            return Ok(());
        }

        let messages = imap
            .session
            .uid_fetch("1:*", "(RFC822.SIZE INTERNALDATE)")?;

        let total_size = messages
            .iter()
            .map(|m| i64::from(m.size.unwrap_or(0)))
            .sum::<i64>();

        let first_date = messages
            .iter()
            .next()
            .ok_or_else(|| {
                OurError::config("Could not find the first message where there should be one")
            })?
            .internal_date()
            .unwrap_or_default();

        // If size is less than a MB, skip
        if total_size <= 1_000_000 {
            return Ok(());
        }

        for (rule_size, rule_days) in extra {
            let cutoff_date =
                Utc::now() - Duration::days(i64::try_from(*rule_days).unwrap_or(i64::MAX));

            let cutoff_str = cutoff_date.format("%d-%b-%Y").to_string();

            // Search for messages older than the cutoff date
            let uids_to_delete = imap
                .session
                .uid_search(format!("SEEN UNFLAGGED BEFORE {cutoff_str}"))?; // Search messages by cutoff date

            // Only delete if the rule applies based on mailbox size and message age
            if total_size > rule_size.bytes() && !uids_to_delete.is_empty() {
                // Mark messages for deletion

                let sequence = ids_list_to_collapsed_sequence(&uids_to_delete);

                if self.config.dry_run {
                    print!("dry | ");
                } else {
                    imap.session.select(mailbox)?;

                    imap.session.uid_store(&sequence, "+FLAGS (\\Deleted)")?;

                    // Expunge to permanently remove messages marked for deletion
                    imap.session.close()?;
                }

                println!(
                    "{mailbox:<42} | {:>5} | {:>10} | {:>4} | {:>11} | {:>11} | {:>4} | {sequence}",
                    mbx.exists,
                    Size::from_bytes(total_size).format().to_string(),
                    uids_to_delete.len(),
                    first_date.format("%d-%b-%Y"),
                    cutoff_str,
                    cutoff_date.signed_duration_since(first_date).num_days(),
                );

                break;
            }
        }

        Ok(())
    }
}
