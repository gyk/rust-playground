use std::iter::Extend;

pub use config::{Config as LibConfig, Value as LibConfigValue, Source};
use serde::Deserialize;
use serde_json::Value;

use crate::{DEFAULT_ID, OVERRIDE_ID};
use crate::nested_source::NestedSource;
use crate::value_ext::ValueExt;
use crate::error::Result;

// FIXME: config-rs does not support keys with dots and JSONPath's `[valid.key.with.dots]` access
// pattern. See <https://github.com/mehcode/config-rs/issues/110>.

#[derive(Debug)]
pub struct Local {
    inner_config: LibConfig,
    cache: Option<Value>,
}

impl Local {
    pub fn new() -> Self {
        Self {
            inner_config: LibConfig::new(),
            cache: None,
        }
    }

    pub fn merge<T>(&mut self, source: T) -> Result<&mut Self>
        where
            T: Source + Send + Sync,
            T: Clone,
            T: 'static,
    {
        self.inner_config.merge(NestedSource::from_source(source))?;
        Ok(self)
    }

    // FIXME: currently `key` cannot contain dots
    pub fn set_default_by(&mut self, key: &str, value: Value) -> Result<()> {
        self.set_override_by(DEFAULT_ID, key, value)
    }

    // FIXME: currently `key` cannot contain dots
    pub fn set_override_by(&mut self, branch: &str, key: &str, value: Value) -> Result<()> {
        let cfg_value = LibConfigValue::deserialize(value)?;
        let override_key = format!("{}.{}", branch, key);
        self.inner_config.set(&override_key, cfg_value)?;
        self.reset_cache();
        Ok(())
    }

    // FIXME: currently `key` cannot contain dots
    pub fn get_default_by(&mut self, key: &str) -> Result<Value> {
        self.get_branch_by(DEFAULT_ID, key)
    }

    // FIXME: currently `key` cannot contain dots
    pub fn get_branch_by(&mut self, branch: &str, key: &str) -> Result<Value> {
        let override_key = format!("{}.{}", branch, key);
        Ok(self.inner_config.get(&override_key)?)
    }

    pub fn get_default(&mut self) -> Result<Value> {
        self.get_branch(DEFAULT_ID)
    }

    pub fn get_branch(&mut self, branch: &str) -> Result<Value> {
        Ok(self.inner_config.get(&branch)?)
    }

    pub fn cache(&mut self) -> Result<&Value> {
        if self.cache.is_none() {
            self.update_cache()?;
        }

        Ok(self.cache.as_ref().unwrap())
    }

    pub fn default_cache(&mut self) -> Result<&Value> {
        let cache = self.cache()?;
        cache.get_value(DEFAULT_ID)
    }

    pub fn override_cache(&mut self) -> Result<&Value> {
        let cache = self.cache()?;
        cache.get_value(OVERRIDE_ID)
    }

    // FIXME: currently `branch` cannot contain dots
    pub fn get_merged(&mut self, branch: &str) -> Result<Value> {
        if self.cache.is_none() {
            self.update_cache()?;
        }

        let mut value = self.cache()?.clone();
        let m_dst = value.as_map_mut()?;
        let m_src = self.cache()?.get_value(branch)?.as_map()?;
        // FIXME: wrong implementation. It cannot handle nested maps.
        m_dst.extend(m_src.clone());

        Ok(value)
    }

    // WORKAROUND: supports `branch` with dots
    pub fn get_merged2(&mut self, branch: &str) -> Result<Value> {
        let mut value = self.default_cache()?.clone();
        let m_dst = value.as_map_mut()?;
        let v_src = self.get_branch(branch)?;
        let m_src = v_src.as_map()?;
        // FIXME: wrong implementation. It cannot handle nested maps.
        m_dst.extend(m_src.clone());

        Ok(value)
    }

    #[inline]
    fn reset_cache(&mut self) {
        self.cache = None;
    }

    fn update_cache(&mut self) -> Result<()> {
        self.inner_config.refresh()?;
        let cfg = self.inner_config.clone();
        let val = cfg.try_into::<Value>()?;
        self.cache = Some(val);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::SocketAddr;
    use std::path::PathBuf;

    use config::File as LibConfigFile;
    use serde_derive::Deserialize;

    #[test]
    fn smoke_local() -> Result<()> {
        #[derive(Debug, Deserialize)]
        struct Settings {
            pub server_addr: SocketAddr,
            pub site_name: String,
            pub rating: u32,
        }

        // FIXME: Keys with dots happens to work when they are nested. See config-rs issue 110.
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("assets/config.json");
        let source = LibConfigFile::from(p);

        // FIXME: use `get_merged` instead of `get_merged2` when config-rs is fixed.

        let mut local = Local::new();
        local.merge(source)?;
        println!("Local = {:#?}", local);
        println!("Local.cache = {:#?}", local.cache()?);
        let default_settings = Settings::deserialize(local.get_default()?)?;
        let merged_settings = Settings::deserialize(local.get_merged2(r"whorepresents.com")?)?;
        println!("Local.default = {:#?}", default_settings);
        println!("Local.merged = {:#?}", merged_settings);

        // modifies config
        local.set_default_by("server_addr", "192.168.1.1:80".into())?;
        local.set_override_by(r"whorepresents.com", "site_name", "Whore presents".into())?;
        local.set_override_by("whorepresents.com", "rating", 1024.into())?;

        println!("\n================================\nAfter modification:\n");
        // Here deliberately uses `default_cache` rather than `get_default`.
        let default_settings = Settings::deserialize(local.default_cache()?.clone())?;
        let merged_settings = Settings::deserialize(local.get_merged2("whorepresents.com")?)?;
        println!("Local.default = {:#?}", default_settings);
        println!("Local.merged = {:#?}", merged_settings);

        Ok(())
    }
}
