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
    use regex::{escape, Error as RegexError, Regex};
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
        pub fn make_include_filter_re(&self) -> Result<Option<Regex>, RegexError> {
            Self::make_filter_re(&self.include, &self.include_re)
        }

        pub fn make_exclude_filter_re(&self) -> Result<Option<Regex>, RegexError> {
            Self::make_filter_re(&self.exclude, &self.exclude_re)
        }

        fn make_filter_re(
            filter: &Option<SingleOrArray<String>>,
            re_filter: &Option<SingleOrArray<String>>,
        ) -> Result<Option<Regex>, RegexError> {
            let mut internal: Vec<String> = vec![];

            if let Some(ref exclude) = filter {
                let mut exclude_escaped: Vec<String> = Into::<Vec<&String>>::into(exclude)
                    .iter()
                    .map(|s| escape(s))
                    .collect();
                internal.append(&mut exclude_escaped);
            }

            if let Some(ref exclude_re) = re_filter.clone() {
                let mut exclude_re_escaped: Vec<String> = Into::<Vec<&String>>::into(exclude_re)
                    .iter()
                    .map(|s| format!("(?:{s})"))
                    .collect();
                internal.append(&mut exclude_re_escaped);
            }

            if internal.is_empty() {
                Ok(None)
            } else {
                Regex::new(&internal.join("|")).map(Some)
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
