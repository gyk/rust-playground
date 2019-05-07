//! Provides extension methods to serde-json's `Value` ('try_trait' is not stable yet).

use serde_json::{Value, Map, value::Index};

use crate::error::{Error, Result};

pub trait ValueExt {
    /// Similar to `as_object`, but returns `Result`.
    fn as_map(&self) -> Result<&Map<String, Value>>;
    /// The mutable version of `as_map`.
    fn as_map_mut(&mut self) -> Result<&mut Map<String, Value>>;
    /// `get` that returns `Result`.
    fn get_value<I: Index>(&self, index: I) -> Result<&Value>;
    /// `get_mut` that returns `Result`.
    fn get_value_mut<I: Index>(&mut self, index: I) -> Result<&mut Value>;
}

impl ValueExt for Value {
    fn as_map(&self) -> Result<&Map<String, Value>> {
        self.as_object()
            .ok_or_else(|| Error::NotJsonMapError)
    }

    fn as_map_mut(&mut self) -> Result<&mut Map<String, Value>> {
        self.as_object_mut()
            .ok_or_else(|| Error::NotJsonMapError)
    }

    fn get_value<I: Index>(&self, index: I) -> Result<&Value> {
        self.get(index)
            .ok_or_else(|| Error::JsonInvalidIndexError)
    }

    fn get_value_mut<I: Index>(&mut self, index: I) -> Result<&mut Value> {
        self.get_mut(index)
            .ok_or_else(|| Error::JsonInvalidIndexError)
    }
}
