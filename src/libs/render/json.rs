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

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use insta::assert_snapshot;

    use super::*;

    macro_rules! row {
        ($($e:expr),* $(,)?) => { [$(&$e as &dyn std::fmt::Display),*] };
    }

    fn make(headers: &'static [&'static str; 2]) -> impl RendererTrait<2> {
        Renderer::new("T", "", headers).expect("new renderer")
    }

    #[test]
    fn json_empty() {
        let mut r = make(&["Name", "Value"]);
        assert_snapshot!(r.output(), @"[]");
    }

    #[test]
    fn json_single_row() {
        let mut r = make(&["Name", "Value"]);
        r.add_row(&row!["foo", "bar"]).expect("add_row");
        assert_snapshot!(r.output(), @r#"[{"Name":"foo","Value":"bar"}]"#);
    }

    #[test]
    fn json_multiple_rows() {
        let mut r = make(&["Name", "Value"]);
        r.add_row(&row!["foo", "bar"]).expect("add_row");
        r.add_row(&row!["baz", "qux"]).expect("add_row");
        assert_snapshot!(r.output(), @r#"[{"Name":"foo","Value":"bar"},{"Name":"baz","Value":"qux"}]"#);
    }

    #[test]
    fn json_special_chars_are_escaped() {
        let mut r = make(&["Name", "Value"]);
        r.add_row(&row!["line1\nline2", r#"has "quotes""#])
            .expect("add_row");
        assert_snapshot!(r.output(), @r#"[{"Name":"line1\nline2","Value":"has \"quotes\""}]"#);
    }

    #[test]
    fn json_mixed_display_types() {
        let mut r = make(&["Count", "Score"]);
        r.add_row(&row![42i32, 100u64]).expect("add_row");
        // All values are serialized as JSON strings, not numbers.
        assert_snapshot!(r.output(), @r#"[{"Count":"42","Score":"100"}]"#);
    }
}
