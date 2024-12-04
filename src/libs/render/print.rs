use crate::libs::render::Renderer as RendererTrait;
use anyhow::{Context, Result};
use std::{collections::HashMap, fmt::Display};
use strfmt::strfmt;

pub struct Renderer<'a> {
    format: &'a str,
    headers: Vec<&'a str>,
    some_output: bool,
}

impl RendererTrait for Renderer<'_> {
    fn new(_title: &'static str, format: &'static str, headers: &[&'static str]) -> Result<Self> {
        Ok(Self {
            format,
            headers: headers.into(),
            some_output: false,
        })
    }

    fn add_row(&mut self, row: &[&dyn Display]) -> Result<()> {
        if !self.some_output {
            self.some_output = true;

            let map: HashMap<String, String> = self
                .headers
                .iter()
                .enumerate()
                .map(|(idx, f)| (idx.to_string(), (*f).to_string()))
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
