//! Build script: makes cargo rebuild the crate when a translation changes.
//!
//! `rust_i18n::i18n!` embeds `locales/` at compile time but does not tell
//! cargo to track those files, so without this an edit to `locales/app.yml`
//! is silently ignored by incremental builds and the binary keeps serving the
//! old text.

/// Registers the `locales/` directory as a build input.
fn main() {
    println!("cargo::rerun-if-changed=locales");
}
