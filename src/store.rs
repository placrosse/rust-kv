use std::collections::HashMap;
use std::marker::PhantomData;

use lmdb;

use crate::config::{Config, DatabaseFlags};
use crate::error::Error;
use crate::txn::Txn;
use crate::types::{Integer, Key, Value};

/// A Store is used to keep data on disk using LMDB
pub struct Store {
    env: lmdb::Environment,
    buckets: HashMap<Option<String>, DatabaseFlags>,

    /// The `config` field stores the initial configuration values for the given store
    pub cfg: Config,
}

/// A Bucket represents a single database, or section of the Store
pub struct Bucket<'a, K: Key, V: 'a + Value<'a>>(
    lmdb::Database,
    PhantomData<K>,
    PhantomData<&'a V>,
);

impl<'a, K: Key, V: Value<'a>> Bucket<'a, K, V> {
    /// Provides access to the underlying LMDB dbi handle
    pub fn db(&self) -> lmdb::Database {
        self.0
    }
}

impl Store {
    pub(crate) fn wrap(env: lmdb::Environment, config: Config) -> Store {
        let mut store = Store {
            env,
            buckets: HashMap::new(),
            cfg: config,
        };

        let mut initialized_default = false;

        for (bucket_name, flag) in &store.cfg.buckets {
            let name = if bucket_name == "default" {
                initialized_default = true;
                None
            } else {
                Some(bucket_name.clone())
            };

            store.buckets.insert(name, flag.database_flags());
        }

        if !initialized_default {
            store.buckets.insert(None, lmdb::DatabaseFlags::empty());
        }

        store
    }

    /// Create a new store with the given configuration
    pub fn new(mut config: Config) -> Result<Store, Error> {
        let env = config.env()?;
        Ok(Self::wrap(env, config))
    }

    /// Get a named bucket
    pub fn bucket<'a, K: Key, V: Value<'a>>(
        &self,
        name: Option<&str>,
    ) -> Result<Bucket<'a, K, V>, Error> {
        let n = name.map(String::from);
        match self.buckets.get(&n) {
            Some(flags) => Ok(Bucket(
                self.env.create_db(name, *flags)?,
                PhantomData,
                PhantomData,
            )),
            None => Err(Error::InvalidBucket),
        }
    }

    /// Get a named bucket
    pub fn int_bucket<'a, V: Value<'a>>(
        &self,
        name: Option<&str>,
    ) -> Result<Bucket<'a, Integer, V>, Error> {
        let n = name.map(String::from);
        match self.buckets.get(&n) {
            Some(flags) => {
                let mut f = *flags;
                f.insert(lmdb::DatabaseFlags::INTEGER_KEY);
                Ok(Bucket(
                    self.env.create_db(name, f)?,
                    PhantomData,
                    PhantomData,
                ))
            }
            None => Err(Error::InvalidBucket),
        }
    }

    /// Create a readonly transaction and pass it to the provided function
    pub fn with_read_txn<'a, Res, F: FnOnce(&Txn<'a>) -> Result<Res, Error>>(
        &'a self,
        f: F,
    ) -> Result<Res, Error> {
        let txn = self.read_txn()?;
        f(&txn)
    }

    /// Create a writable transaction and pass it to the provided function
    pub fn with_write_txn<'a, Res, F: FnOnce(&mut Txn<'a>) -> Result<Res, Error>>(
        &'a mut self,
        f: F,
    ) -> Result<Res, Error> {
        let mut txn = self.write_txn()?;
        let res = f(&mut txn)?;
        txn.commit()?;
        Ok(res)
    }

    #[inline]
    /// Open a readonly transaction
    pub fn read_txn(&self) -> Result<Txn, Error> {
        let txn = self.env.begin_ro_txn()?;
        Ok(Txn::read_only(txn))
    }

    #[inline]
    /// Open a writable transaction
    pub fn write_txn(&self) -> Result<Txn, Error> {
        if self.cfg.readonly {
            return Err(Error::ReadOnly);
        }

        let txn = self.env.begin_rw_txn()?;
        Ok(Txn::read_write(txn))
    }

    #[inline]
    /// Sync data to disk
    pub fn sync(&self, force: bool) -> Result<(), Error> {
        self.env.sync(force)?;
        Ok(())
    }

    #[inline]
    /// Get database statistics
    pub fn stat(&self) -> Result<lmdb::Stat, Error> {
        Ok(self.env.stat()?)
    }
}
