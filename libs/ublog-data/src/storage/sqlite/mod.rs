mod commit;
mod post;
mod resource;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use async_trait::async_trait;
use rusqlite::{Connection, Params, Row};
use uuid::Uuid;

use crate::models::{Commit, CommitPayload, Delta, Post, Resource};
use crate::storage::{PaginatedList, Pagination, Storage};

/// Provide sqlite-based storage for databases.
#[derive(Debug)]
pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    /// Create a new `SqliteStorage` from the given sqlite connection.
    pub fn new(conn: Connection) -> Result<Self, SqliteStorageError> {
        init_db_schema(&conn)?;

        let conn = Mutex::new(conn);
        Ok(Self { conn })
    }

    /// Create a new sqlite connection to the specified sqlite database file and then create a new `SqliteStorage` from
    /// that sqlite connection.
    pub fn new_file<P>(path: P) -> Result<Self, SqliteStorageError>
    where
        P: AsRef<Path>,
    {
        let conn = Connection::open(path)?;
        Self::new(conn)
    }

    /// Create a new in-memory sqlite connection and then create a new `SqliteStorage` from that sqlite connection.
    pub fn new_memory() -> Result<Self, SqliteStorageError> {
        let conn = Connection::open_in_memory()?;
        Self::new(conn)
    }

    fn lock(&self) -> MutexGuard<Connection> {
        self.conn.lock().unwrap()
    }

    fn transact_and_commit<T, F>(
        &self,
        commit_payloads: T,
        transact: F,
    ) -> Result<(), SqliteStorageError>
    where
        T: IntoIterator<Item = CommitPayload>,
        F: FnOnce(&Connection) -> Result<(), SqliteStorageError>,
    {
        let mut conn = self.lock();
        let trans = conn.transaction()?;

        let last_commit = crate::storage::sqlite::commit::get_latest_commit(&*trans)?;
        let mut last_commit_id = last_commit.map(|commit| commit.id).unwrap_or_default();

        transact(&*trans)?;

        for payload in commit_payloads {
            let commit = Commit::new(last_commit_id, payload);
            last_commit_id = commit.id.clone();

            crate::storage::sqlite::commit::insert_commit(&*trans, &commit)?;
        }

        trans.commit()?;

        Ok(())
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    type Error = SqliteStorageError;

    async fn insert_post(
        &self,
        post: &Post,
        post_resources: &[Resource],
    ) -> Result<(), Self::Error> {
        let commit_payload = CommitPayload::create_post(post.slug.clone());
        self.transact_and_commit([commit_payload], |conn| {
            crate::storage::sqlite::post::insert_post(conn, post, post_resources)
        })
    }

    async fn update_post(
        &self,
        post: &Post,
        post_resources: &[Resource],
    ) -> Result<(), Self::Error> {
        let commit_payloads = [
            CommitPayload::delete_post(post.slug.clone()),
            CommitPayload::create_post(post.slug.clone()),
        ];
        self.transact_and_commit(commit_payloads, |conn| {
            crate::storage::sqlite::post::delete_post(conn, &post.slug)?;
            crate::storage::sqlite::post::insert_post(conn, post, post_resources)?;
            Ok(())
        })
    }

    async fn delete_post(&self, post_slug: &str) -> Result<(), Self::Error> {
        let commit_payload = CommitPayload::delete_post(post_slug);
        self.transact_and_commit([commit_payload], |conn| {
            crate::storage::sqlite::post::delete_post(conn, post_slug)
        })
    }

    async fn get_post(&self, post_slug: &str) -> Result<Option<Post>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::post::get_post(&*conn, post_slug)
    }

    async fn get_post_with_resources(
        &self,
        post_slug: &str,
    ) -> Result<Option<(Post, Vec<Resource>)>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::post::get_post_with_resources(&*conn, post_slug)
    }

    async fn get_posts(
        &self,
        special: bool,
        pagination: &Pagination,
    ) -> Result<PaginatedList<Post>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::post::get_posts(&*conn, special, pagination)
    }

    async fn insert_resource(&self, resource: &Resource) -> Result<(), Self::Error> {
        let commit_payload = CommitPayload::create_resource(resource.id);
        self.transact_and_commit([commit_payload], |conn| {
            crate::storage::sqlite::resource::insert_resource(conn, resource)
        })
    }

    async fn delete_resource(&self, resource_id: &Uuid) -> Result<(), Self::Error> {
        let commit_payload = CommitPayload::delete_resource(*resource_id);
        self.transact_and_commit([commit_payload], |conn| {
            crate::storage::sqlite::resource::delete_resource(conn, resource_id)
        })
    }

    async fn get_resource(&self, resource_id: &Uuid) -> Result<Option<Resource>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::resource::get_resource(&*conn, resource_id)
    }

    async fn get_resources(&self) -> Result<Vec<Resource>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::resource::get_resources(&*conn)
    }

    async fn get_commits_since(&self, since_timestamp: i64) -> Result<Vec<Commit>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::commit::get_commits(&*conn, since_timestamp)
    }

    async fn get_latest_commit(&self) -> Result<Option<Commit>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::commit::get_latest_commit(&*conn)
    }

    async fn apply_delta(&self, delta: &Delta) -> Result<(), Self::Error> {
        let mut conn = self.lock();
        let trans = conn.transaction()?;

        for slug in &delta.deleted_post_slugs {
            crate::storage::sqlite::post::delete_post(&trans, slug)?;
        }

        for id in &delta.deleted_resource_ids {
            crate::storage::sqlite::resource::delete_resource(&trans, id)?;
        }

        for (post, post_resources) in &delta.added_posts {
            crate::storage::sqlite::post::insert_post(&trans, post, post_resources)?;
        }

        for resource in &delta.added_resources {
            crate::storage::sqlite::resource::insert_resource(&trans, resource)?;
        }

        crate::storage::sqlite::commit::insert_commits(&trans, &delta.commits)?;

        trans.commit()?;

        Ok(())
    }
}

/// SQlite storage errors.
#[derive(Debug)]
pub enum SqliteStorageError {
    Sqlite(rusqlite::Error),
    Bson(bson::de::Error),
    Uuid(uuid::Error),
}

impl Display for SqliteStorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(err) => write!(f, "sqlite error: {}", err),
            Self::Bson(err) => write!(f, "bson deserialize error: {}", err),
            Self::Uuid(err) => write!(f, "uuid error: {}", err),
        }
    }
}

impl Error for SqliteStorageError {}

impl From<rusqlite::Error> for SqliteStorageError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Sqlite(err)
    }
}

impl From<bson::de::Error> for SqliteStorageError {
    fn from(err: bson::de::Error) -> Self {
        Self::Bson(err)
    }
}

impl From<uuid::Error> for SqliteStorageError {
    fn from(err: uuid::Error) -> Self {
        Self::Uuid(err)
    }
}

fn init_db_schema(conn: &Connection) -> Result<(), SqliteStorageError> {
    crate::storage::sqlite::commit::init_db_schema(conn)?;
    crate::storage::sqlite::post::init_db_schema(conn)?;
    crate::storage::sqlite::resource::init_db_schema(conn)?;

    Ok(())
}

trait SqliteExt {
    fn query_one<S, P, F, T>(
        &self,
        sql: S,
        params: P,
        map_row: F,
    ) -> Result<Option<T>, SqliteStorageError>
    where
        S: AsRef<str>,
        P: Params,
        F: FnOnce(&Row) -> Result<T, SqliteStorageError>;

    fn query_many<S, P, F, T>(
        &self,
        sql: S,
        params: P,
        map_row: F,
    ) -> Result<Vec<T>, SqliteStorageError>
    where
        S: AsRef<str>,
        P: Params,
        F: FnMut(&Row) -> Result<T, SqliteStorageError>;
}

impl SqliteExt for Connection {
    fn query_one<S, P, F, T>(
        &self,
        sql: S,
        params: P,
        map_row: F,
    ) -> Result<Option<T>, SqliteStorageError>
    where
        S: AsRef<str>,
        P: Params,
        F: FnOnce(&Row) -> Result<T, SqliteStorageError>,
    {
        let mut stmt = self.prepare(sql.as_ref()).unwrap();
        let mut rows = stmt.query(params)?;
        rows.next()?.map(map_row).transpose()
    }

    fn query_many<S, P, F, T>(
        &self,
        sql: S,
        params: P,
        mut map_row: F,
    ) -> Result<Vec<T>, SqliteStorageError>
    where
        S: AsRef<str>,
        P: Params,
        F: FnMut(&Row) -> Result<T, SqliteStorageError>,
    {
        let mut stmt = self.prepare(sql.as_ref()).unwrap();
        let mut rows = stmt.query(params)?;
        let mut results = Vec::new();

        while let Some(row) = rows.next()? {
            let item = map_row(row)?;
            results.push(item);
        }

        Ok(results)
    }
}
