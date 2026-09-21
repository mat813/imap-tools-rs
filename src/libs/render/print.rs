use std::{collections::HashMap, fmt::Display};

use exn::{Result, ResultExt as _};
use strfmt::strfmt;
use unicode_width::UnicodeWidthStr as _;

use crate::libs::render::traits::{Renderer, RendererError, RendererUsable, TableSpec};

#[cfg_attr(feature = "tracing", derive(Debug))]
pub struct PrintRenderer {
    /// Format of the rows.
    format: String,
    /// Format of the header row, see [`column_formats`].
    header_format: String,
    headers: Vec<String>,
    some_output: bool,
    buffer: String,
}

/// Split a column specification such as `:<42` into its alignment (`:<`) and its width (`42`).
fn split_width(spec: &str) -> (&str, Option<usize>) {
    let (align, width) = spec.split_at(spec.trim_end_matches(|c: char| c.is_ascii_digit()).len());
    (align, width.parse().ok())
}

/// Build the `strfmt` fields of column `index` for its rows and for its header.
///
/// The width of a column is widened when its translated label does not fit. `strfmt`
/// pads by number of characters, while wide characters (CJK) take two terminal
/// cells: the header is therefore padded with fewer characters, so that its
/// columns line up with the ones of the rows.
fn column_formats(index: usize, spec: &str, label: &str) -> (String, String) {
    let (align, width) = split_width(spec);
    let Some(width) = width else {
        // No width, the cells are printed as is.
        let field = format!("{{{index}{spec}}}");
        return (field.clone(), field);
    };
    let width = width.max(label.width());
    let header_width = (width + label.chars().count()).saturating_sub(label.width());
    (
        format!("{{{index}{align}{width}}}"),
        format!("{{{index}{align}{header_width}}}"),
    )
}

impl RendererUsable for PrintRenderer {}

impl<const N: usize> Renderer<N> for PrintRenderer {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(spec), ret, err(level = "info"))
    )]
    fn new(spec: TableSpec<N>) -> Result<Self, RendererError> {
        let (format, header_format): (Vec<_>, Vec<_>) = spec
            .format
            .iter()
            .zip(&spec.labels)
            .enumerate()
            .map(|(index, (column, label))| column_formats(index, column, label))
            .unzip();
        Ok(Self {
            format: format.join(" | "),
            header_format: header_format.join(" | "),
            headers: spec.labels.into(),
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

            let map: HashMap<_, _> = self.headers.iter().cloned().enumerate().collect();
            let output = strfmt(&self.header_format, &map).or_raise(|| RendererError::Strfmt {
                format: self.header_format.clone(),
                display: Box::new(map),
            })?;
            self.buffer.push_str(&output);
            self.buffer.push('\n');
        }
        let map: HashMap<_, _> = row
            .iter()
            .enumerate()
            .map(|(idx, f)| (idx, f.to_string()))
            .collect();
        let output = strfmt(&self.format, &map).or_raise(|| RendererError::Strfmt {
            format: self.format.clone(),
            display: Box::new(map),
        })?;
        self.buffer.push_str(&output);
        self.buffer.push('\n');

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), ret)
    )]
    #[cfg(test)]
    fn output(&mut self) -> String {
        self.buffer.clone()
    }
}

impl Drop for PrintRenderer {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
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

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(format, headers))
    )]
    fn make(
        format: &'static [&'static str; 2],
        headers: &'static [&'static str; 2],
    ) -> impl Renderer<2> {
        make_spec(TableSpec::untranslated("T", format, headers))
    }

    fn make_spec(spec: TableSpec<2>) -> impl Renderer<2> {
        PrintRenderer::new(spec).expect("new renderer")
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

    #[test]
    fn print_uses_labels_not_keys() {
        let mut r = make_spec(TableSpec::new("T".into(), &["", ""], &["Name", "Value"], [
            "Nom".into(),
            "Valeur".into(),
        ]));
        r.add_row(&row!["foo", "bar"]).expect("add_row");
        assert_snapshot!(r.output(), @"
        Nom | Valeur
        foo | bar
        ");
    }

    #[test]
    fn print_widens_column_to_fit_label() {
        let mut r = make(&[":<3", ":>3"], &["Postfach", "Anzahl"]);
        r.add_row(&row!["foo", "12"]).expect("add_row");
        assert_snapshot!(r.output(), @"
        Postfach | Anzahl
        foo      |     12
        ");
    }

    #[test]
    fn print_aligns_wide_characters() {
        // Each of these labels takes two terminal cells per character.
        let mut r = make(&[":<10", ":>6"], &["名前", "数"]);
        r.add_row(&row!["foo", "12"]).expect("add_row");
        assert_snapshot!(r.output(), @"
        名前       |     数
        foo        |     12
        ");
    }
}
