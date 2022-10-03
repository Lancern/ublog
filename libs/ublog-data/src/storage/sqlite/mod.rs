mod commit;
mod post;
mod resource;

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use async_trait::async_trait;
use rusqlite::Connection;

use crate::models::{Commit, CommitPayload, Delta, Post, PostResource, Resource};
use crate::storage::{Pagination, Storage};

/// Provide sqlite-based storage for databases.
#[derive(Debug)]
pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    /// Create a new `SqliteStorage` from the given sqlite connection.
    pub fn new(conn: Connection) -> Result<Self, rusqlite::Error> {
        init_db_schema(&conn)?;

        let conn = Mutex::new(conn);
        Ok(Self { conn })
    }

    /// Create a new sqlite connection to the specified sqlite database file and then create a new `SqliteStorage` from
    /// that sqlite connection.
    pub fn new_file<P>(path: P) -> Result<Self, rusqlite::Error>
    where
        P: AsRef<Path>,
    {
        let conn = Connection::open(path)?;
        Self::new(conn)
    }

    /// Create a new in-memory sqlite connection and then create a new `SqliteStorage` from that sqlite connection.
    pub fn new_memory() -> Result<Self, rusqlite::Error> {
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
    ) -> Result<(), rusqlite::Error>
    where
        T: IntoIterator<Item = CommitPayload>,
        F: FnOnce(&Connection) -> Result<(), rusqlite::Error>,
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
    type Error = rusqlite::Error;

    async fn insert_post(
        &self,
        post: &Post,
        post_resources: &[PostResource],
    ) -> Result<(), Self::Error> {
        let commit_payload = CommitPayload::create_post(post.slug.clone());
        self.transact_and_commit([commit_payload], |conn| {
            crate::storage::sqlite::post::insert_post(conn, post, post_resources)
        })
    }

    async fn update_post(
        &self,
        post: &Post,
        post_resources: &[PostResource],
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
    ) -> Result<Option<(Post, Vec<PostResource>)>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::post::get_post_with_resources(&*conn, post_slug)
    }

    async fn get_posts(&self, pagination: &Pagination) -> Result<Vec<Post>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::post::get_posts(&*conn, pagination)
    }

    async fn get_post_resource(
        &self,
        post_slug: &str,
        resource_name: &str,
    ) -> Result<Option<PostResource>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::post::get_post_resource(&*conn, post_slug, resource_name)
    }

    async fn insert_resource(&self, resource: &Resource) -> Result<(), Self::Error> {
        let commit_payload = CommitPayload::create_resource(resource.name.clone());
        self.transact_and_commit([commit_payload], |conn| {
            crate::storage::sqlite::resource::insert_resource(conn, resource)
        })
    }

    async fn delete_resource(&self, resource_name: &str) -> Result<(), Self::Error> {
        let commit_payload = CommitPayload::delete_resource(resource_name);
        self.transact_and_commit([commit_payload], |conn| {
            crate::storage::sqlite::resource::delete_resource(conn, resource_name)
        })
    }

    async fn get_resource(&self, resource_name: &str) -> Result<Option<Resource>, Self::Error> {
        let conn = self.lock();
        crate::storage::sqlite::resource::get_resource(&*conn, resource_name)
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

        for name in &delta.deleted_resource_names {
            crate::storage::sqlite::resource::delete_resource(&trans, name)?;
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

fn init_db_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    crate::storage::sqlite::commit::init_db_schema(conn)?;
    crate::storage::sqlite::post::init_db_schema(conn)?;
    crate::storage::sqlite::resource::init_db_schema(conn)?;

    Ok(())
}
