use std::fmt::Display;

use exn::Result;
use serde_json::Value;

use crate::libs::render::traits::{Renderer as RendererTrait, RendererError, RendererUsable};

/// CSV renderer that buffers output to an internal `Vec<u8>`.
/// Output is flushed to stdout on `Drop` (unless running in test mode).
#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct Renderer {
    headers: &'static [&'static str],
    json: Vec<Value>,
}

impl RendererUsable for Renderer {}

impl<const N: usize> RendererTrait<N> for Renderer {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "trace",
            skip(_title, _format, headers),
            ret,
            err(level = "info")
        )
    )]
    fn new(
        _title: &'static str,
        _format: &'static str,
        headers: &'static [&'static str; N],
    ) -> Result<Self, RendererError> {
        Ok(Self {
            headers,
            json: vec![],
        })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, row), err(level = "info"))
    )]
    fn add_row(&mut self, row: &[&dyn Display; N]) -> Result<(), RendererError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(row = ?row.iter().map(std::string::ToString::to_string).collect::<Vec<_>>());

        self.json.push(Value::Object(
            self.headers
                .iter()
                .zip(row)
                .map(|(h, v)| (h.to_string(), Value::String(v.to_string())))
                .collect(),
        ));

        Ok(())
    }

    #[cfg(test)]
    fn output(&mut self) -> String {
        Value::Array(self.json.clone()).to_string()
    }
}

impl Drop for Renderer {
    #[expect(clippy::print_stdout, reason = "we print")]
    fn drop(&mut self) {
        if !cfg!(test) {
            print!("{}", Value::Array(self.json.clone()));
        }
    }
}
