use crate::libs::{
    args,
    config::Config,
    imap::{ids_list_to_collapsed_sequence, Imap},
    render::{new_renderer, Renderer},
};
use anyhow::{anyhow, Context, Result};
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
    pub fn execute(&self) -> Result<()> {
        let config = Config::<MyExtra>::new_with_args(&self.config)?;

        let mut imap = Imap::connect(&config)?;

        let mut renderer = new_renderer(
            if config.dry_run {
                "Mailbox Cleaner DRY-RUN"
            } else {
                "Mailbox Cleaner"
            },
            "{0:<42} | {1:>5} | {2:>10} | {3:>4} | {4:>11} | {5:>11} | {6:>4} | {7}",
            &[
                "Mailbox",
                "Msgs",
                "Size",
                "Del",
                "First date",
                "Cutoff date",
                "Days",
                "Sequence",
            ],
        )?;

        for (mailbox, result) in imap.list()? {
            match result.extra {
                Some(ref extra) => {
                    self.cleanup_mailbox(&mut imap, &mut renderer, &mailbox, extra)?;
                }
                None => Err(anyhow!(
                    "Mailbox {mailbox} does not have an extra parameter"
                ))?,
            }
        }

        Ok(())
    }

    fn cleanup_mailbox(
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

        // If there are not enough messages, skip
        if mbx.exists <= 300 {
            return Ok(());
        }

        let messages = imap
            .session
            .uid_fetch("1:*", "(RFC822.SIZE INTERNALDATE)")
            .context("imap uid fetch failed")?;

        let total_size = messages
            .iter()
            .map(|m| i64::from(m.size.unwrap_or(0)))
            .sum::<i64>();

        let first_date = messages
            .iter()
            .next()
            .context("Could not find the first message where there should be one")?
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
                .uid_search(format!("SEEN UNFLAGGED BEFORE {cutoff_str}"))
                .context("imap uid search failed")?; // Search messages by cutoff date

            // Only delete if the rule applies based on mailbox size and message age
            if total_size > rule_size.bytes() && !uids_to_delete.is_empty() {
                // Mark messages for deletion

                let sequence = ids_list_to_collapsed_sequence(&uids_to_delete);

                if !self.config.dry_run {
                    imap.session
                        .select(mailbox)
                        .with_context(|| format!("imap select {mailbox:?} failed"))?;

                    imap.session
                        .uid_store(&sequence, "+FLAGS (\\Deleted)")
                        .context("imap uid store failed")?;

                    // Expunge to permanently remove messages marked for deletion
                    imap.session.close().context("imap close failed")?;
                }

                renderer.add_row(&[
                    &mailbox,
                    &mbx.exists,
                    &Size::from_bytes(total_size).format(),
                    &uids_to_delete.len(),
                    &first_date.format("%d-%b-%Y"),
                    &cutoff_str,
                    &cutoff_date.signed_duration_since(first_date).num_days(),
                    &sequence,
                ])?;

                break;
            }
        }

        Ok(())
    }
}
