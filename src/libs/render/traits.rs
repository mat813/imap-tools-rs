use std::fmt::Display;

use derive_more::Display;
use exn::Result;

#[derive(Debug, Display)]
pub struct RendererError(pub String);
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
