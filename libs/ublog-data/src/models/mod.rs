pub(crate) mod commit;
pub(crate) mod post;
pub(crate) mod resource;

use std::borrow::Borrow;
use std::sync::RwLock;

use rusqlite::{Connection, Row, Rows};

use crate::db::Pagination;

pub use crate::models::commit::Commit;
pub use crate::models::post::{Post, PostResource};
pub use crate::models::resource::Resource;

pub(crate) trait Model: Sized {
    const OBJECT_NAME: &'static str;

    type SelectKey: ?Sized;

    fn init_db_schema(conn: &Connection) -> Result<(), rusqlite::Error>;

    fn select_one_from<K>(_conn: &RwLock<Connection>, _key: &K) -> Result<Self, rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>,
    {
        panic!(
            "Selecting a(n) {} object is not a supported operation.",
            Self::OBJECT_NAME
        );
    }

    fn select_many_from(
        _conn: &RwLock<Connection>,
        _pagination: &Pagination,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        panic!(
            "Selecting multiple {} objects with pagination is not a supported operation.",
            Self::OBJECT_NAME
        );
    }

    fn insert_into(&mut self, _conn: &RwLock<Connection>) -> Result<(), rusqlite::Error> {
        panic!(
            "Insert a(n) {} object is not a supported operation.",
            Self::OBJECT_NAME
        );
    }

    fn update_into(&mut self, _conn: &RwLock<Connection>) -> Result<(), rusqlite::Error> {
        panic!(
            "Updating a(n) {} object is not a supported operation.",
            Self::OBJECT_NAME
        );
    }

    fn delete_from<K>(_conn: &RwLock<Connection>, _key: &K) -> Result<(), rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>,
    {
        panic!(
            "Deleting a(n) {} object is not a supported operation.",
            Self::OBJECT_NAME
        )
    }

    fn from_row(row: &Row) -> Result<Self, rusqlite::Error>;

    fn from_rows(rows: Rows) -> Result<Vec<Self>, rusqlite::Error> {
        rows.mapped(Self::from_row).collect()
    }
}
