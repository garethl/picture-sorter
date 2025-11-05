use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

use crate::kv_store::KVStore;

#[derive(Debug, Clone)]
pub struct Cache {
    store: KVStore<String, String>,
}

impl Cache {
    pub fn new(cache_path: &str) -> Result<Self> {
        Ok(Cache {
            store: KVStore::new(cache_path)?,
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
            Ok(serialized) => Ok(serde_json::from_str::<P>(&serialized)?),
            Err(err) => Err(err),
        }
    }

    pub async fn get_async<'a, P, Fut>(&self, path: &str, f: impl FnOnce() -> Fut) -> Result<P>
    where
        P: Serialize + DeserializeOwned,
        Fut: Future<Output = Result<P>>,
    {
        let result = self
            .store
            .get_or_async(&path.to_owned(), async || {
                let value = f().await?;
                let serialized = serde_json::to_string(&value)?;
                Ok(serialized)
            })
            .await;

        match result {
            Ok(serialized) => Ok(serde_json::from_str::<P>(&serialized)?),
            Err(err) => Err(err),
        }
    }
}
