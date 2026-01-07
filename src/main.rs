#![allow(clippy::missing_docs_in_private_items, reason = "TODO: docs")]
#![allow(clippy::todo, reason = "TODO: fixup last todos")]

use derive_more::Display;
use exn::{Result, ResultExt as _};
mod commands;
mod libs;
mod run;

#[derive(Debug, Display)]
struct MainError;

impl std::error::Error for MainError {}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", err(level = "info"))
)]
fn main() -> Result<(), MainError> {
    #[cfg(feature = "tracing")]
    {
        use tracing_subscriber::{
            fmt::format::FmtSpan, layer::SubscriberExt as _, util::SubscriberInitExt as _,
            Layer as _,
        };

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .pretty()
                    // .json()
                    .with_level(true)
                    .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
                    .with_ansi(true)
                    .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
            )
            .init();
    }

    run::run().or_raise(|| MainError)
}
