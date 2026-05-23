use std::fmt::Display;

use exn::Result;

#[derive(Debug, derive_more::Display)]
pub enum RendererError {
    #[display("formatting {format:?} for {display:?}")]
    Strfmt {
        format: String,
        display: Box<dyn std::fmt::Debug + Send + Sync>,
    },

    // CSV specific
    Csv,
    #[display("write headers")]
    CsvWriteHeaders,
    #[display("write record")]
    CsvWriteRecord,
    // Cursive specific
    #[cfg(feature = "cursive")]
    Cursive,
    #[display("renderer requires a terminal")]
    #[cfg(feature = "cursive")]
    CursiveRequireTerminal,
    #[display("backend init")]
    #[cfg(feature = "cursive")]
    CursiveBackendInit,
    #[display("rendering interrupted by user")]
    #[cfg(feature = "cursive")]
    CursiveInterrupted,
    // JSON specific
    #[cfg(feature = "json")]
    Json,
    // Print specific
    Print,
    // Terminal specific
    #[cfg(feature = "ratatui")]
    Terminal,
    #[display("ratatui init")]
    #[cfg(feature = "ratatui")]
    TerminalInit,
    #[display("term clear")]
    #[cfg(feature = "ratatui")]
    TerminalClear,
    #[display("term draw")]
    #[cfg(feature = "ratatui")]
    TerminalDraw,
    #[display("converting {_0} in a u16")]
    #[cfg(feature = "ratatui")]
    TerminalU16(usize),
}
impl std::error::Error for RendererError {}

pub trait RendererUsable {
    #[allow(dead_code, reason = "when the feature is not enabled")]
    fn is_usable() -> bool
    where
        Self: Sized,
    {
        true
    }
}

pub trait Renderer<const N: usize>: RendererUsable {
    fn new(
        title: &'static str,
        format: &'static [&'static str; N],
        headers: &'static [&'static str; N],
    ) -> Result<Self, RendererError>
    where
        Self: Sized;
    fn add_row(&mut self, row: &[&dyn Display; N]) -> Result<(), RendererError>;

    /// Returns the accumulated output as a string, for renderers that buffer internally.
    /// Other renderers return an empty string.
    #[cfg(test)]
    fn output(&mut self) -> String {
        String::new()
    }
}
