#![allow(clippy::missing_docs_in_private_items, reason = "TODO: docs")]
#![allow(clippy::todo, reason = "TODO: fixup last todos")]
// We use tokio current_thread runtime, so futures don't need to be Send.
#![allow(clippy::future_not_send, reason = "current_thread runtime")]

use derive_more::Display;
use exn::{Result, ResultExt as _};
mod commands;
mod libs;
mod run;
#[cfg(test)]
mod test_helpers;

#[derive(Debug, Display)]
struct MainError;

impl std::error::Error for MainError {}

#[tokio::main(flavor = "current_thread")]
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", err(level = "info"))
)]
async fn main() -> Result<(), MainError> {
    #[cfg(feature = "tracing")]
    {
        use tracing_subscriber::{
            Layer as _, fmt::format::FmtSpan, layer::SubscriberExt as _,
            util::SubscriberInitExt as _,
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

    run::run().await.or_raise(|| MainError)
}
