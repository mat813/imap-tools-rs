use crate::libs::{
    args,
    config::Config,
    imap::{ids_list_to_collapsed_sequence, Imap},
    render::{new_renderer, Renderer},
};
use chrono::{Duration, Utc};
use clap::Args;
use derive_more::Display;
use exn::{bail, OptionExt as _, Result, ResultExt as _};
use size::Size;
use std::collections::BTreeMap;

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

        #[expect(
            clippy::literal_string_with_formatting_args,
            reason = "We need it for later"
        )]
        let mut renderer = new_renderer(
            if config.base.dry_run {
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
        )
        .or_raise(|| CleanError("new renderer".to_owned()))?;

        for (mailbox, result) in imap.list().or_raise(|| CleanError("list".to_owned()))? {
            match result.extra {
                Some(ref extra) => {
                    self.cleanup_mailbox(&mut imap, &mut renderer, &mailbox, extra)
                        .or_raise(|| CleanError("cleanup mailbox".to_owned()))?;
                }
                None => bail!(CleanError(format!(
                    "Mailbox {mailbox} does not have an extra parameter"
                ))),
            }
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, imap, renderer), err(level = "info"))
    )]
    fn cleanup_mailbox(
        &self,
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer>,
        mailbox: &str,
        extra: &MyExtra,
    ) -> Result<(), CleanError> {
        let mbx = imap
            .session
            .examine(mailbox)
            .or_raise(|| CleanError(format!("imap examine {mailbox:?} failed")))?;

        // If there are not enough messages, skip
        if mbx.exists <= 300 {
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
                .or_raise(|| CleanError("imap uid search failed".to_owned()))?; // Search messages by cutoff date

            // Only delete if the rule applies based on mailbox size and message age
            if total_size > rule_size.bytes() && !uids_to_delete.is_empty() {
                // Mark messages for deletion

                let sequence = ids_list_to_collapsed_sequence(&uids_to_delete);

                if !self.config.dry_run {
                    imap.session
                        .select(mailbox)
                        .or_raise(|| CleanError(format!("imap select {mailbox:?} failed")))?;

                    imap.session
                        .uid_store(&sequence, "+FLAGS (\\Deleted)")
                        .or_raise(|| CleanError("imap uid store failed".to_owned()))?;

                    // Expunge to permanently remove messages marked for deletion
                    imap.session
                        .close()
                        .or_raise(|| CleanError("imap close failed".to_owned()))?;
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
