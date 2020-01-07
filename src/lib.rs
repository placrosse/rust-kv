#![deny(missing_docs)]

//! `kv` is a simple way to embed a key/value store in Rust applications. It is build on LMDB and
//! aims to be as lightweight as possible, while still providing a nice high level interface.
//!
//! ## Getting started
//!
//! ```rust
//! use kv::{Config, Error, Manager, ValueRef};
//!
//! fn run() -> Result<(), Error> {
//!     // First step create a manager, this ensured that each LMDB environment will only be
//!     // accessed once per process
//!     let mut mgr = Manager::new();
//!
//!     // Next configure a database
//!     let mut cfg = Config::default("/tmp/rust-kv");
//!
//!     // Add a bucket named `test`
//!     cfg.bucket("test", None);
//!
//!     // Get a Store handle
//!     let handle = mgr.open(cfg)?;
//!
//!     // Get read-write access to the underlying store
//!     let store = handle.write()?;
//!
//!     // A Bucket provides typed access to an LMDB database
//!     let bucket = store.bucket::<&str, &str>(Some("test"))?;
//!
//!     {
//!         // Finally, a transaction is needed, they can be read-write or readonly, here we will use a
//!         // write transaction to add data
//!         let mut txn = store.write_txn()?;
//!
//!         // To set a value
//!         let () = txn.set(&bucket, "testing", "abc123")?;
//!
//!         // Make sure to commit the transaction. There is also an `abort` function to abandon
//!         // the transaction
//!         txn.commit()?;
//!     }
//!
//!     {
//!         // This time a readonly transaction
//!         let txn = store.read_txn()?;
//!
//!         // Getting a value is easy once everything is set up
//!         let val = txn.get(&bucket, "testing")?;
//!         println!("testing => {}", val);
//!     }
//!
//!     Ok(())
//! }
//! #
//! # fn main() {
//! #     run().unwrap();
//! # }
//! ```

mod buf;
mod config;
mod cursor;
mod encoding;
mod error;
mod manager;
mod store;
#[cfg(test)]
mod test;
mod txn;
mod types;

#[cfg(feature = "bincode-value")]
pub use crate::encoding::bincode;
#[cfg(feature = "cbor-value")]
pub use crate::encoding::cbor;
#[cfg(feature = "json-value")]
pub use crate::encoding::json;
#[cfg(feature = "msgpack-value")]
pub use crate::encoding::msgpack;

pub use crate::buf::ValueBuf;
pub use crate::config::{Config, DatabaseFlags};
pub use crate::cursor::{Cursor, CursorOp, Iter};
pub use crate::encoding::{Encoding, Serde};
pub use crate::error::Error;
pub use crate::manager::Manager;
pub use crate::store::{Bucket, Store};
pub use crate::txn::Txn;
pub use crate::types::{Integer, Key, Value, ValueMut, ValueRef};
