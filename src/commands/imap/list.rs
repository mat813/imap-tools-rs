use crate::libs::{args, base_config::BaseConfig, imap::Imap, render::new_renderer};
use clap::Args;
use derive_more::Display;
use exn::{Result, ResultExt as _};
use imap_proto::NameAttribute;
use regex::Regex;

#[derive(Debug, Display)]
pub struct ImapListCommandError(String);
impl std::error::Error for ImapListCommandError {}

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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub fn execute(&self) -> Result<(), ImapListCommandError> {
        let config =
            BaseConfig::new(&self.config).or_raise(|| ImapListCommandError("config".into()))?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?config);

        let mut imap: Imap<()> =
            Imap::connect_base(&config).or_raise(|| ImapListCommandError("connect".into()))?;

        #[expect(
            clippy::literal_string_with_formatting_args,
            reason = "We need it for later"
        )]
        let mut renderer = new_renderer("Mailbox List", "{0:<42} {1}", &["Mailbox", "Attributes"])
            .or_raise(|| ImapListCommandError("new renderer".to_owned()))?;

        for mailbox in imap
            .session
            .list(self.reference.as_deref(), self.pattern.as_deref())
            .or_raise(|| {
                ImapListCommandError(format!(
                    "imap list failed with ref:{:?} and pattern:{:?}",
                    self.reference, self.pattern
                ))
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
            renderer
                .add_row(&[&mailbox.name(), &format!("{:?}", mailbox.attributes())])
                .or_raise(|| ImapListCommandError("renderer add row".to_owned()))?;
        }

        Ok(())
    }
}
