use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SingleOrArray<T>
where
    T: Clone + Debug,
{
    Single(T),
    Array(Vec<T>),
}

impl<T> From<SingleOrArray<T>> for Vec<T>
where
    T: Clone + Debug,
{
    fn from(value: SingleOrArray<T>) -> Self {
        match value {
            SingleOrArray::Single(s) => vec![s],
            SingleOrArray::Array(vec) => vec,
        }
    }
}

impl<'a, T> From<&'a SingleOrArray<T>> for Vec<&'a T>
where
    T: Clone + Debug,
{
    fn from(value: &'a SingleOrArray<T>) -> Self {
        match value {
            SingleOrArray::Single(s) => vec![s],
            SingleOrArray::Array(vec) => vec.iter().collect(),
        }
    }
}

// This is a fallible impl, but we know that the input vector will always be non-empty
#[expect(clippy::fallible_impl_from)]
impl<T> From<Vec<T>> for SingleOrArray<T>
where
    T: Clone + Debug,
{
    fn from(v: Vec<T>) -> Self {
        if v.len() == 1 {
            Self::Single(v.into_iter().next().unwrap())
        } else {
            Self::Array(v)
        }
    }
}
