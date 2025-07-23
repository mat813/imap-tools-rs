mod archive;
mod clean;
#[expect(clippy::module_inception, reason = "ok")]
mod commands;
mod find_dups;
mod imap;
mod list;
pub use commands::Commands;
