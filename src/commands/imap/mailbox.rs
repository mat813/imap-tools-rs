//! What `imap create` and `imap delete` have in common.

use std::{borrow::Cow, io::Write};

use async_imap::error::Error as ImapProtoError;
use exn::{Result, ResultExt as _};
use rust_i18n::t;

use crate::libs::{args, base_config::BaseConfig, imap::Imap};

#[derive(Debug, derive_more::Display)]
pub enum MailboxCommandError {
    #[display("{}", t!("error.shared.config"))]
    Config,
    #[display("{}", t!("error.shared.connect"))]
    Connect,
    #[display("{}", t!("error.shared.imap_close"))]
    Close,
    #[display("{}", t!("error.imap_mailbox.run", command = command))]
    Run { command: &'static str },
    #[display("{}", t!("error.shared.write_output"))]
    WriteOutput,
    #[display("{}", t!("error.imap_create.imap_create", mailbox = mailbox))]
    Create { mailbox: String },
    #[display("{}", t!("error.imap_delete.imap_delete", mailbox = mailbox))]
    Delete { mailbox: String },
}
impl std::error::Error for MailboxCommandError {}

/// Connect, let `op` talk to the server, then log out.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", skip(op), err(level = "info"))
)]
pub async fn connect_and_run(
    args: &args::Generic,
    command: &'static str,
    op: impl AsyncFnOnce(&mut Imap<()>, &mut (dyn Write + Send)) -> Result<(), MailboxCommandError>,
) -> Result<(), MailboxCommandError> {
    let config = BaseConfig::new(args).or_raise(|| MailboxCommandError::Config)?;
    #[cfg(feature = "tracing")]
    tracing::trace!(?config);

    let mut imap: Imap<()> = Imap::connect_base(&config)
        .await
        .or_raise(|| MailboxCommandError::Connect)?;

    op(&mut imap, &mut std::io::stdout())
        .await
        .or_raise(|| MailboxCommandError::Run { command })?;

    imap.close().await.or_raise(|| MailboxCommandError::Close)
}

/// Print the outcome of a mailbox command: `success` when it worked, or
/// `tolerated(reason)` when the server answered NO with a `reason` containing
/// `tolerated_reason`. Any other error becomes `failure`.
pub fn report(
    out: &mut (dyn Write + Send),
    result: std::result::Result<(), ImapProtoError>,
    tolerated_reason: &str,
    success: Cow<'static, str>,
    tolerated: impl FnOnce(&str) -> Cow<'static, str>,
    failure: MailboxCommandError,
) -> Result<(), MailboxCommandError> {
    let message = match result {
        Ok(()) => success,
        Err(ImapProtoError::No(reason)) if reason.contains(tolerated_reason) => tolerated(&reason),
        Err(e) => return Err(e).or_raise(|| failure),
    };
    writeln!(out, "{message}").or_raise(|| MailboxCommandError::WriteOutput)
}
