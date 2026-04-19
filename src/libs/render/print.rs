use std::{collections::HashMap, fmt::Display};

use exn::{Result, ResultExt as _};
use strfmt::strfmt;

use crate::libs::render::traits::{Renderer as RendererTrait, RendererError, RendererUsable};

#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct Renderer {
    format: String,
    headers: &'static [&'static str],
    some_output: bool,
    buffer: String,
}

impl RendererUsable for Renderer {}

impl<const N: usize> RendererTrait<N> for Renderer {
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
        format: &'static [&'static str; N],
        headers: &'static [&'static str; N],
    ) -> Result<Self, RendererError> {
        Ok(Self {
            format: format
                .iter()
                .enumerate()
                .map(|(k, v)| format!("{{{k}{v}}}"))
                .collect::<Vec<_>>()
                .join(" | "),
            headers,
            some_output: false,
            buffer: String::new(),
        })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, row), err(level = "info"))
    )]
    fn add_row(&mut self, row: &[&dyn Display; N]) -> Result<(), RendererError> {
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
            let output = strfmt(&self.format, &map)
                .or_raise(|| RendererError(format!("strfmt failed {:?} {:?}", self.format, map)))?;
            self.buffer.push_str(&output);
            self.buffer.push('\n');
        }
        let map: HashMap<_, _> = row
            .iter()
            .enumerate()
            .map(|(idx, f)| (idx, f.to_string()))
            .collect();
        let output = strfmt(&self.format, &map)
            .or_raise(|| RendererError(format!("strfmt failed {:?} {:?}", self.format, map)))?;
        self.buffer.push_str(&output);
        self.buffer.push('\n');

        Ok(())
    }

    #[cfg(test)]
    fn output(&mut self) -> String {
        self.buffer.clone()
    }
}

impl Drop for Renderer {
    #[expect(clippy::print_stdout, reason = "we print")]
    fn drop(&mut self) {
        if !cfg!(test) {
            print!("{}", self.buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use insta::assert_snapshot;

    use super::*;

    macro_rules! row {
        ($($e:expr),* $(,)?) => { [$(&$e as &dyn std::fmt::Display),*] };
    }

    fn make(
        format: &'static [&'static str; 2],
        headers: &'static [&'static str; 2],
    ) -> impl RendererTrait<2> {
        Renderer::new("T", format, headers).expect("new renderer")
    }

    #[test]
    fn print_empty() {
        let mut r = make(&["", ""], &["Name", "Value"]);
        assert_snapshot!(r.output(), @"");
    }

    #[test]
    fn print_writes_headers_on_first_row() {
        let mut r = make(&["", ""], &["Name", "Value"]);
        r.add_row(&row!["foo", "bar"]).expect("add_row");
        assert_snapshot!(r.output(), @"
        Name | Value
        foo | bar
        ");
    }

    #[test]
    fn print_multiple_rows_headers_once() {
        let mut r = make(&["", ""], &["Name", "Value"]);
        r.add_row(&row!["foo", "bar"]).expect("add_row");
        r.add_row(&row!["baz", "qux"]).expect("add_row");
        assert_snapshot!(r.output(), @"
        Name | Value
        foo | bar
        baz | qux
        ");
    }

    #[test]
    fn print_format_with_width_specifier() {
        let mut r = make(&[":<10", ""], &["Name", "Value"]);
        r.add_row(&row!["foo", "bar"]).expect("add_row");
        assert_snapshot!(r.output(), @"
        Name       | Value
        foo        | bar
        ");
    }
}
