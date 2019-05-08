use std::iter::Extend;

pub use config::{Config as LibConfig, Value as LibConfigValue, Source};
use serde::Deserialize;
use serde_json::Value;

use crate::DEFAULT_ID;
use crate::nested_source::NestedSource;
use crate::value_ext::ValueExt;
use crate::error::Result;

// FIXME: config-rs does not support keys with dots and JSONPath's `[valid.key.with.dots]` access
// pattern. See <https://github.com/mehcode/config-rs/issues/110>.

#[derive(Debug)]
pub struct Local {
    inner_config: LibConfig,
    default_cache: Option<Value>,
    override_cache: Option<Value>,
}

impl Local {
    pub fn new() -> Self {
        Self {
            inner_config: LibConfig::new(),
            default_cache: None,
            override_cache: None,
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
    pub fn set_default(&mut self, key: &str, value: Value) -> Result<()> {
        let cfg_value = LibConfigValue::deserialize(value)?;
        let default_key = format!("{}.{}", DEFAULT_ID, key);
        self.inner_config.set(&default_key, cfg_value)?;
        self.reset_caches();
        Ok(())
    }

    // FIXME: currently `key` cannot contain dots
    pub fn set_override(&mut self, branch: &str, key: &str, value: Value) -> Result<()> {
        let cfg_value = LibConfigValue::deserialize(value)?;
        let override_key = format!("{}.{}", branch, key);
        self.inner_config.set(&override_key, cfg_value)?;
        self.reset_caches();
        Ok(())
    }

    // FIXME: currently `key` cannot contain dots
    pub fn get_override(&mut self, branch: &str, key: Option<&str>) -> Result<Value> {
        let concat_key;
        let override_key: &str = match key {
            Some(key) => {
                concat_key = format!("{}.{}", branch, key);
                &concat_key
            }
            None => branch,
        };
        Ok(self.inner_config.get(override_key)?)
    }

    pub fn default_cache(&mut self) -> Result<&Value> {
        if self.default_cache.is_none() {
            self.update_caches()?;
        }

        Ok(self.default_cache.as_ref().unwrap())
    }

    pub fn override_cache(&mut self) -> Result<&Value> {
        if self.override_cache.is_none() {
            self.update_caches()?;
        }

        Ok(self.override_cache.as_ref().unwrap())
    }

    // FIXME: currently `branch` cannot contain dots
    pub fn fetch_merged(&mut self, branch: &str) -> Result<Value> {
        if self.default_cache.is_none() || self.override_cache.is_none() {
            self.update_caches()?;
        }

        let mut value = self.default_cache()?.clone();
        let m_dst = value.as_map_mut()?;
        let m_src = self.override_cache()?.get_value(branch)?.as_map()?;
        m_dst.extend(m_src.clone());

        Ok(value)
    }

    // WORKAROUND: supports `branch` with dots
    pub fn fetch_merged2(&mut self, branch: &str) -> Result<Value> {
        if self.default_cache.is_none() || self.override_cache.is_none() {
            self.update_caches()?;
        }

        let mut value = self.default_cache()?.clone();
        let m_dst = value.as_map_mut()?;
        let v_src = self.get_override(branch, None)?;
        let m_src = v_src.as_map()?;
        m_dst.extend(m_src.clone());

        Ok(value)
    }

    fn reset_caches(&mut self) {
        self.default_cache = None;
        self.override_cache = None;
    }

    fn update_caches(&mut self) -> Result<()> {
        self.inner_config.refresh()?;
        let cfg = self.inner_config.clone();
        let mut val = cfg.try_into::<Value>()?;

        let m = val.as_map_mut()?;
        self.default_cache = m.remove(DEFAULT_ID);
        self.override_cache = Some(val);
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

        // FIXME: use `fetch_merged` instead of `fetch_merged2` when config-rs is fixed.

        let mut local = Local::new();
        local.merge(source)?;
        println!("Local = {:#?}", local);
        println!("Local.default = {:#?}", local.default_cache()?);
        println!("Local.override = {:#?}", local.override_cache()?);
        let default_settings = Settings::deserialize(local.default_cache()?.clone())?;
        let merged_settings = Settings::deserialize(local.fetch_merged2(r"whorepresents.com")?)?;
        println!("Local.default = {:#?}", default_settings);
        println!("Local.merged = {:#?}", merged_settings);

        // modifies config
        local.set_default("server_addr", "192.168.1.1:80".into())?;
        local.set_override(r"whorepresents.com", "site_name", "Whore presents".into())?;
        local.set_override("whorepresents.com", "rating", 1024.into())?;

        println!("\n================================\nAfter modification:\n");
        let default_settings = Settings::deserialize(local.default_cache()?.clone())?;
        let merged_settings = Settings::deserialize(local.fetch_merged2("whorepresents.com")?)?;
        println!("Local.default = {:#?}", default_settings);
        println!("Local.merged = {:#?}", merged_settings);

        Ok(())
    }
}
