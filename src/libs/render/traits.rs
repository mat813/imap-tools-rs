use std::{borrow::Cow, fmt::Display};

use exn::Result;
use rust_i18n::t;

#[derive(Debug, derive_more::Display)]
pub enum RendererError {
    // Generic
    #[display("{}", t!("error.renderer.strfmt", format = format : {:?}, display = display : {:?}))]
    Strfmt {
        format: String,
        display: Box<dyn std::fmt::Debug + Send + Sync>,
    },

    // CSV specific
    #[display("{}", t!("error.renderer.csv"))]
    Csv,
    #[display("{}", t!("error.renderer.csv_write_headers"))]
    CsvWriteHeaders,
    #[display("{}", t!("error.renderer.csv_write_record"))]
    CsvWriteRecord,

    // Cursive specific
    #[display("{}", t!("error.renderer.cursive"))]
    #[cfg(feature = "cursive")]
    Cursive,
    #[display("{}", t!("error.renderer.cursive_require_terminal"))]
    #[cfg(feature = "cursive")]
    CursiveRequireTerminal,
    #[display("{}", t!("error.renderer.cursive_backend_init"))]
    #[cfg(feature = "cursive")]
    CursiveBackendInit,
    #[display("{}", t!("error.renderer.cursive_interrupted"))]
    #[cfg(feature = "cursive")]
    CursiveInterrupted,

    // JSON specific
    #[display("{}", t!("error.renderer.json"))]
    Json,

    // Print specific
    #[display("{}", t!("error.renderer.print"))]
    Print,

    // Terminal specific
    #[display("{}", t!("error.renderer.terminal"))]
    #[cfg(feature = "ratatui")]
    Terminal,
    #[display("{}", t!("error.renderer.terminal_init"))]
    #[cfg(feature = "ratatui")]
    TerminalInit,
    #[display("{}", t!("error.renderer.terminal_clear"))]
    #[cfg(feature = "ratatui")]
    TerminalClear,
    #[display("{}", t!("error.renderer.terminal_draw"))]
    #[cfg(feature = "ratatui")]
    TerminalDraw,
    #[display("{}", t!("error.renderer.terminal_u16", width = width))]
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

/// What a renderer draws: a title, and `N` columns.
///
/// The machine-readable renderers (CSV, JSON) use the stable, English `keys`, so
/// that their output does not depend on the language. The renderers meant for a
/// human use the translated `title` and `labels`.
#[derive(Debug)]
pub struct TableSpec<const N: usize> {
    /// Translated title of the table.
    #[allow(dead_code, reason = "only shown by the optional TUI renderers")]
    pub title: String,
    /// Width and alignment of each column, as a `strfmt` specification.
    pub format: &'static [&'static str; N],
    /// Stable, untranslated name of each column.
    pub keys: &'static [&'static str; N],
    /// Translated header of each column.
    pub labels: [String; N],
}

impl<const N: usize> TableSpec<N> {
    pub fn new(
        title: Cow<'_, str>,
        format: &'static [&'static str; N],
        keys: &'static [&'static str; N],
        labels: [Cow<'_, str>; N],
    ) -> Self {
        Self {
            title: title.into_owned(),
            format,
            keys,
            labels: labels.map(Cow::into_owned),
        }
    }

    /// A table whose labels are its keys, for tests that do not care about translations.
    #[cfg(test)]
    pub fn untranslated(
        title: &str,
        format: &'static [&'static str; N],
        keys: &'static [&'static str; N],
    ) -> Self {
        Self {
            title: title.to_owned(),
            format,
            keys,
            labels: (*keys).map(str::to_owned),
        }
    }
}

pub trait Renderer<const N: usize>: RendererUsable {
    fn new(spec: TableSpec<N>) -> Result<Self, RendererError>
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
