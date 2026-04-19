mod csv;
#[cfg(feature = "json")]
mod json;
mod print;
#[cfg(feature = "ratatui")]
mod terminal;
mod traits;

use exn::{Exn, Result, ResultExt as _, bail};
use serde::{Deserialize, Serialize};

pub use crate::libs::render::traits::Renderer;
use crate::libs::render::traits::RendererError;
#[cfg(feature = "ratatui")]
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
    #[cfg(feature = "json")]
    /// Json output
    Json,
}

#[allow(clippy::derivable_impls, reason = "special cases")]
impl Default for RendererArg {
    fn default() -> Self {
        if cfg!(test) {
            return Self::Csv;
        }

        #[cfg(feature = "ratatui")]
        if terminal::Renderer::is_usable() {
            return Self::Ratatui;
        }

        Self::Terminal
    }
}

#[derive(Clone, Debug, derive_more::Display)]
pub enum RendererArgError {
    #[display("Unknown renderer {_0:?}")]
    Unknown(String),
}

impl std::error::Error for RendererArgError {}

impl std::str::FromStr for RendererArg {
    type Err = Exn<RendererArgError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "csv" => Ok(Self::Csv),
            #[cfg(feature = "ratatui")]
            "ratatui" => Ok(Self::Ratatui),
            "terminal" => Ok(Self::Terminal),
            s => bail!(RendererArgError::Unknown(s.to_owned())),
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
            csv::Renderer::new(title, format, headers)
                .or_raise(|| RendererError("csv".to_owned()))?,
        )),
        #[cfg(feature = "ratatui")]
        RendererArg::Ratatui => Ok(Box::new(
            terminal::Renderer::new(title, format, headers)
                .or_raise(|| RendererError("terminal".to_owned()))?,
        )),
        #[cfg(feature = "json")]
        RendererArg::Json => Ok(Box::new(
            json::Renderer::new(title, format, headers)
                .or_raise(|| RendererError("json".to_owned()))?,
        )),
        RendererArg::Terminal => Ok(Box::new(
            print::Renderer::new(title, format, headers)
                .or_raise(|| RendererError("print".to_owned()))?,
        )),
    }
}
