use crate::libs::render::Renderer as RendererTrait;
use anyhow::{Context as _, Result};
use std::{collections::HashMap, fmt::Display};
use strfmt::strfmt;

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
    fn new(_title: &'static str, format: &'static str, headers: &[&'static str]) -> Result<Self> {
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
    fn add_row(&mut self, row: &[&dyn Display]) -> Result<()> {
        #[cfg(feature = "tracing")]
        tracing::trace!(row = ?row.iter().map(|r| format!("{r}")).collect::<Vec<String>>());

        if !self.some_output {
            self.some_output = true;

            let map: HashMap<String, String> = self
                .headers
                .iter()
                .enumerate()
                .map(|(idx, f)| (idx.to_string(), (*f).to_owned()))
                .collect();
            let output = strfmt(self.format, &map)
                .with_context(|| format!("strfmt failed {:?} {:?}", self.format, map))?;
            println!("{output}");
        }
        let map: HashMap<String, String> = row
            .iter()
            .enumerate()
            .map(|(idx, f)| (idx.to_string(), f.to_string()))
            .collect();
        let output = strfmt(self.format, &map)
            .with_context(|| format!("strfmt failed {:?} {:?}", self.format, map))?;
        println!("{output}");

        Ok(())
    }
}
