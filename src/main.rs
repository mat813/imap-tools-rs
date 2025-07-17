#![allow(clippy::missing_docs_in_private_items, reason = "TODO: docs")]
#![allow(clippy::todo, reason = "TODO: fixup last todos")]
mod commands;
mod libs;
mod run;

fn main() -> anyhow::Result<()> {
    run::run()
}
