use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

use crate::kv_store::KVStore;

#[derive(Debug, Clone)]
pub struct Cache {
    store: KVStore<String, String>,
}

impl Cache {
    pub fn new(cache_dir: String) -> Result<Self> {
        Ok(Cache {
            store: KVStore::new(&cache_dir)?,
        })
    }

    pub fn get<'a, P>(&self, path: &str, f: impl FnOnce() -> Result<P>) -> Result<P>
    where
        P: Serialize + DeserializeOwned,
    {
        let result = self.store.get_or(&path.to_owned(), || {
            let value = f()?;
            let serialized = serde_json::to_string(&value)?;
            Ok(serialized)
        });

        match result {
            Ok(serialized) => Ok(serde_json::from_str::<P>(&*serialized)?),
            Err(err) => Err(err),
        }
    }
}
