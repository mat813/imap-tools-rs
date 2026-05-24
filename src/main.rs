#![doc = include_str!("../README.md")]
#![allow(clippy::missing_docs_in_private_items, reason = "TODO: docs")]
#![allow(clippy::todo, reason = "TODO: fixup last todos")]
// We use tokio current_thread runtime, so futures don't need to be Send.
#![allow(clippy::future_not_send, reason = "current_thread runtime")]

use exn::{Result, ResultExt as _};
mod commands;
mod libs;
mod run;
#[cfg(test)]
mod test_helpers;

#[derive(Debug, derive_more::Display)]
enum MainError {
    #[display("opening log file {file:?}")]
    #[cfg(feature = "tracing")]
    Tracing { file: String },
    #[display("Running CLI")]
    Run,
}

impl std::error::Error for MainError {}

#[tokio::main]
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", err(level = "info"))
)]
async fn main() -> Result<(), MainError> {
    #[cfg(feature = "tracing")]
    {
        use tracing_subscriber::{
            Layer as _,
            fmt::{format::FmtSpan, writer::BoxMakeWriter},
            layer::SubscriberExt as _,
            util::SubscriberInitExt as _,
        };

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(
                        if let Some(file) = option_env!("LOG_OUTPUT") {
                            BoxMakeWriter::new(std::fs::File::create(file).or_raise(|| {
                                MainError::Tracing {
                                    file: file.to_owned(),
                                }
                            })?)
                        } else {
                            BoxMakeWriter::new(std::io::stdout)
                        },
                    )
                    .pretty()
                    .with_level(true)
                    .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
                    .with_ansi(true)
                    .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
            )
            .init();
    }

    run::run().await.or_raise(|| MainError::Run)
}
