//! Build script: it makes cargo rebuild the crate when a translation changes.
//!
//! `rust_i18n::i18n!` embeds `locales/` at compile time. It does not tell
//! cargo to track those files. Without this script, incremental builds ignore
//! an edit to `locales/app.yml` without a warning. The binary keeps the old
//! text.

/// Register the `locales/` directory as a build input.
fn main() {
    println!("cargo::rerun-if-changed=locales");
}
