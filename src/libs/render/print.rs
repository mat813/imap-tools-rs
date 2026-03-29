use std::{collections::HashMap, fmt::Display};

use exn::{Result, ResultExt as _};
use strfmt::strfmt;

use crate::libs::render::{Renderer as RendererTrait, RendererError};

#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct Renderer<'a> {
    format: &'a str,
    headers: Vec<&'a str>,
    some_output: bool,
}

impl RendererTrait for Renderer<'_> {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "trace",
            skip(_title, format, headers),
            ret,
            err(level = "info")
        )
    )]
    fn new(
        _title: &'static str,
        format: &'static str,
        headers: &[&'static str],
    ) -> Result<Self, RendererError> {
        Ok(Self {
            format,
            headers: headers.into(),
            some_output: false,
        })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, row), err(level = "info"))
    )]
    #[expect(clippy::print_stdout, reason = "we print")]
    fn add_row(&mut self, row: &[&dyn Display]) -> Result<(), RendererError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(row = ?row.iter().map(std::string::ToString::to_string).collect::<Vec<_>>());

        if !self.some_output {
            self.some_output = true;

            let map: HashMap<_, _> = self
                .headers
                .iter()
                .enumerate()
                .map(|(idx, f)| (idx, *f))
                .collect();
            let output = strfmt(self.format, &map)
                .or_raise(|| RendererError(format!("strfmt failed {:?} {:?}", self.format, map)))?;
            println!("{output}");
        }
        let map: HashMap<_, _> = row
            .iter()
            .enumerate()
            .map(|(idx, f)| (idx, f.to_string()))
            .collect();
        let output = strfmt(self.format, &map)
            .or_raise(|| RendererError(format!("strfmt failed {:?} {:?}", self.format, map)))?;
        println!("{output}");

        Ok(())
    }
}
