use regex::Regex;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt::Debug;

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
            pattern: Some("*".to_string()),
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
            priv_include: intermediate.include.map(std::convert::Into::into),
            priv_include_re: intermediate.include_re.map(std::convert::Into::into),
            priv_exclude: intermediate.exclude.map(std::convert::Into::into),
            priv_exclude_re: intermediate.exclude_re.map(std::convert::Into::into),
        })
    }
}

mod internal {
    use super::Filter as RealFilter;
    use crate::libs::single_or_array::SingleOrArray;
    use anyhow::{Context, Result};
    use regex::{escape, Regex};
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    /// Private structure without the regex
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
    pub struct Filter<T>
    where
        T: Clone + Debug,
    {
        pub reference: Option<String>,
        pub pattern: Option<String>,
        pub extra: Option<T>,
        pub include: Option<SingleOrArray<String>>,
        pub include_re: Option<SingleOrArray<String>>,
        pub exclude: Option<SingleOrArray<String>>,
        pub exclude_re: Option<SingleOrArray<String>>,
    }

    impl<T> Filter<T>
    where
        T: Clone + Debug + Serialize,
    {
        pub fn make_include_filter_re(&self) -> Result<Option<Regex>> {
            Self::make_filter_re(self.include.as_ref(), self.include_re.as_ref())
        }

        pub fn make_exclude_filter_re(&self) -> Result<Option<Regex>> {
            Self::make_filter_re(self.exclude.as_ref(), self.exclude_re.as_ref())
        }

        fn make_filter_re(
            filter: Option<&SingleOrArray<String>>,
            re_filter: Option<&SingleOrArray<String>>,
        ) -> Result<Option<Regex>> {
            let mut internal: Vec<String> = vec![];

            if let Some(exclude) = filter {
                let mut exclude_escaped: Vec<String> = Into::<Vec<&String>>::into(exclude)
                    .iter()
                    .map(|s| escape(s))
                    .collect();
                internal.append(&mut exclude_escaped);
            }

            if let Some(exclude_re) = re_filter {
                let mut exclude_re_escaped: Vec<String> = Into::<Vec<&String>>::into(exclude_re)
                    .iter()
                    .map(|s| format!("(?:{s})"))
                    .collect();
                internal.append(&mut exclude_re_escaped);
            }

            if internal.is_empty() {
                Ok(None)
            } else {
                let full_re = internal.join("|");
                Regex::new(&full_re)
                    .with_context(|| format!("regexp creation failed for {full_re:?}"))
                    .map(Some)
            }
        }
    }

    impl<T> From<&RealFilter<T>> for Filter<T>
    where
        T: Clone + Debug + Serialize,
    {
        fn from(filter: &RealFilter<T>) -> Self {
            Self {
                reference: filter.reference.clone(),
                pattern: filter.pattern.clone(),
                extra: filter.extra.clone(),
                include: filter.priv_include.clone().map(std::convert::Into::into),
                include_re: filter.priv_include_re.clone().map(std::convert::Into::into),
                exclude: filter.priv_exclude.clone().map(std::convert::Into::into),
                exclude_re: filter.priv_exclude_re.clone().map(std::convert::Into::into),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::libs::filter::Filter;
    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use serde_any::{from_str, to_string, Format};

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    struct ExtraConfig {
        additional_info: String,
    }

    /// Test deserialization and internal regex creation
    #[test]
    fn test_filter_deserialization_and_regex_creation() {
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
        assert_eq!(filter.reference, Some("test_ref".to_string()));
        assert_eq!(filter.pattern, Some("*".to_string()));

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
                additional_info: "test info".to_string()
            })
        );
    }

    /// Test serialization, ensuring expected fields are present
    #[test]
    fn test_filter_serialization() {
        let filter = Filter::<ExtraConfig> {
            reference: Some("test_ref".to_string()),
            pattern: Some("*".to_string()),
            include_re: Some(Regex::new("include_this|^include_pattern.*").unwrap()),
            exclude_re: Some(Regex::new("exclude_this|^exclude_pattern.*").unwrap()),
            extra: Some(ExtraConfig {
                additional_info: "test info".to_string(),
            }),
            priv_include: Some(vec!["include_this".to_string(), "also_this".to_string()]),
            priv_include_re: Some(vec!["^include_pattern.*".to_string()]),
            priv_exclude: Some(vec!["exclude_this".to_string()]),
            priv_exclude_re: Some(vec!["^exclude_pattern.*".to_string()]),
        };

        let json = to_string(&filter, Format::Json).unwrap();
        assert!(json.contains(r#""reference":"test_ref""#));
        assert!(json.contains(r#""pattern":"*""#));
        assert!(json.contains(r#""include":["include_this","also_this"]"#));
        assert!(json.contains(r#""include-re":"^include_pattern.*""#));
        assert!(json.contains(r#""exclude":"exclude_this""#));
        assert!(json.contains(r#""exclude-re":"^exclude_pattern.*""#));
        assert!(json.contains(r#""additional_info":"test info""#));
    }

    /// Test default values
    #[test]
    fn test_filter_default() {
        let filter: Filter<()> = Filter::default();
        assert!(filter.reference.is_none());
        assert_eq!(filter.pattern, Some("*".to_string()));
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
    fn test_filter_no_regex_created() {
        let json_data = r#"
    {
        "pattern": "*"
    }"#;

        let filter: Filter<()> = from_str(json_data, Format::Json).unwrap();
        assert!(filter.include_re.is_none());
        assert!(filter.exclude_re.is_none());
    }

    /// Test error handling for invalid regex patterns
    #[test]
    fn test_filter_invalid_regex() {
        let json_data = r#"
    {
        "pattern": "*",
        "include-re": ["["]
    }"#;

        let result: Result<Filter<()>, _> = from_str(json_data, Format::Json);
        assert!(result.is_err());
    }
}
