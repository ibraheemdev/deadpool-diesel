#![deny(rust_2018_idioms)]

use std::fmt;
use std::marker::PhantomData;

use diesel::prelude::*;

use deadpool::managed::{Manager, RecycleResult};

use async_trait::async_trait;

mod rt;

pub struct ConnectionManager<T> {
    database_url: String,
    _marker: PhantomData<T>,
}

impl<T> fmt::Debug for ConnectionManager<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConnectionManager<{}>", std::any::type_name::<T>())
    }
}

impl<T> ConnectionManager<T> {
    pub fn new(database_url: impl Into<String>) -> Self {
        ConnectionManager {
            database_url: database_url.into(),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    ConnectionError(ConnectionError),
    QueryError(diesel::result::Error),
    SpawnError(rt::JoinError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::ConnectionError(ref e) => e.fmt(f),
            Error::QueryError(ref e) => e.fmt(f),
            Error::SpawnError(ref e) => e.fmt(f)
        }
    }
}

impl std::error::Error for Error {}

pub trait DeadpoolConnection: Connection {
    fn ping(&self) -> QueryResult<()>;
}

#[cfg(feature = "postgres")]
impl DeadpoolConnection for diesel::pg::PgConnection {
    fn ping(&self) -> QueryResult<()> {
        self.execute("SELECT 1").map(|_| ())
    }
}

#[cfg(feature = "mysql")]
impl DeadpoolConnection for diesel::mysql::MysqlConnection {
    fn ping(&self) -> QueryResult<()> {
        self.execute("SELECT 1").map(|_| ())
    }
}

#[cfg(feature = "sqlite")]
impl DeadpoolConnection for diesel::sqlite::SqliteConnection {
    fn ping(&self) -> QueryResult<()> {
        self.execute("SELECT 1").map(|_| ())
    }
}

#[async_trait]
impl<T> Manager<T, Error> for ConnectionManager<T>
where
    T: DeadpoolConnection + Send + Sync + 'static,
{
    async fn create(&self) -> Result<T, Error> {
        let url = self.database_url.clone();
        rt::spawn_blocking(move || T::establish(&url).map_err(Error::ConnectionError))
            .await
            .map_err(Error::SpawnError)?
    }

    async fn recycle(&self, conn: &mut T) -> RecycleResult<Error> {
        conn.ping().map_err(Error::QueryError).map_err(|e| e.into())
    }
}
