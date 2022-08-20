use std::borrow::Borrow;
use std::sync::RwLock;

use rusqlite::{Connection, Row};
use ublog_models::resource::Resource;

use crate::models::Model;
use crate::Pagination;

impl Model for Resource {
    type SelectKey = str;
    type UpdateMask = ();

    fn init_db_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        const INIT_SQL: &str = r#"
            CREATE TABLE IF NOT EXISTS resources (
                name TEXT NOT NULL PRIMARY KEY,
                ty   TEXT NOT NULL,
                data BLOB NOT NULL
            ) WITHOUT ROWID;
        "#;
        conn.execute_batch(INIT_SQL)
    }

    fn select_one_from<K>(conn: &RwLock<Connection>, key: &K) -> Result<Self, rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>,
    {
        const SELECT_SQL: &str = r#"
            SELECT name, ty, data FROM resources
            WHERE name == ?;
        "#;

        let conn = conn.read().unwrap();

        let name: &str = key.borrow();
        conn.query_row(SELECT_SQL, (name,), Self::from_row)
    }

    fn select_many_from(
        _conn: &RwLock<Connection>,
        _pagination: &Pagination,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        panic!("Selecting many resource objects from database is not a supported operation");
    }

    fn insert_into(&mut self, conn: &RwLock<Connection>) -> Result<(), rusqlite::Error> {
        const INSERT_SQL: &str = r#"
            INSERT INTO resources (name, ty, data)
            VALUES (?, ?, ?);
        "#;

        let conn = conn.read().unwrap();

        conn.execute(INSERT_SQL, (&self.name, &self.ty, &self.data))?;
        Ok(())
    }

    fn update_into(
        &mut self,
        _conn: &RwLock<Connection>,
        _mask: &Self::UpdateMask,
    ) -> Result<(), rusqlite::Error> {
        panic!("Updating resource object into database is not a supported operation");
    }

    fn delete_from<K>(conn: &RwLock<Connection>, key: &K) -> Result<(), rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>,
    {
        const DELETE_SQL: &str = r#"
            DELETE FROM resources
            WHERE name == ?;
        "#;

        let conn = conn.read().unwrap();

        let name: &str = key.borrow();
        conn.execute(DELETE_SQL, (name,))?;

        Ok(())
    }

    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Resource {
            name: row.get("name")?,
            ty: row.get("ty")?,
            data: row.get("data")?,
        })
    }
}
