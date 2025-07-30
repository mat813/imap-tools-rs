use crate::libs::{args, base_config::BaseConfig, imap::Imap, render::new_renderer};
use anyhow::{bail, Context as _, Result};
use clap::Args;
use imap_proto::NameAttribute;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use size::Size;
use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub enum Sort {
    #[default]
    Name,
    Size,
}

impl FromStr for Sort {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "size" => Ok(Self::Size),
            "name" => Ok(Self::Name),
            _ => bail!("Invalid sort {s}"),
        }
    }
}

#[derive(Args, Debug, Clone)]
#[command(
    about = "List mailboxes",
    long_about = "This command allows to list mailboxes."
)]
pub struct DiskUsage {
    #[clap(flatten)]
    config: args::Generic,

    /// Only include folder paths matching this re
    #[arg(long)]
    pub include_re: Vec<Regex>,

    /// Exclude folder paths matching this re
    #[arg(long)]
    pub exclude_re: Vec<Regex>,

    /// sort results by `name` or `size`
    #[arg(long, default_value = "name")]
    pub sort: Sort,

    /// Show progress bar
    #[arg(long)]
    pub progress: bool,

    /// Imap pattern
    #[clap(default_value = Some("*"))]
    pattern: Option<String>,

    /// Imap reference list
    reference: Option<String>,
}

impl DiskUsage {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<()> {
        let config = BaseConfig::new(&self.config)?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> = Imap::connect_base(&config)?;

        #[expect(
            clippy::literal_string_with_formatting_args,
            reason = "We need it for later"
        )]
        let mut renderer = new_renderer("Mailbox Size", "{0:<42} {1}", &["Mailbox", "Attributes"])?;

        let mut result: Vec<(String, u64)> = vec![];

        let mailboxes = imap
            .session
            .list(self.reference.as_deref(), self.pattern.as_deref())
            .with_context(|| {
                format!(
                    "imap list failed with ref:{:?} and pattern:{:?}",
                    self.reference, self.pattern
                )
            })?;

        let mailboxes = mailboxes
            .iter()
            // Filter out folders that are marked as NoSelect, which are not mailboxes, only folders
            .filter(|mbx| !mbx.attributes().contains(&NameAttribute::NoSelect))
            // If we have an include regex, keep folders that match it
            // Otherwise, keep everything
            .filter(|mbx| {
                if self.include_re.is_empty() {
                    true
                } else {
                    self.include_re.iter().any(|re| re.is_match(mbx.name()))
                }
            })
            // If we have an exclude regex, filter out folders that match it
            // Otherwise, keep everything
            .filter(|mbx| {
                if self.exclude_re.is_empty() {
                    true
                } else {
                    self.exclude_re.iter().all(|re| !re.is_match(mbx.name()))
                }
            })
            .collect::<Vec<_>>();

        let len_mbox = u64::try_from(mailboxes.len())?;

        let bar = self.progress.then(|| ProgressBar::new(len_mbox));

        if let Some(ref b) = bar {
            b.set_style(
                ProgressStyle::with_template(
                    "[{elapsed_precise}/{duration_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                )?
                .progress_chars("##-"),
            );
        }

        for mailbox in mailboxes {
            if let Some(ref b) = bar {
                b.inc(1);
                b.set_message(mailbox.name().to_owned());
            }

            let mbx = imap.session.examine(mailbox.name())?;

            if mbx.exists == 0 {
                result.push((mailbox.name().to_owned(), 0));
                continue;
            }

            let total: u64 = imap
                .session
                .uid_fetch("1:*", "(RFC822.SIZE)")
                .context("imap uid fetch failed")?
                .iter()
                .map(|m| u64::from(m.size.unwrap_or(0)))
                .sum();

            result.push((mailbox.name().to_owned(), total));
        }

        if let Some(b) = bar {
            b.finish();
        }

        result.sort_by(|a, b| match self.sort {
            Sort::Name => a.0.cmp(&b.0),
            Sort::Size => a.1.cmp(&b.1),
        });

        for (mbx, total) in result {
            renderer.add_row(&[&mbx, &Size::from_bytes(total).format()])?;
        }

        Ok(())
    }
}
