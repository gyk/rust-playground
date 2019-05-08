use std::collections::HashMap;
use std::collections::hash_map::Entry;

use config::{Value as LibConfigValue, Source, ConfigError};

pub const DEFAULT_ID: &str = "_default_";
pub const OVERRIDE_ID: &str = "_override_";

#[derive(Debug)]
pub struct NestedSource<S: Source> {
    inner_source: S,
}

impl<S: Source> NestedSource<S> {
    pub fn from_source(src: S) -> Self {
        Self {
            inner_source: src,
        }
    }
}

impl<S> Source for NestedSource<S>
    where S: Source + Send + Sync,
          S: Clone,
          S: 'static
{
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(NestedSource::from_source(self.inner_source.clone()))
    }

    fn collect(&self) -> Result<HashMap<String, LibConfigValue>, ConfigError> {
        let mut original_map = self.inner_source.collect()?;
        let mut m = HashMap::with_capacity(original_map.len());

        match original_map.entry(OVERRIDE_ID.to_owned()) {
            Entry::Vacant(..) => {
                // The override part is absent
            }
            Entry::Occupied(o) => {
                let over_m = o
                    .remove()
                    .into_table()?;

                for (over_k, over_v) in over_m {
                    m.insert(over_k, over_v);
                }
            }
        };

        m.insert(DEFAULT_ID.to_owned(), LibConfigValue::from(original_map));
        Ok(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use config::{Config as LibConfig, File as LibConfigFile, FileFormat};

    use crate::error::Result;

    #[test]
    fn smoke_nested_source() -> Result<()> {
        let source = NestedSource::from_source(
            LibConfigFile::from_str(
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
                FileFormat::Json)
        );

        let mut cfg = LibConfig::default();
        cfg.merge(source)?;
        println!("config = {:#?}", cfg);
        // After the override part is lifted, the keys with dots transforms into a sequence of
        // identifiers. For example, "childrenswear.co.uk" becomes
        //
        // - childrenswear
        //     - co
        //         - uk

        Ok(())
    }
}
