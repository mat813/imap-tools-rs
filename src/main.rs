#![doc = include_str!("../README.md")]
#![allow(clippy::missing_docs_in_private_items, reason = "TODO: docs")]

use exn::{Result, ResultExt as _};
use rust_i18n::t;
mod commands;
mod libs;
mod run;
#[cfg(test)]
mod test_helpers;

rust_i18n::i18n!("locales", fallback = "en");

#[derive(Debug, derive_more::Display)]
enum MainError {
    #[display("{}", t!("error.main.tracing", file = file : {:?}))]
    #[cfg(feature = "tracing")]
    Tracing { file: String },
    #[display("{}", t!("error.main.run"))]
    Run,
}

impl std::error::Error for MainError {}

#[tokio::main]
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", err(level = "info"))
)]
async fn main() -> Result<(), MainError> {
    libs::i18n::init();

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
