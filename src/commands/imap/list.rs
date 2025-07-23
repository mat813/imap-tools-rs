use crate::libs::{args, base_config::BaseConfig, imap::Imap, render::new_renderer};
use anyhow::{Context as _, Result};
use clap::Args;
use imap_proto::NameAttribute;
use regex::Regex;

#[derive(Args, Debug, Clone)]
#[command(
    about = "List mailboxes",
    long_about = "This command allows to list mailboxes."
)]
pub struct List {
    #[clap(flatten)]
    config: args::Generic,

    /// Only include folder paths matching this re
    #[arg(long)]
    pub include_re: Vec<Regex>,

    /// Exclude folder paths matching this re
    #[arg(long)]
    pub exclude_re: Vec<Regex>,

    /// Include `NoSelect` "folders"
    #[arg(long)]
    pub no_select: bool,

    /// Imap pattern
    #[clap(default_value = Some("*"))]
    pattern: Option<String>,

    /// Imap reference list
    reference: Option<String>,
}

impl List {
    pub fn execute(&self) -> Result<()> {
        let config = BaseConfig::new_with_args(&self.config)?;

        let mut imap: Imap<()> = Imap::connect_base(&config)?;

        #[expect(
            clippy::literal_string_with_formatting_args,
            reason = "We need it for later"
        )]
        let mut renderer = new_renderer("Mailbox List", "{0:<42} {1}", &["Mailbox", "Attributes"])?;

        for mailbox in imap
            .session
            .list(self.reference.as_deref(), self.pattern.as_deref())
            .with_context(|| {
                format!(
                    "imap list failed with ref:{:?} and pattern:{:?}",
                    self.reference, self.pattern
                )
            })?
            .iter()
            // Filter out folders that are marked as NoSelect, which are not mailboxes, only folders
            .filter(|mbx| self.no_select || !mbx.attributes().contains(&NameAttribute::NoSelect))
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
        {
            renderer.add_row(&[&mailbox.name(), &format!("{:?}", mailbox.attributes())])?;
        }

        Ok(())
    }
}
