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

#[cfg(test)]
mod tests {
    use super::*;

    fn init_db_connection() -> RwLock<Connection> {
        let conn = Connection::open_in_memory().unwrap();
        Resource::init_db_schema(&conn).unwrap();
        RwLock::new(conn)
    }

    #[test]
    fn test_insert_resource_basic() {
        let conn = init_db_connection();

        let mut res = Resource {
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        res.insert_into(&conn).unwrap();
    }

    #[test]
    fn test_insert_resource_name_conflict() {
        let conn = init_db_connection();

        let mut res = Resource {
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        res.insert_into(&conn).unwrap();

        res = Resource {
            name: String::from("res"),
            ty: String::from("text/css"),
            data: vec![1, 2, 3, 4],
        };
        let insert_err = res.insert_into(&conn).unwrap_err();
        let insert_sqlite_err_code = insert_err.sqlite_error_code().unwrap();
        assert_eq!(
            insert_sqlite_err_code,
            rusqlite::ErrorCode::ConstraintViolation
        );
    }

    #[test]
    fn test_select_basic() {
        let conn = init_db_connection();

        let mut res = Resource {
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        res.insert_into(&conn).unwrap();

        let selected_res = Resource::select_one_from(&conn, "res").unwrap();
        assert_eq!(res.name, selected_res.name);
        assert_eq!(res.ty, selected_res.ty);
        assert_eq!(res.data, selected_res.data);
    }

    #[test]
    fn test_select_not_exist() {
        let conn = init_db_connection();

        let select_err = Resource::select_one_from(&conn, "res").unwrap_err();
        match select_err {
            rusqlite::Error::QueryReturnedNoRows => {}
            _ => panic!("Unexpected error returned"),
        }
    }

    #[test]
    fn test_delete_basic() {
        let conn = init_db_connection();

        let mut res = Resource {
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        res.insert_into(&conn).unwrap();

        Resource::delete_from(&conn, "res").unwrap();

        let select_err = Resource::select_one_from(&conn, "res").unwrap_err();
        match select_err {
            rusqlite::Error::QueryReturnedNoRows => {}
            _ => panic!("Unexpected error returned"),
        }
    }

    #[test]
    fn test_delete_not_exist() {
        let conn = init_db_connection();
        Resource::delete_from(&conn, "res").unwrap();
    }
}
