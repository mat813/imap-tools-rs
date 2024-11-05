use crate::libs::{
    args,
    config::Config,
    error::OurError,
    imap::{ids_list_to_collapsed_sequence, Imap},
};
use clap::Args;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
};

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

impl FindDups {
    pub fn execute(&self) -> Result<(), OurError> {
        let config = Config::<MyExtra>::new_with_args(&self.config)?;

        let mut imap = Imap::connect(&config)?;

        for (mailbox, _result) in imap.list()? {
            self.process(&mut imap, &mailbox)?;
        }

        Ok(())
    }

    fn process(&self, imap: &mut Imap<MyExtra>, mailbox: &str) -> Result<(), OurError> {
        print!("[{mailbox}] ");
        io::stdout().flush().unwrap(); // Ensure immediate print to terminal

        // Examine the mailbox in read only mode, so that we don't change any
        // "seen" flags if there are no duplicate messages
        let mbx = imap.session.examine(mailbox)?;

        // If there are less than 2 messages, there cannot possible be
        // duplicates, stop here
        if mbx.exists < 2 {
            return Ok(());
        }

        // Fetch message headers to find duplicates
        let messages = imap
            .session
            .uid_fetch("1:*", "(BODY[HEADER.FIELDS (MESSAGE-ID)])")?;
        let duplicates = find_duplicates(&messages);

        // Delete duplicate messages
        if duplicates.is_empty() {
            print!("\r\x1B[2K"); // `\x1B[2K` clears the entire line
            io::stdout().flush().unwrap(); // Ensure immediate print to terminal
        } else {
            let duplicate_set = ids_list_to_collapsed_sequence(&duplicates);

            if self.config.dry_run {
                println!("dry: {} {duplicate_set}", duplicates.len());
            } else {
                // Re-open the mailbox in read-write mode
                imap.session.select(mailbox)?;

                imap.session
                    .uid_store(&duplicate_set, "+FLAGS (\\Deleted)")?;

                imap.session.close()?;

                println!("{} {duplicate_set}", duplicates.len(),);
            }
        }

        Ok(())
    }
}

fn find_duplicates(messages: &imap::types::ZeroCopy<Vec<imap::types::Fetch>>) -> HashSet<u32> {
    let mut message_ids: HashMap<String, Vec<u32>> = HashMap::new();

    // Collect message IDs with sequence numbers
    for message in messages {
        if let Some(header) = message.header() {
            if let Ok(header_text) = std::str::from_utf8(header) {
                if let Some(id) = parse_message_id(header_text) {
                    message_ids
                        .entry(id)
                        .or_default()
                        .push(message.uid.unwrap());
                }
            }
        }
    }

    // Identify duplicates
    let mut duplicates = HashSet::<u32>::new();

    for ids in message_ids.values() {
        if ids.len() > 1 {
            // Keep the first message and mark the rest as duplicates
            duplicates.extend(&ids[1..]);
        }
    }

    duplicates
}

// Define the regex as a static global variable
static MESSAGE_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Regular expression to capture Message-ID values across line breaks
    Regex::new(r"(?i)Message-ID:\s*(<[^>]+>)").unwrap()
});

// Parses a Message-ID from the header
fn parse_message_id(header: &str) -> Option<String> {
    // Clean the input by replacing any line breaks followed by whitespace with a single space
    let cleaned_headers = header; //.replace("\r\n ", " ").replace("\n ", " ");

    // Find and capture the Message-ID using the regex
    MESSAGE_ID_REGEX
        .captures(cleaned_headers)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        // If the length of the message id is too short, say it's None
        .and_then(|s| (s.len() > 4).then_some(s))
}
