use crate::libs::{
    args,
    config::Config,
    imap::{ids_list_to_collapsed_sequence, Imap},
    render::{new_renderer, Renderer},
};
use anyhow::{Context as _, Result};
use clap::Args;
use imap::types::{Fetches, Uid};
use regex::Regex;
use std::collections::{HashMap, HashSet};

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
    pub fn execute(&self) -> Result<()> {
        let config = Config::<MyExtra>::new(&self.config)?;

        let mut renderer = new_renderer(
            if config.base.dry_run {
                "Mailbox Deduplication DRY-RUN"
            } else {
                "Mailbox Deduplication"
            },
            "[{0}] {1} {2}",
            &["Mailbox", "Dups", "Sequence"],
        )?;

        let mut imap = Imap::connect(&config)?;

        for (mailbox, _result) in imap.list()? {
            self.process(&mut imap, &mut renderer, &mailbox)?;
        }

        Ok(())
    }

    fn process(
        &self,
        imap: &mut Imap<MyExtra>,
        renderer: &mut Box<dyn Renderer>,
        mailbox: &str,
    ) -> Result<()> {
        // Examine the mailbox in read only mode, so that we don't change any
        // "seen" flags if there are no duplicate messages
        let mbx = imap
            .session
            .examine(mailbox)
            .with_context(|| format!("imap examine {mailbox:?} failed"))?;

        // If there are less than 2 messages, there cannot possible be
        // duplicates, stop here
        if mbx.exists < 2 {
            return Ok(());
        }

        // Fetch message headers to find duplicates
        let messages = imap
            .session
            .uid_fetch("1:*", "(BODY.PEEK[HEADER.FIELDS (MESSAGE-ID)])")
            .context("imap uid fetch failed")?;
        let duplicates = Self::find_duplicates(&messages)?;

        // Delete duplicate messages
        if !duplicates.is_empty() {
            let duplicate_set = ids_list_to_collapsed_sequence(&duplicates);

            if !self.config.dry_run {
                // Re-open the mailbox in read-write mode
                imap.session
                    .select(mailbox)
                    .with_context(|| format!("imap select {mailbox:?} failed"))?;

                imap.session
                    .uid_store(&duplicate_set, "+FLAGS (\\Deleted)")
                    .context("imap uid store failed")?;

                imap.session.close().context("imap close failed")?;
            }

            renderer.add_row(&[&mailbox, &duplicates.len(), &duplicate_set])?;
        }

        Ok(())
    }

    fn find_duplicates(messages: &Fetches) -> Result<HashSet<Uid>> {
        let mut message_ids: HashMap<String, Vec<Uid>> = HashMap::new();

        // Collect message IDs with sequence numbers
        for message in messages.iter() {
            if let Some(id) = Self::parse_message_id(message.header()) {
                message_ids
                    .entry(id)
                    .or_default()
                    .push(message.uid.context("The server does not support the UIDPLUS capability, and all our operations need UIDs for safety")?);
            }
        }

        // Identify duplicates
        let mut duplicates = HashSet::<Uid>::new();

        for ids in message_ids.values() {
            if ids.len() > 1 {
                // Keep the first message and mark the rest as duplicates
                #[expect(clippy::indexing_slicing, reason = "we just tested it's ok")]
                duplicates.extend(&ids[1..]);
            }
        }

        Ok(duplicates)
    }

    // Parses a Message-ID from the header
    fn parse_message_id(header: Option<&[u8]>) -> Option<String> {
        let header_text = std::str::from_utf8(header?).ok()?;

        // Clean the input by replacing any line breaks followed by whitespace with a single space
        let cleaned_headers = header_text; //.replace("\r\n ", " ").replace("\n ", " ");

        // Find and capture the Message-ID using the regex
        let s = MESSAGE_ID_REGEX
            .captures(cleaned_headers)?
            .get(1)
            .map(|m| m.as_str().to_owned())?;
        // If the length of the message id is too short, say it's None

        (s.len() > 4).then_some(s)
    }
}
