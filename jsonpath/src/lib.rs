#[macro_use]
extern crate pest_derive;

use std::iter::FromIterator;
use std::mem;

use serde_json::{Value, Map, map::Entry};

mod parser;
pub mod error;

use crate::parser::parse;
use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum PathItem {
    Child(String),
    Index(isize),
}

pub struct Selector {
    json_path: Vec<PathItem>,
}

impl Selector {
    pub fn new(expression: &str) -> Result<Self> {
        Ok(Self {
            json_path: parse(expression)?,
        })
    }

    pub fn get<'a>(&self, value: &'a Value) -> Option<&'a Value> {
        let mut curr = Some(value);
        for item in &self.json_path {
            curr = match curr {
                Some(curr) => {
                    match item {
                        PathItem::Child(ident) => {
                            curr
                                .as_object()
                                .and_then(|m| m.get(ident))
                        }
                        PathItem::Index(index) => {
                            curr
                                .as_array()
                                .and_then(|a| signed_get(a, *index))
                        }
                    }
                }
                None => return None,
            }
        }

        curr
    }

    fn json_path_get_mut<'a>(json_path: &[PathItem], value: &'a mut Value)
        -> Option<&'a mut Value>
    {
        let mut curr = Some(value);
        for item in json_path {
            curr = match curr {
                Some(curr) => {
                    match item {
                        PathItem::Child(ident) => {
                            curr
                                .as_object_mut()
                                .and_then(|m| m.get_mut(ident))
                        }
                        PathItem::Index(index) => {
                            curr
                                .as_array_mut()
                                .and_then(|a| signed_get_mut(a, *index))
                        }
                    }
                }
                None => return None,
            }
        }

        curr
    }

    pub fn get_mut<'a>(&self, value: &'a mut Value) -> Option<&'a mut Value> {
        Self::json_path_get_mut(&self.json_path, value)
    }

    pub fn set(&self, value: &mut Value, new_value: Value) -> Result<Option<Value>> {
        let mut curr = value;
        let mut is_missing = false;
        for item in &self.json_path {
            // JSON type mismatch is treated as error in `set` method.
            curr = match *item {
                PathItem::Child(ref ident) => {
                    if curr.is_null() {
                        is_missing = true;
                        *curr = Value::Object(Map::new());
                    }

                    let m = curr
                        .as_object_mut()
                        .ok_or_else(|| Error::NotJsonMapError)?;
                    match m.entry(ident.to_owned()) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(v) => {
                            v.insert(Value::Null)
                        }
                    }
                }
                PathItem::Index(index) => {
                    if curr.is_null() {
                        is_missing = true;
                        *curr = Value::Array(vec![]);
                    }

                    let a = curr
                        .as_array_mut()
                        .ok_or_else(|| Error::NotJsonArrayError)?;

                    (0 ..= (index - a.len() as isize)).for_each(|_| a.push(Value::Null));

                    match signed_get_mut(a, index) {
                        Some(v) => v,
                        None => return Err(Error::JsonInvalidArrayIndexError(index)),
                    }
                }
            };
        }

        let ret = mem::replace(curr, new_value);
        Ok(if is_missing {
            None
        } else {
            Some(ret)
        })
    }

    pub fn remove(&self, value: &mut Value) -> Option<Value> {
        let last_index = self.json_path.len() - 1;
        let json_path_but_last = &self.json_path[..last_index];
        if let Some(last_level) = Self::json_path_get_mut(json_path_but_last, value) {
            let last_path_item = &self.json_path[last_index];
            match last_path_item {
                PathItem::Child(ident) => {
                    last_level
                        .as_object_mut()
                        .and_then(|m| m.remove(ident))
                }
                PathItem::Index(index) => {
                    last_level
                        .as_array_mut()
                        .and_then(|a| signed_remove(a, *index))
                }
            }
        } else {
            None
        }
    }
}

// We can't write `impl<T> SliceIndex<[T]> for isize`.
fn signed_get<T>(arr: &[T], index: isize) -> Option<&T> {
    match signed_index(index, arr.len()) {
        Some(i) => arr.get(i),
        None => None,
    }
}

fn signed_get_mut<T>(arr: &mut [T], index: isize) -> Option<&mut T> {
    match signed_index(index, arr.len()) {
        Some(i) => arr.get_mut(i),
        None => None,
    }
}

fn signed_remove<T>(arr: &mut Vec<T>, index: isize) -> Option<T> {
    match signed_index(index, arr.len()) {
        Some(i) => Some(arr.remove(i)),
        None => None,
    }
}

fn signed_index(index: isize, len: usize) -> Option<usize> {
    if index >= 0 {
        let u_index = index as usize;
        if u_index >= len {
            None
        } else {
            Some(u_index)
        }
    } else {
        len.checked_sub(index.abs() as usize)
    }
}

/// Creates nested JSON value from given JSONPath and value on leaf node. For example, `{ json_path:
/// "foo.bar[2].baz", leaf_value: 42 }` will generates
///
/// ```js
/// {
///     "foo" : {
///         "bar": [
///             null,
///             null,
///             "baz": 42
///         ]
///     }
/// }
/// ```
#[deprecated]
#[allow(dead_code)]
fn create_from_path(json_path: &[PathItem], leaf_value: Value) -> Result<Value> {
    let mut curr = leaf_value;
    for item in json_path.iter().rev() {
        match *item {
            PathItem::Child(ref ident) => {
                curr = Value::Object(Map::from_iter(vec![(ident.to_owned(), curr)]));
            }
            PathItem::Index(index) => {
                if index < 0 {
                    return Err(Error::JsonInvalidArrayIndexError(index));
                }
                let index = index as usize;
                let mut a: Vec<Value> = (0..=index).map(|_| Value::Null).collect();
                a[index] = curr;
                curr = Value::Array(a);
            }
        }
    }
    Ok(curr)
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn selector_smoke() -> Result<()> {
        let mut data = json!({
            "foo": {
                "bar": 42,
                "baz": "Hello"
            },
            "qux": null
        });

        let selector = Selector::new("foo.bar")?;
        assert_eq!(selector.get(&data), Some(&Value::Number(42.into())));

        let selector = Selector::new("foo.baz")?;
        let value = Value::String("World".into());
        assert_eq!(selector.set(&mut data, value.clone())?, Some(Value::String("Hello".into())));
        assert_eq!(selector.get(&data), Some(&value));

        assert_eq!(selector.remove(&mut data), Some(value.clone()));
        assert_eq!(selector.get(&data), None);

        Ok(())
    }

    #[test]
    fn selector_path_with_dots() -> Result<()> {
        let data = json!({
            "foo": {
                "bar.baz": true,
                "bar": {
                    "baz": 1
                },
                "0": 0,
            }
        });

        let selector = Selector::new("foo.bar.baz")?;
        assert_eq!(selector.get(&data), Some(&Value::Number(1.into())));

        let selector_dots = Selector::new("foo['bar.baz']")?;
        assert_eq!(selector_dots.get(&data), Some(&Value::Bool(true)));

        let selector_number_string = Selector::new("foo['0']")?;
        assert_eq!(selector_number_string.get(&data),Some(&Value::Number(0.into())));

        Ok(())
    }

    #[test]
    fn selector_map() -> Result<()> {
        let selector = Selector::new("foo.bar.baz")?;
        let new_value = Value::Number(42.into());
        let expected = json!({
            "foo": {
                "bar": {
                    "baz": 42
                }
            }
        });

        // The corresponding value of partial key is an empty map.
        let mut data = json!({
            "foo": {}
        });
        assert_eq!(selector.set(&mut data, new_value.clone())?, None);
        assert_eq!(data, expected);

        // The corresponding value of partial key is null.
        let mut data = json!({
            "foo": null
        });
        assert_eq!(selector.set(&mut data, new_value.clone())?, None);
        assert_eq!(data, expected);

        // The old value is null.
        let mut data = json!({
            "foo": {
                "bar": {
                    "baz": null
                }
            }
        });
        assert_eq!(selector.set(&mut data, new_value.clone())?, Some(Value::Null));
        assert_eq!(data, expected);

        // The corresponding value of partial key is an array, but trying to assign a map.
        let mut data = json!({
            "foo": []
        });
        assert!(selector.set(&mut data, new_value.clone()).is_err());

        // Remove keys from map
        let mut data = expected.clone();
        assert_eq!(selector.remove(&mut data), Some(new_value.clone()));
        assert_eq!(data, json!({
            "foo": {
                "bar": {
                }
            }
        }));

        Ok(())
    }

    #[test]
    fn selector_array() -> Result<()> {
        // Normal case
        let mut data = json!({
            "foo": [
                0,
                1,
                2
            ]
        });
        let selector = Selector::new("foo[1]")?;
        assert_eq!(selector.get(&data), Some(&Value::Number(1.into())));
        selector.set(&mut data, Value::Number(100.into()))?;
        assert_eq!(selector.get(&data), Some(&Value::Number(100.into())));

        // Remove items from array
        let mut data = json!([
            0,
            1,
            2
        ]);
        let selector = Selector::new("[1]")?;
        assert_eq!(selector.remove(&mut data), Some(Value::Number(1.into())));
        assert_eq!(data, json!([0, 2]));
        assert_eq!(selector.remove(&mut data), Some(Value::Number(2.into())));
        assert_eq!(data, json!([0]));
        assert_eq!(selector.remove(&mut data), None);

        // Array index exceeds upper bound
        let mut data = json!({
            "foo": [
                0
            ]
        });
        let selector = Selector::new("foo[2]")?;
        assert_eq!(selector.get(&data), None);
        selector.set(&mut data, Value::Number(2.into()))?;
        assert_eq!(data, json!({
            "foo": [
                0,
                null,
                2
            ]
        }));

        // Interleaved arrays and maps
        let data = json!({
            "foo": [
                {
                    "bar": [
                        null,
                        "baz"
                    ]
                }
            ]
        });
        let selector = Selector::new("foo[0].bar[1]")?;
        assert_eq!(selector.get(&data), Some(&Value::String("baz".into())));

        Ok(())
    }
}
