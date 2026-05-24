mod csv;
#[cfg(feature = "cursive")]
mod cursive;
mod json;
mod print;
#[cfg(feature = "ratatui")]
mod terminal;
mod traits;

use exn::{Exn, Result, ResultExt as _, bail};
use serde::{Deserialize, Serialize};

pub use crate::libs::render::traits::Renderer;
use crate::libs::render::traits::RendererError;
#[cfg(any(feature = "ratatui", feature = "cursive"))]
use crate::libs::render::traits::RendererUsable as _;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, derive_more::Display, clap::ValueEnum)]
pub enum RendererArg {
    /// CSV output
    Csv,
    /// Table-ish output
    Terminal,
    #[cfg(feature = "ratatui")]
    /// Ratatui-TUI output
    Ratatui,
    /// Json output
    Json,
    #[cfg(feature = "cursive")]
    /// Interactive TUI output (cursive)
    Cursive,
}

#[allow(clippy::derivable_impls, reason = "special cases")]
impl Default for RendererArg {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip()))]
    fn default() -> Self {
        if cfg!(test) {
            return Self::Csv;
        }

        #[cfg(feature = "cursive")]
        if cursive::CursiveRenderer::is_usable() {
            return Self::Cursive;
        }

        #[cfg(feature = "ratatui")]
        if terminal::TerminalRenderer::is_usable() {
            return Self::Ratatui;
        }

        Self::Terminal
    }
}

#[derive(Clone, Debug, derive_more::Display)]
pub enum RendererArgError {
    #[display("Unknown renderer {renderer:?}")]
    Unknown { renderer: String },
}

impl std::error::Error for RendererArgError {}

impl std::str::FromStr for RendererArg {
    type Err = Exn<RendererArgError>;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(s), ret, err(level = "debug"))
    )]
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "csv" => Ok(Self::Csv),
            #[cfg(feature = "ratatui")]
            "ratatui" => Ok(Self::Ratatui),
            "terminal" => Ok(Self::Terminal),
            #[cfg(feature = "cursive")]
            "cursive" => Ok(Self::Cursive),
            s => bail!(RendererArgError::Unknown {
                renderer: s.to_owned()
            }),
        }
    }
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", skip(title, format, headers), err(level = "info"))
)]
pub fn new_renderer<const N: usize>(
    renderer: Option<RendererArg>,
    title: &'static str,
    format: &'static [&'static str; N],
    headers: &'static [&'static str; N],
) -> Result<Box<dyn Renderer<N>>, RendererError> {
    match renderer.unwrap_or_default() {
        RendererArg::Csv => Ok(Box::new(
            csv::CsvRenderer::new(title, format, headers).or_raise(|| RendererError::Csv)?,
        )),
        #[cfg(feature = "ratatui")]
        RendererArg::Ratatui => Ok(Box::new(
            terminal::TerminalRenderer::new(title, format, headers)
                .or_raise(|| RendererError::Terminal)?,
        )),
        RendererArg::Json => Ok(Box::new(
            json::JsonRenderer::new(title, format, headers).or_raise(|| RendererError::Json)?,
        )),
        RendererArg::Terminal => Ok(Box::new(
            print::PrintRenderer::new(title, format, headers).or_raise(|| RendererError::Print)?,
        )),
        #[cfg(feature = "cursive")]
        RendererArg::Cursive => Ok(Box::new(
            cursive::CursiveRenderer::new(title, format, headers)
                .or_raise(|| RendererError::Cursive)?,
        )),
    }
}
