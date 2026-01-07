use derive_more::Display;
use exn::{Result, ResultExt as _};
use std::fmt::Display;
mod print;
#[cfg(feature = "ratatui")]
mod terminal;

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
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", skip(title, format, headers), err(level = "info"))
)]
pub fn new_renderer(
    title: &'static str,
    format: &'static str,
    headers: &[&'static str],
) -> Result<Box<dyn Renderer>, RendererError> {
    #[cfg(feature = "ratatui")]
    if terminal::Renderer::is_usable() && option_env!("RENDERER").is_none_or(|v| v == "ratatui") {
        return Ok(Box::new(
            terminal::Renderer::new(title, format, headers)
                .or_raise(|| RendererError("terminal".to_owned()))?,
        ));
    }

    Ok(Box::new(
        print::Renderer::new(title, format, headers)
            .or_raise(|| RendererError("print".to_owned()))?,
    ))
}
