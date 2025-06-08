#![warn(
    clippy::allow_attributes,
    clippy::as_ptr_cast_mut,
    clippy::as_underscore,
    clippy::assigning_clones,
    clippy::borrow_as_ptr,
    clippy::branches_sharing_code,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::if_then_some_else_none,
    clippy::match_like_matches_macro,
    clippy::match_same_arms,
    clippy::nursery,
    clippy::option_as_ref_deref,
    clippy::needless_raw_strings,
    clippy::unneeded_field_pattern,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::pedantic
)]
mod commands;
mod libs;
mod run;

fn main() -> anyhow::Result<()> {
    run::run()
}
