use std::fmt::Debug;

use regex::Regex;
use serde::{Deserialize, Deserializer, de};

#[derive(Clone, Debug)]
pub struct Filter<T>
where
    T: Clone + Debug,
{
    pub reference: Option<String>,

    pub pattern: Option<String>,

    pub include_re: Option<Regex>,

    pub exclude_re: Option<Regex>,

    pub extra: Option<T>,
}

impl<T> Default for Filter<T>
where
    T: Clone + Debug,
{
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip()))]
    fn default() -> Self {
        Self {
            reference: None,
            pattern: Some("*".to_owned()),
            include_re: None,
            exclude_re: None,
            extra: None,
        }
    }
}

impl<'de, T> Deserialize<'de> for Filter<T>
where
    T: Deserialize<'de> + Clone + Debug,
{
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(deserializer), ret, err(level = "info"))
    )]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let intermediate = internal::Filter::<T>::deserialize(deserializer)?;

        let include_re = intermediate
            .make_include_filter_re()
            .map_err(de::Error::custom)?;

        let exclude_re = intermediate
            .make_exclude_filter_re()
            .map_err(de::Error::custom)?;

        Ok(Self {
            reference: intermediate.reference,
            pattern: intermediate.pattern,
            include_re,
            exclude_re,
            extra: intermediate.extra,
        })
    }
}

mod internal {
    use std::{fmt, fmt::Debug};

    use exn::{Result, ResultExt as _};
    use regex::{Regex, escape};
    use rust_i18n::t;
    use serde::{Deserialize, Deserializer, de};

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(deserializer), ret, err(level = "debug"))
    )]
    fn deserialize_string_or_vec<'de, D>(
        deserializer: D,
    ) -> core::result::Result<Option<Vec<String>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;

        impl<'de> de::Visitor<'de> for V {
            type Value = Vec<String>;

            #[cfg_attr(
                feature = "tracing",
                tracing::instrument(level = "trace", skip(self, formatter), err(level = "debug"))
            )]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or a sequence of strings")
            }

            #[cfg_attr(
                feature = "tracing",
                tracing::instrument(level = "trace", skip(self, v), ret, err(level = "debug"))
            )]
            fn visit_str<E: de::Error>(self, v: &str) -> core::result::Result<Self::Value, E> {
                Ok(vec![v.to_owned()])
            }

            #[cfg_attr(
                feature = "tracing",
                tracing::instrument(level = "trace", skip(self, seq), ret, err(level = "debug"))
            )]
            fn visit_seq<S: de::SeqAccess<'de>>(
                self,
                seq: S,
            ) -> core::result::Result<Self::Value, S::Error> {
                Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
            }
        }

        deserializer.deserialize_any(V).map(Some)
    }

    #[derive(Debug, derive_more::Display)]
    pub enum FilterError {
        #[display("{}", t!("error.filter.bad_regex", re = re : {:?}))]
        BadRegex { re: String },
    }
    impl std::error::Error for FilterError {}

    /// Private structure without the regex
    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
    pub struct Filter<T>
    where
        T: Clone + Debug,
    {
        pub reference: Option<String>,
        pub pattern: Option<String>,
        pub extra: Option<T>,
        #[serde(default, deserialize_with = "deserialize_string_or_vec")]
        pub include: Option<Vec<String>>,
        #[serde(default, deserialize_with = "deserialize_string_or_vec")]
        pub include_re: Option<Vec<String>>,
        #[serde(default, deserialize_with = "deserialize_string_or_vec")]
        pub exclude: Option<Vec<String>>,
        #[serde(default, deserialize_with = "deserialize_string_or_vec")]
        pub exclude_re: Option<Vec<String>>,
    }

    impl<T> Filter<T>
    where
        T: Clone + Debug,
    {
        #[cfg_attr(
            feature = "tracing",
            tracing::instrument(level = "trace", skip(self), ret, err(level = "info"))
        )]
        pub fn make_include_filter_re(&self) -> Result<Option<Regex>, FilterError> {
            Self::make_filter_re(self.include.as_deref(), self.include_re.as_deref())
        }

        #[cfg_attr(
            feature = "tracing",
            tracing::instrument(level = "trace", skip(self), ret, err(level = "info"))
        )]
        pub fn make_exclude_filter_re(&self) -> Result<Option<Regex>, FilterError> {
            Self::make_filter_re(self.exclude.as_deref(), self.exclude_re.as_deref())
        }

        #[cfg_attr(
            feature = "tracing",
            tracing::instrument(
                level = "trace",
                skip(filter, re_filter),
                ret,
                err(level = "info")
            )
        )]
        fn make_filter_re(
            filter: Option<&[String]>,
            re_filter: Option<&[String]>,
        ) -> Result<Option<Regex>, FilterError> {
            #[cfg(feature = "tracing")]
            tracing::trace!(?filter, ?re_filter);

            let mut internal: Vec<String> = vec![];

            if let Some(exclude) = filter {
                let mut exclude_escaped: Vec<_> = exclude.iter().map(|s| escape(s)).collect();
                internal.append(&mut exclude_escaped);
            }

            if let Some(exclude_re) = re_filter {
                let mut exclude_re_escaped: Vec<_> =
                    exclude_re.iter().map(|s| format!("(?:{s})")).collect();
                internal.append(&mut exclude_re_escaped);
            }

            if internal.is_empty() {
                Ok(None)
            } else {
                let full_re = internal.join("|");
                Regex::new(&full_re)
                    .or_raise(|| FilterError::BadRegex { re: full_re })
                    .map(Some)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "test")]

    use serde::Deserialize;
    use serde_any::{Format, from_str};

    use crate::libs::filter::Filter;

    #[derive(Clone, Debug, Deserialize, PartialEq)]
    struct ExtraConfig {
        additional_info: String,
    }

    /// Test deserialization and internal regex creation
    #[test]
    fn filter_deserialization_and_regex_creation() {
        let json_data = r#"
    {
        "reference": "test_ref",
        "pattern": "*",
        "include": ["include_this", "also_this"],
        "include-re": ["^include_pattern.*"],
        "exclude": ["exclude_this"],
        "exclude-re": ["^exclude_pattern.*"],
        "extra": {"additional_info": "test info"}
    }"#;

        let filter: Filter<ExtraConfig> = from_str(json_data, Format::Json).expect("should parse");
        assert_eq!(filter.reference, Some("test_ref".to_owned()));
        assert_eq!(filter.pattern, Some("*".to_owned()));

        // Test regex patterns created in include_re and exclude_re
        assert!(filter.include_re.is_some());
        let include_re = filter.include_re.as_ref().expect("should parse");
        assert!(include_re.is_match("include_this"));
        assert!(include_re.is_match("include_pattern123"));
        assert!(!include_re.is_match("exclude_this"));

        assert!(filter.exclude_re.is_some());
        let exclude_re = filter.exclude_re.as_ref().expect("should parse");
        assert!(exclude_re.is_match("exclude_this"));
        assert!(exclude_re.is_match("exclude_pattern123"));
        assert!(!exclude_re.is_match("include_this"));

        // Check additional information in extra field
        assert_eq!(
            filter.extra,
            Some(ExtraConfig {
                additional_info: "test info".to_owned()
            })
        );
    }

    /// Test default values
    #[test]
    fn filter_default() {
        let filter: Filter<()> = Filter::default();
        assert!(filter.reference.is_none());
        assert_eq!(filter.pattern, Some("*".to_owned()));
        assert!(filter.include_re.is_none());
        assert!(filter.exclude_re.is_none());
        assert!(filter.extra.is_none());
    }

    /// Test regex creation with empty include/exclude to confirm None is returned
    #[test]
    fn filter_no_regex_created() {
        let json_data = r#"
    {
        "pattern": "*"
    }"#;

        let filter: Filter<()> =
            from_str(json_data, Format::Json).expect("valid JSON should parse into Filter");
        assert!(filter.include_re.is_none());
        assert!(filter.exclude_re.is_none());
    }

    /// Test `make_filter_re` with only literal includes (no regex part)
    #[test]
    fn filter_only_literal_includes() {
        let json_data = r#"{"pattern": "*", "include": ["foo", "b.r"]}"#;
        let filter: Filter<()> =
            from_str(json_data, Format::Json).expect("valid JSON should parse into Filter");
        let re = filter
            .include_re
            .as_ref()
            .expect("include_re should be set after parsing literal includes");
        // Literals are regex-escaped: "foo" matches exactly, "b.r" does NOT match "bar"
        assert!(re.is_match("foo"), "literal 'foo' should match");
        assert!(re.is_match("b.r"), "literal 'b.r' should match");
        assert!(!re.is_match("bar"), "'bar' should not match escaped 'b.r'");
    }

    /// Test `make_filter_re` with only regex includes (no literal part)
    #[test]
    fn filter_only_regex_includes() {
        let json_data = r#"{"pattern": "*", "include-re": ["^prefix.*"]}"#;
        let filter: Filter<()> =
            from_str(json_data, Format::Json).expect("valid JSON should parse into Filter");
        let re = filter
            .include_re
            .as_ref()
            .expect("include_re should be set after parsing regex includes");
        assert!(
            re.is_match("prefix_anything"),
            "prefix_anything should match"
        );
        assert!(!re.is_match("no_prefix"), "no_prefix should not match");
    }

    /// Test error handling for invalid regex patterns
    #[test]
    fn filter_invalid_regex() {
        let json_data = r#"
    {
        "pattern": "*",
        "include-re": ["["]
    }"#;

        let result: Result<Filter<()>, _> = from_str(json_data, Format::Json);
        assert!(result.is_err());
    }
}
