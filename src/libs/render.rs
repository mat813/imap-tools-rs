use anyhow::Result;
use std::fmt::Display;
mod print;
#[cfg(feature = "ratatui")]
mod terminal;

pub trait Renderer {
    fn is_usable() -> bool
    where
        Self: Sized,
    {
        true
    }

    fn new(title: &'static str, format: &'static str, headers: &[&'static str]) -> Result<Self>
    where
        Self: Sized;
    fn add_row(&mut self, row: &[&dyn Display]) -> Result<()>;
}

pub fn new_renderer(
    title: &'static str,
    format: &'static str,
    headers: &[&'static str],
) -> Result<Box<dyn Renderer>> {
    #[cfg(feature = "ratatui")]
    if terminal::Renderer::is_usable() && option_env!("RENDERER").is_none_or(|v| v == "ratatui") {
        return Ok(Box::new(terminal::Renderer::new(title, format, headers)?));
    }

    Ok(Box::new(print::Renderer::new(title, format, headers)?))
}
