mod csv;
#[cfg(feature = "cursive")]
mod cursive;
mod json;
mod print;
#[cfg(feature = "ratatui")]
mod terminal;
mod traits;

use exn::{Result, ResultExt as _};
use rust_i18n::t;
use serde::Deserialize;

use crate::libs::render::traits::RendererError;
pub use crate::libs::render::traits::{Renderer, TableSpec};

#[derive(Clone, Copy, Debug, Deserialize, derive_more::Display, clap::ValueEnum)]
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

        #[cfg(any(feature = "cursive", feature = "ratatui"))]
        if std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            #[cfg(feature = "cursive")]
            return Self::Cursive;
            #[cfg(all(feature = "ratatui", not(feature = "cursive")))]
            return Self::Ratatui;
        }

        Self::Terminal
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
