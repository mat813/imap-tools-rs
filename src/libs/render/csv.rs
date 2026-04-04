use std::{fmt::Display, io::stdout};

use exn::{Result, ResultExt as _};

use crate::libs::render::{Renderer as RendererTrait, RendererError};

#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct Renderer {
    writer: csv::Writer<std::io::Stdout>,
}

impl RendererTrait for Renderer {
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
        headers: &[&'static str],
    ) -> Result<Self, RendererError> {
        let mut writer = csv::Writer::from_writer(stdout());
        writer
            .write_record(headers)
            .or_raise(|| RendererError("csv write headers failed".to_owned()))?;
        Ok(Self { writer })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, row), err(level = "info"))
    )]
    fn add_row(&mut self, row: &[&dyn Display]) -> Result<(), RendererError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(row = ?row.iter().map(std::string::ToString::to_string).collect::<Vec<_>>());

        let record: Vec<String> = row.iter().map(std::string::ToString::to_string).collect();
        self.writer
            .write_record(&record)
            .or_raise(|| RendererError("csv write record failed".to_owned()))?;
        Ok(())
    }
}
