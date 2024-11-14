use crate::libs::{error::OurResult, render::Renderer as RendererTrait};
use std::{collections::HashMap, fmt::Display};
use strfmt::strfmt;

pub struct Renderer<'a> {
    format: &'a str,
    headers: Vec<&'a str>,
    some_output: bool,
}

impl<'a> RendererTrait for Renderer<'a> {
    fn new(
        _title: &'static str,
        format: &'static str,
        headers: &[&'static str],
    ) -> OurResult<Self> {
        Ok(Self {
            format,
            headers: headers.into(),
            some_output: false,
        })
    }

    fn add_row(&mut self, row: &[&dyn Display]) -> OurResult<()> {
        if !self.some_output {
            self.some_output = true;

            let map: HashMap<String, String> = self
                .headers
                .iter()
                .enumerate()
                .map(|(idx, f)| (idx.to_string(), (*f).to_string()))
                .collect();
            let output = strfmt(self.format, &map)?;
            println!("{output}");
        }
        let map: HashMap<String, String> = row
            .iter()
            .enumerate()
            .map(|(idx, f)| (idx.to_string(), f.to_string()))
            .collect();
        let output = strfmt(self.format, &map)?;
        println!("{output}");

        Ok(())
    }
}
