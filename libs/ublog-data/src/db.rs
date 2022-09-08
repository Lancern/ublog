use std::borrow::Borrow;
use std::path::Path;
use std::sync::RwLock;

use rusqlite::Connection;

use crate::models::post::PostModelExt;
use crate::models::Model;
use crate::models::{Post, PostResource, Resource};

/// A database connection.
#[derive(Debug)]
pub struct Database {
    conn: RwLock<Connection>,
}

impl Database {
    /// Create a new database connection that connects to the database backed at the specified file.
    pub fn new<P>(path: P) -> Result<Self, rusqlite::Error>
    where
        P: AsRef<Path>,
    {
        let conn = Connection::open(path)?;
        Self::new_from_sqlite_connection(conn)
    }

    /// Create a new database connection with the given underlying sqlite database connection.
    pub fn new_from_sqlite_connection(mut conn: Connection) -> Result<Self, rusqlite::Error> {
        let trans = conn.transaction()?;

        macro_rules! init_db_schema {
            ( $trans:expr, $($t:ty),* $(,)? ) => {
                $(
                    <$t>::init_db_schema($trans)?;
                )*
            };
        }
        init_db_schema!(&trans, Post, PostResource, Resource);

        trans.commit()?;

        Ok(Self {
            conn: RwLock::new(conn),
        })
    }

    /// Get the post object with the given slug.
    pub fn get_post<T>(&self, slug: T) -> Result<Post, rusqlite::Error>
    where
        T: AsRef<str>,
    {
        let slug = slug.as_ref();
        self.select_one(slug)
    }

    /// Get a view of post objects within the specified page.
    pub fn get_posts(&self, pagination: &Pagination) -> Result<Vec<Post>, rusqlite::Error> {
        self.select_many(pagination)
    }

    /// Insert the given post into the database.
    ///
    /// The `id` field is ignored when inserting the post object. It will be set to the post's identifier before this
    /// function returns.
    pub fn insert_post(&self, post: &mut Post) -> Result<(), rusqlite::Error> {
        self.insert_one(post)
    }

    /// Update the specified post object.
    pub fn update_post<T>(&self, post: &mut Post) -> Result<(), rusqlite::Error>
    where
        T: AsRef<str>,
    {
        self.update_one(post)
    }

    /// Set the views count of the specified post.
    pub fn update_post_views<T>(&self, post_slug: T, views: u64) -> Result<(), rusqlite::Error>
    where
        T: AsRef<str>,
    {
        let slug = post_slug.as_ref();
        Post::update_views_into(&self.conn, slug, views)
    }

    /// Delete the post object with the given slug.
    pub fn delete_post<T>(&self, slug: T) -> Result<(), rusqlite::Error>
    where
        T: AsRef<str>,
    {
        let slug = slug.as_ref();
        self.delete_one::<Post, _>(slug)
    }

    /// Insert a new post resource object.
    pub fn insert_post_resource(&self, post_res: &mut PostResource) -> Result<(), rusqlite::Error> {
        self.insert_one(post_res)
    }

    /// Delete a post resource object.
    pub fn delete_post_resource<N>(&self, post_id: i64, res_name: N) -> Result<(), rusqlite::Error>
    where
        N: AsRef<str>,
    {
        let res_name = res_name.as_ref();
        self.delete_one::<PostResource, _>(&(post_id, String::from(res_name)))
    }

    /// Get the static resource object with the given name.
    pub fn get_resource<N>(&self, name: N) -> Result<Resource, rusqlite::Error>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.select_one(name)
    }

    /// Insert the given resource object into the database.
    ///
    /// Due to design problems, this function receives a `&mut Resource`. But this function will not change any fields
    /// of the given [`Resource`] object.
    pub fn insert_resource(&self, res: &mut Resource) -> Result<(), rusqlite::Error> {
        self.insert_one(res)
    }

    /// Delete the resource object with the given name.
    pub fn delete_resource<N>(&self, name: N) -> Result<(), rusqlite::Error>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.delete_one::<Resource, _>(name)
    }

    fn select_one<M, K>(&self, key: &K) -> Result<M, rusqlite::Error>
    where
        M: Model,
        K: ?Sized + Borrow<M::SelectKey>,
    {
        M::select_one_from(&self.conn, key)
    }

    fn select_many<M>(&self, pagination: &Pagination) -> Result<Vec<M>, rusqlite::Error>
    where
        M: Model,
    {
        M::select_many_from(&self.conn, pagination)
    }

    fn insert_one<M>(&self, model: &mut M) -> Result<(), rusqlite::Error>
    where
        M: Model,
    {
        model.insert_into(&self.conn)
    }

    fn update_one<M>(&self, model: &mut M) -> Result<(), rusqlite::Error>
    where
        M: Model,
    {
        model.update_into(&self.conn)
    }

    fn delete_one<M, K>(&self, key: &K) -> Result<(), rusqlite::Error>
    where
        M: Model,
        K: ?Sized + Borrow<M::SelectKey>,
    {
        M::delete_from(&self.conn, key)
    }
}

/// Pagination parameters.
#[derive(Clone, Copy, Debug)]
pub struct Pagination {
    page: usize,
    page_size: usize,
}

impl Pagination {
    /// Create a new [`Pagination`] object.
    ///
    /// `page` gives the page number starting from 1. `page_size` gives the number of items displayed on each page.
    pub fn from_page_and_size(page: usize, page_size: usize) -> Self {
        assert!(page > 0);
        assert!(page_size > 0);
        assert!((page - 1).checked_mul(page_size).is_some());

        Self { page, page_size }
    }

    /// Get the page number. Page numbers start from 1.
    pub fn page(&self) -> usize {
        self.page
    }

    /// Get the number of items on each page.
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// Get the number of items before the first element of the specified page.
    pub fn skip_count(&self) -> usize {
        (self.page - 1) * self.page_size
    }
}
