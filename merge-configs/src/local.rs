use std::iter::Extend;

pub use config::{Config as LibConfig, Value as LibConfigValue};
use serde::Deserialize;
use serde_json::{Value, Map, map::Entry};

use crate::OVERRIDE_ID;
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
    pub fn from_config(cfg: LibConfig) -> Self {
        Self {
            inner_config: cfg,
            default_cache: None,
            override_cache: None,
        }
    }

    pub fn set_default(&mut self, key: &str, value: Value) -> Result<()> {
        let cfg_value = LibConfigValue::deserialize(value)?;
        self.inner_config.set(key, cfg_value)?;
        self.reset_caches();
        Ok(())
    }

    // FIXME: currently `key` cannot contain dots
    pub fn set_override(&mut self, branch: &str, key: &str, value: Value) -> Result<()> {
        let override_key = format!("{}.{}.{}", OVERRIDE_ID, branch, key);
        let cfg_value = LibConfigValue::deserialize(value)?;
        self.inner_config.set(&override_key, cfg_value)?;
        self.reset_caches();
        Ok(())
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

    pub fn fetch_merged(&mut self, branch: &str) -> Result<Value> {
        if self.default_cache.is_none() || self.override_cache.is_none() {
            self.update_caches()?;
        }

        let mut value = self.default_cache()?.clone();
        match (value.as_object_mut(), self.override_cache()?.as_object()) {
            (Some(m_dst), Some(m_src)) => {
                m_dst.extend(
                    m_src
                        .get(branch).expect("missing branch")
                        .as_object().expect("not a JSON map").clone());
            }
            _ => panic!(),
        }

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

        let m = val.as_object_mut().expect("not a JSON map");
        let mut merged = Map::new();

        match m.entry(OVERRIDE_ID) {
            Entry::Vacant(..) => {
                // The override part is absent
            }
            Entry::Occupied(o) => {
                let domain_m = o
                    .get()
                    .as_object()
                    .expect("not a JSON map")
                    .clone();

                for (domain_k, domain_v) in domain_m {
                    merged.insert(domain_k, domain_v);
                }

                o.remove();
            }
        };

        self.default_cache = Some(val);
        self.override_cache = Some(Value::Object(merged));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::SocketAddr;

    use config::{File as LibConfigFile, FileFormat};
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
        let source = LibConfigFile::from_str(
            r#"{
                "server_addr": "127.0.0.1:80",
                "site_name": "Joe's server",
                "rating": 0,

                "_override_": {
                    "itscrap.com": {
                        "server_addr": "184.168.131.241:80",
                        "site_name": "IT scrap",
                        "rating": 5
                    },
                    "whorepresents.com": {
                        "site_name": "Who represents?",
                        "rating": 95
                    },
                    "childrenswear.co.uk": {
                        "site_name": "Children's wear"
                    },
                    "localhost": {
                        "site_name": "Home"
                    }
                }
            }"#,
            FileFormat::Json);
        let mut cfg = LibConfig::default();
        cfg.merge(source)?;
        println!("config = {:#?}", cfg);
        println!("\n================================\n");

        let mut local = Local::from_config(cfg);
        println!("Local.default = {:#?}", local.default_cache()?);
        println!("Local.override = {:#?}", local.override_cache()?);
        let default_settings = Settings::deserialize(local.default_cache()?.clone())?;
        let merged_settings = Settings::deserialize(local.fetch_merged(r"localhost")?)?;
        println!("Local.default = {:#?}", default_settings);
        println!("Local.merged = {:#?}", merged_settings);

        // modifies config
        local.set_default("server_addr", "192.168.1.1:80".into())?;
        local.set_override(r"localhost", "site_name", "My home".into())?;
        local.set_override("localhost", "rating", 1024.into())?;

        println!("\n================================\nAfter modification:\n");
        let default_settings = Settings::deserialize(local.default_cache()?.clone())?;
        let merged_settings = Settings::deserialize(local.fetch_merged("localhost")?)?;
        println!("Local.default = {:#?}", default_settings);
        println!("Local.merged = {:#?}", merged_settings);

        Ok(())
    }
}
