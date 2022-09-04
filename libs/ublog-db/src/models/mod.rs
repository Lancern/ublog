pub(crate) mod post;
pub(crate) mod resource;

use std::borrow::Borrow;
use std::sync::RwLock;

use rusqlite::{Connection, Row, Rows};

use crate::Pagination;

pub(crate) trait Model: Sized {
    type SelectKey: ?Sized;
    type UpdateMask: ?Sized;

    fn init_db_schema(conn: &Connection) -> Result<(), rusqlite::Error>;

    fn select_one_from<K>(conn: &RwLock<Connection>, key: &K) -> Result<Self, rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>;

    fn select_many_from(
        conn: &RwLock<Connection>,
        pagination: &Pagination,
    ) -> Result<Vec<Self>, rusqlite::Error>;

    fn insert_into(&mut self, conn: &RwLock<Connection>) -> Result<(), rusqlite::Error>;

    fn update_into<K>(
        &mut self,
        conn: &RwLock<Connection>,
        key: &K,
        mask: &Self::UpdateMask,
    ) -> Result<(), rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>;

    fn delete_from<K>(conn: &RwLock<Connection>, key: &K) -> Result<(), rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>;

    fn from_row(row: &Row) -> Result<Self, rusqlite::Error>;

    fn from_rows(rows: Rows) -> Result<Vec<Self>, rusqlite::Error> {
        rows.mapped(Self::from_row).collect()
    }
}
