#[macro_use]
extern crate pest_derive;

use std::mem;

use serde_json::Value;

mod parser;
pub mod error;

use crate::parser::parse;
use crate::error::Result;

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

    pub fn get_mut<'a>(&self, value: &'a mut Value) -> Option<&'a mut Value> {
        let mut curr = Some(value);
        for item in &self.json_path {
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

    pub fn set(&self, value: &mut Value, new_value: Value) -> Option<Value> {
        self.get_mut(value)
            .map(|p| mem::replace(p, new_value))
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

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn selector() -> Result<()> {
        let data = json!({
            "foo": {
                "bar": 42,
                "baz": "Hello"
            },
            "qux": null
        });

        let selector = Selector::new("foo.bar")?;
        assert_eq!(selector.get(&data), Some(&Value::Number(42.into())));

        let mut data = data;
        let selector = Selector::new("foo.baz")?;
        let value = Value::String("World".into());
        assert_eq!(selector.set(&mut data, value.clone()), Some(Value::String("Hello".into())));
        assert_eq!(selector.get(&data), Some(&value));

        Ok(())
    }
}
