use std::fmt::Display;

use exn::Result;

#[derive(Debug, derive_more::Display)]
pub enum RendererError {
    // Generic
    #[display("formatting {format:?} for {display:?}")]
    Strfmt {
        format: String,
        display: Box<dyn std::fmt::Debug + Send + Sync>,
    },

    // CSV specific
    #[display("Writing CSV output")]
    Csv,
    #[display("Writing CSV headers")]
    CsvWriteHeaders,
    #[display("Writing CSV record")]
    CsvWriteRecord,

    // Cursive specific
    #[display("Running cursive renderer")]
    #[cfg(feature = "cursive")]
    Cursive,
    #[display("renderer requires a terminal")]
    #[cfg(feature = "cursive")]
    CursiveRequireTerminal,
    #[display("Initializing cursive backend")]
    #[cfg(feature = "cursive")]
    CursiveBackendInit,
    #[display("rendering interrupted by user")]
    #[cfg(feature = "cursive")]
    CursiveInterrupted,

    // JSON specific
    #[display("Serializing output to JSON")]
    Json,

    // Print specific
    #[display("Printing output")]
    Print,

    // Terminal specific
    #[display("Running ratatui renderer")]
    #[cfg(feature = "ratatui")]
    Terminal,
    #[display("Initializing ratatui terminal")]
    #[cfg(feature = "ratatui")]
    TerminalInit,
    #[display("Clearing terminal")]
    #[cfg(feature = "ratatui")]
    TerminalClear,
    #[display("Drawing terminal frame")]
    #[cfg(feature = "ratatui")]
    TerminalDraw,
    #[display("Converting {width} to u16")]
    #[cfg(feature = "ratatui")]
    TerminalU16 { width: usize },
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
