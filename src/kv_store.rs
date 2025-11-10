use std::{marker::PhantomData, time::Duration};

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct KVStore<K, V> {
    pool: Pool<SqliteConnectionManager>,
    _key: PhantomData<K>,
    _value: PhantomData<V>,
}

pub trait Convertable: Sized {
    fn to_raw(&self) -> &[u8];
    fn from_raw(raw: &[u8]) -> Result<Self>;
}

impl Convertable for Vec<u8> {
    fn to_raw(&self) -> &[u8] {
        self
    }

    fn from_raw(raw: &[u8]) -> Result<Self> {
        Ok(Vec::from(raw))
    }
}

impl Convertable for String {
    fn to_raw(&self) -> &[u8] {
        self.as_str().as_bytes()
    }

    fn from_raw(raw: &[u8]) -> Result<Self> {
        Ok(String::from_utf8(raw.to_vec())?)
    }
}

impl<K, V> KVStore<K, V>
where
    K: Convertable,
    V: Convertable,
{
    pub fn new(store_file_name: &str) -> Result<KVStore<K, V>> {
        let manager = SqliteConnectionManager::file(store_file_name).with_init(|c| {
            c.execute_batch(
                "PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;",
            )
        });
        let pool = Pool::builder()
            .idle_timeout(Some(Duration::from_millis(10000)))
            .connection_timeout(Duration::from_millis(1000))
            .min_idle(Some(1))
            .build(manager)?;

        pool.get()?.execute(
            "CREATE TABLE IF NOT EXISTS data(key BLOB PRIMARY KEY, value BLOB);",
            (),
        )?;

        Ok(KVStore {
            pool,
            _key: PhantomData,
            _value: PhantomData,
        })
    }

    pub fn get(&self, key: &K) -> Result<Option<V>> {
        let key = key.to_raw();
        let result = self.get_internal(key)?;
        result.map(|value| V::from_raw(&value)).transpose()
    }

    fn get_internal(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let connection = self.pool.get()?;
        let mut cmd = connection.prepare_cached("SELECT value FROM data where key = ? LIMIT 1")?;

        let mut rows = cmd.query(params![key])?;

        match rows.next()? {
            Some(row) => {
                let value: Vec<u8> = row.get(0)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn get_or(&self, key: &K, f: impl FnOnce() -> Result<V>) -> Result<V> {
        let key = key.to_raw();
        let value = self.get_internal(key)?;

        let value = match value {
            Some(value) => V::from_raw(&value)?,
            None => {
                let value = f()?;
                let raw_value = V::to_raw(&value);
                self.set_internal(key, raw_value)?;
                value
            }
        };

        Ok(value)
    }

    pub async fn get_or_async<Fut>(&self, key: &K, f: impl FnOnce() -> Fut) -> Result<V>
    where
        Fut: Future<Output = Result<V>>,
    {
        let key = key.to_raw();
        let value = self.get_internal(key)?;

        let value = match value {
            Some(value) => V::from_raw(&value)?,
            None => {
                let value = f().await?;
                let raw_value = V::to_raw(&value);
                self.set_internal(key, raw_value)?;
                value
            }
        };

        Ok(value)
    }

    pub fn set(&self, key: &K, value: &V) -> Result<()> {
        let key = key.to_raw();
        let value = value.to_raw();
        self.set_internal(key, value)
    }

    fn set_internal(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let connection = self.pool.get()?;
        let mut cmd = connection.prepare_cached("INSERT INTO data(key, value) VALUES(?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value;")?;

        cmd.execute(params![key, value])?;

        Ok(())
    }

    pub fn delete(&self, key: &K) -> Result<bool> {
        let key = key.to_raw();

        let connection = self.pool.get()?;
        let mut cmd = connection.prepare_cached("DELETE FROM data WHERE key = ?;")?;

        let rows_changed = cmd.execute(params![key])?;

        Ok(rows_changed > 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() -> Result<()> {
        let store = KVStore::new(":memory:")?;
        store.set(
            &"test".to_owned().into_bytes(),
            &"this is a test".to_owned().into_bytes(),
        )?;
        let result = store.get(&"test".to_owned().into_bytes())?;
        assert_eq!(Some("this is a test".to_owned().into_bytes()), result);

        Ok(())
    }

    #[test]
    fn string() -> Result<()> {
        let store = KVStore::new(":memory:")?;

        store.set(&"test".to_string(), &"value".to_string())?;

        let result = store.get(&"test".to_string())?;
        assert_eq!(Some("value".to_owned()), result);
        Ok(())
    }

    #[test]
    fn get_or() -> Result<()> {
        let store = KVStore::new(":memory:")?;

        let result = store.get_or(&"test".to_string(), || Ok("value".to_string()))?;
        assert_eq!("value".to_owned(), result);

        let result = store.get(&"test".to_string())?;
        assert_eq!(Some("value".to_owned()), result);
        Ok(())
    }
}
