mod archive;
mod clean;
#[expect(clippy::module_inception)]
mod commands;
mod find_dups;
mod list;
pub use commands::Commands;
