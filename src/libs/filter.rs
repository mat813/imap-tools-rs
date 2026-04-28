use std::fmt::Debug;

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, de};

#[derive(Debug, Clone)]
pub struct Filter<T>
where
    T: Clone + Debug + Serialize,
{
    pub reference: Option<String>,

    pub pattern: Option<String>,

    pub include_re: Option<Regex>,

    pub exclude_re: Option<Regex>,

    pub extra: Option<T>,

    priv_include: Option<Vec<String>>,
    priv_include_re: Option<Vec<String>>,
    priv_exclude: Option<Vec<String>>,
    priv_exclude_re: Option<Vec<String>>,
}

impl<T> Default for Filter<T>
where
    T: Clone + Debug + Serialize,
{
    fn default() -> Self {
        Self {
            reference: None,
            pattern: Some("*".to_owned()),
            include_re: None,
            exclude_re: None,
            extra: None,
            priv_include: None,
            priv_include_re: None,
            priv_exclude: None,
            priv_exclude_re: None,
        }
    }
}

impl<T> Serialize for Filter<T>
where
    T: Clone + Debug + Serialize,
{
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, serializer), err(level = "info"))
    )]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let intermediate: internal::Filter<T> = self.into();
        intermediate.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Filter<T>
where
    T: Deserialize<'de> + Clone + Debug + Serialize,
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

        // Return Filter with internal_re populated
        Ok(Self {
            reference: intermediate.reference,
            pattern: intermediate.pattern,
            include_re,
            exclude_re,
            extra: intermediate.extra,
            priv_include: intermediate.include,
            priv_include_re: intermediate.include_re,
            priv_exclude: intermediate.exclude,
            priv_exclude_re: intermediate.exclude_re,
        })
    }
}

mod internal {
    use std::{fmt, fmt::Debug};

    use derive_more::Display;
    use exn::{Result, ResultExt as _};
    use regex::{Regex, escape};
    use serde::{Deserialize, Deserializer, Serialize, de};

    use super::Filter as RealFilter;

    fn deserialize_string_or_vec<'de, D>(
        deserializer: D,
    ) -> core::result::Result<Option<Vec<String>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;

        impl<'de> de::Visitor<'de> for V {
            type Value = Vec<String>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or a sequence of strings")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> core::result::Result<Self::Value, E> {
                Ok(vec![v.to_owned()])
            }

            fn visit_seq<S: de::SeqAccess<'de>>(
                self,
                seq: S,
            ) -> core::result::Result<Self::Value, S::Error> {
                Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
            }
        }

        deserializer.deserialize_any(V).map(Some)
    }

    #[derive(Debug, Display)]
    pub struct FilterError(String);
    impl std::error::Error for FilterError {}

    /// Private structure without the regex
    #[derive(Deserialize, Serialize, Debug)]
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
        T: Clone + Debug + Serialize,
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
                    .or_raise(|| FilterError(format!("regexp creation failed for {full_re:?}")))
                    .map(Some)
            }
        }
    }

    impl<T> From<&RealFilter<T>> for Filter<T>
    where
        T: Clone + Debug + Serialize,
    {
        #[cfg_attr(
            feature = "tracing",
            tracing::instrument(level = "trace", skip(filter), ret)
        )]
        fn from(filter: &RealFilter<T>) -> Self {
            #[cfg(feature = "tracing")]
            tracing::trace!(?filter);

            Self {
                reference: filter.reference.clone(),
                pattern: filter.pattern.clone(),
                extra: filter.extra.clone(),
                include: filter.priv_include.clone(),
                include_re: filter.priv_include_re.clone(),
                exclude: filter.priv_exclude.clone(),
                exclude_re: filter.priv_exclude_re.clone(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test")]

    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use serde_any::{Format, from_str, to_string};

    use crate::libs::filter::Filter;

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

        let filter: Filter<ExtraConfig> = from_str(json_data, Format::Json).unwrap();
        assert_eq!(filter.reference, Some("test_ref".to_owned()));
        assert_eq!(filter.pattern, Some("*".to_owned()));

        // Test regex patterns created in include_re and exclude_re
        assert!(filter.include_re.is_some());
        let include_re = filter.include_re.as_ref().unwrap();
        assert!(include_re.is_match("include_this"));
        assert!(include_re.is_match("include_pattern123"));
        assert!(!include_re.is_match("exclude_this"));

        assert!(filter.exclude_re.is_some());
        let exclude_re = filter.exclude_re.as_ref().unwrap();
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

    /// Test serialization, ensuring expected fields are present
    #[test]
    fn filter_serialization() {
        let filter = Filter::<ExtraConfig> {
            reference: Some("test_ref".to_owned()),
            pattern: Some("*".to_owned()),
            include_re: Some(Regex::new("include_this|^include_pattern.*").unwrap()),
            exclude_re: Some(Regex::new("exclude_this|^exclude_pattern.*").unwrap()),
            extra: Some(ExtraConfig {
                additional_info: "test info".to_owned(),
            }),
            priv_include: Some(vec!["include_this".to_owned(), "also_this".to_owned()]),
            priv_include_re: Some(vec!["^include_pattern.*".to_owned()]),
            priv_exclude: Some(vec!["exclude_this".to_owned()]),
            priv_exclude_re: Some(vec!["^exclude_pattern.*".to_owned()]),
        };

        let json = to_string(&filter, Format::Json).unwrap();
        assert!(json.contains(r#""reference":"test_ref""#));
        assert!(json.contains(r#""pattern":"*""#));
        assert!(json.contains(r#""include":["include_this","also_this"]"#));
        assert!(json.contains(r#""include-re":["^include_pattern.*"]"#));
        assert!(json.contains(r#""exclude":["exclude_this"]"#));
        assert!(json.contains(r#""exclude-re":["^exclude_pattern.*"]"#));
        assert!(json.contains(r#""additional_info":"test info""#));
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
        assert!(filter.priv_include.is_none());
        assert!(filter.priv_include_re.is_none());
        assert!(filter.priv_exclude.is_none());
        assert!(filter.priv_exclude_re.is_none());
    }

    /// Test regex creation with empty include/exclude to confirm None is returned
    #[test]
    fn filter_no_regex_created() {
        let json_data = r#"
    {
        "pattern": "*"
    }"#;

        let filter: Filter<()> = from_str(json_data, Format::Json).unwrap();
        assert!(filter.include_re.is_none());
        assert!(filter.exclude_re.is_none());
    }

    /// Test `make_filter_re` with only literal includes (no regex part)
    #[test]
    fn filter_only_literal_includes() {
        let json_data = r#"{"pattern": "*", "include": ["foo", "b.r"]}"#;
        let filter: Filter<()> = from_str(json_data, Format::Json).unwrap();
        let re = filter.include_re.as_ref().unwrap();
        // Literals are regex-escaped: "foo" matches exactly, "b.r" does NOT match "bar"
        assert!(re.is_match("foo"), "literal 'foo' should match");
        assert!(re.is_match("b.r"), "literal 'b.r' should match");
        assert!(!re.is_match("bar"), "'bar' should not match escaped 'b.r'");
    }

    /// Test `make_filter_re` with only regex includes (no literal part)
    #[test]
    fn filter_only_regex_includes() {
        let json_data = r#"{"pattern": "*", "include-re": ["^prefix.*"]}"#;
        let filter: Filter<()> = from_str(json_data, Format::Json).unwrap();
        let re = filter.include_re.as_ref().unwrap();
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
