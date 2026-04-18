use std::fmt::Display;

use exn::{Result, ResultExt as _};

use crate::libs::render::traits::{Renderer as RendererTrait, RendererError, RendererUsable};

/// CSV renderer that buffers output to an internal `Vec<u8>`.
/// Output is flushed to stdout on `Drop` (unless running in test mode).
#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct Renderer {
    writer: csv::Writer<Vec<u8>>,
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
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer
            .write_record(headers)
            .or_raise(|| RendererError("csv write headers failed".to_owned()))?;
        Ok(Self { writer })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, row), err(level = "info"))
    )]
    fn add_row(&mut self, row: &[&dyn Display; N]) -> Result<(), RendererError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(row = ?row.iter().map(std::string::ToString::to_string).collect::<Vec<_>>());

        let record: Vec<String> = row.iter().map(std::string::ToString::to_string).collect();
        self.writer
            .write_record(&record)
            .or_raise(|| RendererError("csv write record failed".to_owned()))?;
        Ok(())
    }

    #[cfg(test)]
    fn output(&mut self) -> String {
        let _ = self.writer.flush();
        String::from_utf8_lossy(self.writer.get_ref()).into_owned()
    }
}

impl Drop for Renderer {
    #[expect(clippy::print_stdout, reason = "we print")]
    fn drop(&mut self) {
        if !cfg!(test) {
            let _ = self.writer.flush();
            print!("{}", String::from_utf8_lossy(self.writer.get_ref()));
        }
    }
}
