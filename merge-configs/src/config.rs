#![allow(dead_code)]
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use serde_json::Value;

use crate::local::Local;
use crate::value_ext::ValueExt;
use crate::error::Result;

#[derive(Clone)]
pub struct Config {
    local: Arc<Mutex<Local>>,

    // Currently it has no `remote`, so all of the public APIs just wrap around `local`.
}

impl Config {
    pub fn new(local: Local) -> Self {
        Self {
            local: Arc::new(Mutex::new(local)),
        }
    }

    pub fn set_default_by<T: Into<Value>>(&self, key: &str, value: T) -> Result<()> {
        self.local.lock()?.set_default_by(key, value.into())
    }

    pub fn fetch_default_by<'de, T: Deserialize<'de>>(&self, key: &str) -> Result<T> {
        let mut local = self.local.lock()?;
        let default_cache = local.default_cache()?;
        Ok(T::deserialize(default_cache.get_value(key)?.clone())?)
    }

    pub fn set_override_by<T: Into<Value>>(&self, branch: &str, key: &str, value: T)
        -> Result<()>
    {
        self.local.lock()?.set_override_by(branch, key, value.into())
    }

    pub fn fetch_override_by<'de, T: Deserialize<'de>>(&self, branch: &str, key: &str)
        -> Result<T>
    {
        let mut local = self.local.lock()?;
        let override_cache = local.override_cache()?;
        let override_key = format!("{}.{}", branch, key);
        Ok(T::deserialize(override_cache.get_value(&override_key)?.clone())?)
    }

    pub fn fetch_merged(&self, branch: &str) -> Result<Value> {
        self.local.lock()?.get_merged(branch)
    }

    pub fn fetch_merged2(&self, branch: &str) -> Result<Value> {
        self.local.lock()?.get_merged2(branch)
    }
}
