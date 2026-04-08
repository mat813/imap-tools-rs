use std::fmt::Display;

use derive_more::Display;
use exn::{Exn, Result, ResultExt as _, bail};
use serde::{Deserialize, Serialize};
mod csv;
mod print;
#[cfg(feature = "ratatui")]
mod terminal;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, derive_more::Display, clap::ValueEnum)]
pub enum RendererArg {
    /// CSV output
    Csv,
    /// Table-ish output
    Terminal,
    #[cfg(feature = "ratatui")]
    /// Ratatui-TUI output
    Ratatui,
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

#[derive(Debug, Display)]
pub struct RendererError(String);
impl std::error::Error for RendererError {}

pub trait Renderer {
    #[allow(dead_code, reason = "when the feature is not enabled")]
    fn is_usable() -> bool
    where
        Self: Sized,
    {
        true
    }

    fn new(
        title: &'static str,
        format: &'static str,
        headers: &[&'static str],
    ) -> Result<Self, RendererError>
    where
        Self: Sized;
    fn add_row(&mut self, row: &[&dyn Display]) -> Result<(), RendererError>;

    /// Returns the accumulated output as a string, for renderers that buffer internally.
    /// Other renderers return an empty string.
    #[cfg(test)]
    fn output(&mut self) -> String {
        String::new()
    }
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", skip(title, format, headers), err(level = "info"))
)]
pub fn new_renderer(
    renderer: Option<RendererArg>,
    title: &'static str,
    format: &'static str,
    headers: &[&'static str],
) -> Result<Box<dyn Renderer>, RendererError> {
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
        RendererArg::Terminal => Ok(Box::new(
            print::Renderer::new(title, format, headers)
                .or_raise(|| RendererError("print".to_owned()))?,
        )),
    }
}
