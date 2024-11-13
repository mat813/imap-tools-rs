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
            Self::Single(
                v.into_iter()
                    .next()
                    // We can unwrap() here because we know there is one entry, so next() is not None.
                    .unwrap(),
            )
        } else {
            Self::Array(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_any::{from_str, to_string, Format};

    #[test]
    fn test_serialization_single() {
        let single = SingleOrArray::Single(42);
        let serialized = to_string(&single, Format::Json).expect("Serialization failed");
        assert_eq!(serialized, "42");
    }

    #[test]
    fn test_serialization_array() {
        let array = SingleOrArray::Array(vec![1, 2, 3]);
        let serialized = to_string(&array, Format::Json).expect("Serialization failed");
        assert_eq!(serialized, "[1,2,3]");
    }

    #[test]
    fn test_deserialization_single() {
        let json = "42";
        let deserialized: SingleOrArray<i32> =
            from_str(json, Format::Json).expect("Deserialization failed");
        match deserialized {
            SingleOrArray::Single(value) => assert_eq!(value, 42),
            SingleOrArray::Array(_) => panic!("Expected Single variant"),
        }
    }

    #[test]
    fn test_deserialization_array() {
        let json = "[1,2,3]";
        let deserialized: SingleOrArray<i32> =
            from_str(json, Format::Json).expect("Deserialization failed");
        match deserialized {
            SingleOrArray::Array(values) => assert_eq!(values, vec![1, 2, 3]),
            SingleOrArray::Single(_) => panic!("Expected Array variant"),
        }
    }

    #[test]
    fn test_conversion_single_to_vec() {
        let single = SingleOrArray::Single(42);
        let vec: Vec<i32> = Vec::from(single);
        assert_eq!(vec, vec![42]);
    }

    #[test]
    fn test_conversion_array_to_vec() {
        let array = SingleOrArray::Array(vec![1, 2, 3]);
        let vec: Vec<i32> = Vec::from(array);
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_conversion_single_to_vec_ref() {
        let single = SingleOrArray::Single(42);
        let vec_ref: Vec<&i32> = Vec::from(&single);
        assert_eq!(vec_ref, vec![&42]);
    }

    #[test]
    fn test_conversion_array_to_vec_ref() {
        let array = SingleOrArray::Array(vec![1, 2, 3]);
        let vec_ref: Vec<&i32> = Vec::from(&array);
        assert_eq!(vec_ref, vec![&1, &2, &3]);
    }

    #[test]
    fn test_conversion_vec_single_to_singleorarray() {
        let vec = vec![42];
        let single_or_array: SingleOrArray<i32> = SingleOrArray::from(vec);
        match single_or_array {
            SingleOrArray::Single(value) => assert_eq!(value, 42),
            SingleOrArray::Array(_) => panic!("Expected Single variant"),
        }
    }

    #[test]
    fn test_conversion_vec_array_to_singleorarray() {
        let vec = vec![1, 2, 3];
        let single_or_array: SingleOrArray<i32> = SingleOrArray::from(vec);
        match single_or_array {
            SingleOrArray::Array(values) => assert_eq!(values, vec![1, 2, 3]),
            SingleOrArray::Single(_) => panic!("Expected Array variant"),
        }
    }

    #[test]
    fn test_cloning_single_variant() {
        let single = SingleOrArray::Single(42);
        let cloned = single;
        match cloned {
            SingleOrArray::Single(value) => assert_eq!(value, 42),
            SingleOrArray::Array(_) => panic!("Expected Single variant"),
        }
    }

    #[test]
    fn test_cloning_array_variant() {
        let array = SingleOrArray::Array(vec![1, 2, 3]);
        let cloned = array;
        match cloned {
            SingleOrArray::Array(values) => assert_eq!(values, vec![1, 2, 3]),
            SingleOrArray::Single(_) => panic!("Expected Array variant"),
        }
    }

    #[test]
    fn test_debug_output_single() {
        let single = SingleOrArray::Single(42);
        let debug_output = format!("{single:?}");
        assert_eq!(debug_output, "Single(42)");
    }

    #[test]
    fn test_debug_output_array() {
        let array = SingleOrArray::Array(vec![1, 2, 3]);
        let debug_output = format!("{array:?}");
        assert_eq!(debug_output, "Array([1, 2, 3])");
    }
}
