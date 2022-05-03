use super::tx::Transaction;
use crate::ctx::Context;
use crate::dbs::Attach;
use crate::dbs::Executor;
use crate::dbs::Options;
use crate::dbs::Response;
use crate::dbs::Session;
use crate::dbs::Variables;
use crate::err::Error;
use crate::sql;
use crate::sql::query::Query;

/// The underlying datastore instance which stores the dataset.
pub struct Datastore {
	pub(super) inner: Inner,
}

pub(super) enum Inner {
	#[cfg(feature = "kv-echodb")]
	Mem(super::mem::Datastore),
	#[cfg(feature = "kv-indxdb")]
	IxDB(super::ixdb::Datastore),
	#[cfg(feature = "kv-yokudb")]
	File(super::file::Datastore),
	#[cfg(feature = "kv-tikv")]
	TiKV(super::tikv::Datastore),
}

impl Datastore {
	/// Creates a new datastore instance
	///
	/// # Examples
	///
	/// ```rust,no_run
	/// # use surrealdb::Datastore;
	/// # fn main() -> Result<()> {
	/// let ds = Datastore::new("memory")?;
	/// # Ok(())
	/// # }
	/// ```
	///
	/// Or to create a file-backed store:
	///
	/// ```rust,no_run
	/// # use surrealdb::Datastore;
	/// # fn main() -> Result<()> {
	/// let ds = Datastore::new("file://temp.db")?;
	/// # Ok(())
	/// # }
	/// ```
	///
	/// Or to connect to a tikv-backed distributed store:
	///
	/// ```rust,no_run
	/// # use surrealdb::Datastore;
	/// # fn main() -> Result<()> {
	/// let ds = Datastore::new("tikv://127.0.0.1:2379")?;
	/// # Ok(())
	/// # }
	/// ```
	pub async fn new(path: &str) -> Result<Datastore, Error> {
		match path {
			#[cfg(feature = "kv-echodb")]
			"memory" => {
				info!("Starting kvs store in {}", path);
				super::mem::Datastore::new().await.map(|v| Datastore {
					inner: Inner::Mem(v),
				})
			}
			// Parse and initiate an IxDB database
			#[cfg(feature = "kv-indxdb")]
			s if s.starts_with("ixdb:") => {
				info!("Starting kvs store at {}", path);
				let s = s.trim_start_matches("ixdb://");
				super::ixdb::Datastore::new(s).await.map(|v| Datastore {
					inner: Inner::IxDB(v),
				})
			}
			// Parse and initiate an File database
			#[cfg(feature = "kv-yokudb")]
			s if s.starts_with("file:") => {
				info!("Starting kvs store at {}", path);
				let s = s.trim_start_matches("file://");
				super::file::Datastore::new(s).await.map(|v| Datastore {
					inner: Inner::File(v),
				})
			}
			// Parse and initiate an TiKV database
			#[cfg(feature = "kv-tikv")]
			s if s.starts_with("tikv:") => {
				info!("Starting kvs store at {}", path);
				let s = s.trim_start_matches("tikv://");
				super::tikv::Datastore::new(s).await.map(|v| Datastore {
					inner: Inner::TiKV(v),
				})
			}
			// The datastore path is not valid
			_ => unreachable!(),
		}
	}
	/// Create a new transaction
	pub async fn transaction(&self, write: bool, lock: bool) -> Result<Transaction, Error> {
		match &self.inner {
			#[cfg(feature = "kv-echodb")]
			Inner::Mem(v) => {
				let tx = v.transaction(write, lock).await?;
				Ok(Transaction {
					inner: super::tx::Inner::Mem(tx),
				})
			}
			#[cfg(feature = "kv-indxdb")]
			Inner::IxDB(v) => {
				let tx = v.transaction(write, lock).await?;
				Ok(Transaction {
					inner: super::tx::Inner::IxDB(tx),
				})
			}
			#[cfg(feature = "kv-yokudb")]
			Inner::File(v) => {
				let tx = v.transaction(write, lock).await?;
				Ok(Transaction {
					inner: super::tx::Inner::File(tx),
				})
			}
			#[cfg(feature = "kv-tikv")]
			Inner::TiKV(v) => {
				let tx = v.transaction(write, lock).await?;
				Ok(Transaction {
					inner: super::tx::Inner::TiKV(tx),
				})
			}
		}
	}
	/// Execute a query
	pub async fn execute(
		&self,
		txt: &str,
		sess: &Session,
		vars: Variables,
	) -> Result<Vec<Response>, Error> {
		// Create a new query options
		let mut opt = Options::default();
		// Create a new query executor
		let mut exe = Executor::new(self);
		// Create a default context
		let ctx = Context::default();
		// Start an execution context
		let ctx = sess.context(ctx);
		// Store the query variables
		let ctx = vars.attach(ctx);
		// Parse the SQL query text
		let ast = sql::parse(txt)?;
		// Freeze the context
		let ctx = ctx.freeze();
		// Process all statements
		opt.auth = sess.au.clone();
		opt.ns = sess.ns();
		opt.db = sess.db();
		exe.execute(ctx, opt, ast).await
	}
	/// Execute a query
	pub async fn process(
		&self,
		ast: Query,
		sess: &Session,
		vars: Variables,
	) -> Result<Vec<Response>, Error> {
		// Create a new query options
		let mut opt = Options::default();
		// Create a new query executor
		let mut exe = Executor::new(self);
		// Create a default context
		let ctx = Context::default();
		// Start an execution context
		let ctx = sess.context(ctx);
		// Store the query variables
		let ctx = vars.attach(ctx);
		// Freeze the context
		let ctx = ctx.freeze();
		// Process all statements
		opt.auth = sess.au.clone();
		opt.ns = sess.ns();
		opt.db = sess.db();
		exe.execute(ctx, opt, ast).await
	}
}