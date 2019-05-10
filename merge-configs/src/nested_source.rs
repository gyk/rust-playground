use std::collections::HashMap;

use config::{Value as LibConfigValue, Source, ConfigError};

use crate::{DEFAULT_ID, OVERRIDE_ID};

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
        let mut override_map = original_map
            .remove(OVERRIDE_ID)
            .and_then(|m| m.into_table().ok())
            .unwrap_or_else(|| HashMap::with_capacity(1));

        override_map.insert(DEFAULT_ID.to_owned(), LibConfigValue::from(original_map));
        Ok(override_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use config::{Config as LibConfig, File as LibConfigFile};

    use crate::error::Result;

    #[test]
    fn smoke_nested_source() -> Result<()> {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("assets/config.json");
        let source = NestedSource::from_source(LibConfigFile::from(p));

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
