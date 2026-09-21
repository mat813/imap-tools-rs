mod csv;
#[cfg(feature = "cursive")]
mod cursive;
mod json;
mod print;
#[cfg(feature = "ratatui")]
mod terminal;
mod traits;

use exn::{Exn, Result, ResultExt as _, bail};
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::libs::render::traits::RendererError;
#[cfg(any(feature = "ratatui", feature = "cursive"))]
use crate::libs::render::traits::RendererUsable as _;
pub use crate::libs::render::traits::{Renderer, TableSpec};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, derive_more::Display, clap::ValueEnum)]
pub enum RendererArg {
    /// CSV output
    #[value(help = t!("cli.renderer.csv"))]
    Csv,
    /// Table-ish output
    #[value(help = t!("cli.renderer.terminal"))]
    Terminal,
    #[cfg(feature = "ratatui")]
    /// Ratatui TUI output
    #[value(help = t!("cli.renderer.ratatui"))]
    Ratatui,
    /// JSON output
    #[value(help = t!("cli.renderer.json"))]
    Json,
    #[cfg(feature = "cursive")]
    /// Interactive TUI output (cursive)
    #[value(help = t!("cli.renderer.cursive"))]
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
    #[display("{}", t!("error.renderer_arg.unknown", renderer = renderer : {:?}))]
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
    tracing::instrument(level = "trace", skip(spec), err(level = "info"))
)]
pub fn new_renderer<const N: usize>(
    renderer: Option<RendererArg>,
    spec: TableSpec<N>,
) -> Result<Box<dyn Renderer<N> + Send>, RendererError> {
    match renderer.unwrap_or_default() {
        RendererArg::Csv => Ok(Box::new(
            csv::CsvRenderer::new(spec).or_raise(|| RendererError::Csv)?,
        )),
        #[cfg(feature = "ratatui")]
        RendererArg::Ratatui => Ok(Box::new(
            terminal::TerminalRenderer::new(spec).or_raise(|| RendererError::Terminal)?,
        )),
        RendererArg::Json => Ok(Box::new(
            json::JsonRenderer::new(spec).or_raise(|| RendererError::Json)?,
        )),
        RendererArg::Terminal => Ok(Box::new(
            print::PrintRenderer::new(spec).or_raise(|| RendererError::Print)?,
        )),
        #[cfg(feature = "cursive")]
        RendererArg::Cursive => Ok(Box::new(
            cursive::CursiveRenderer::new(spec).or_raise(|| RendererError::Cursive)?,
        )),
    }
}
