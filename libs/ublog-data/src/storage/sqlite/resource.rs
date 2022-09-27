use rusqlite::{Connection, Row, Rows};

use crate::models::Resource;

pub(crate) fn init_db_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    const INIT_SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS resources (
            name TEXT NOT NULL PRIMARY KEY,
            ty   TEXT NOT NULL,
            data BLOB NOT NULL
        ) WITHOUT ROWID;
    "#;
    conn.execute_batch(INIT_SQL)
}

pub(crate) fn get_resource(
    conn: &Connection,
    name: &str,
) -> Result<Option<Resource>, rusqlite::Error> {
    const SELECT_SQL: &str = r#"
        SELECT name, ty, data
        FROM resources
        WHERE name == ?;
    "#;

    match conn.query_row(SELECT_SQL, (name,), create_resource_from_row) {
        Ok(res) => Ok(Some(res)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err),
    }
}

pub(crate) fn get_resources(conn: &Connection) -> Result<Vec<Resource>, rusqlite::Error> {
    const SELECT_SQL: &str = r#"
        SELECT name, ty
        FROM resources;
    "#;

    let mut stmt = conn.prepare(SELECT_SQL).unwrap();
    let rows = stmt.query(())?;
    create_resources_from_rows(rows)
}

pub(crate) fn insert_resource(
    conn: &Connection,
    resource: &Resource,
) -> Result<(), rusqlite::Error> {
    const INSERT_SQL: &str = r#"
        INSERT INTO resources (name, ty, data)
        VALUES (?, ?, ?);
    "#;

    conn.execute(INSERT_SQL, (&resource.name, &resource.ty, &resource.data))?;
    Ok(())
}

pub(crate) fn delete_resource(conn: &Connection, name: &str) -> Result<(), rusqlite::Error> {
    const DELETE_SQL: &str = r#"
        DELETE FROM resources
        WHERE name == ?;
    "#;

    conn.execute(DELETE_SQL, (name,))?;

    Ok(())
}

fn create_resource_from_row(row: &Row) -> Result<Resource, rusqlite::Error> {
    Ok(Resource {
        name: row.get("name")?,
        ty: row.get("ty")?,
        data: row.get("data")?,
    })
}

fn create_resources_from_rows(rows: Rows) -> Result<Vec<Resource>, rusqlite::Error> {
    rows.mapped(|row| {
        Ok(Resource {
            name: row.get("name")?,
            ty: row.get("ty")?,
            data: Vec::new(),
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_db_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_insert_resource_basic() {
        let conn = init_db_connection();

        let res = Resource {
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        insert_resource(&conn, &res).unwrap();
    }

    #[test]
    fn test_insert_resource_name_conflict() {
        let conn = init_db_connection();

        let res = Resource {
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        insert_resource(&conn, &res).unwrap();

        let res = Resource {
            name: String::from("res"),
            ty: String::from("text/css"),
            data: vec![1, 2, 3, 4],
        };
        let insert_err = insert_resource(&conn, &res).unwrap_err();
        let insert_sqlite_err_code = insert_err.sqlite_error_code().unwrap();
        assert_eq!(
            insert_sqlite_err_code,
            rusqlite::ErrorCode::ConstraintViolation
        );
    }

    #[test]
    fn test_select_basic() {
        let conn = init_db_connection();

        let res = Resource {
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        insert_resource(&conn, &res).unwrap();

        let selected_res = get_resource(&conn, "res").unwrap().unwrap();
        assert_eq!(res.name, selected_res.name);
        assert_eq!(res.ty, selected_res.ty);
        assert_eq!(res.data, selected_res.data);
    }

    #[test]
    fn test_select_not_exist() {
        let conn = init_db_connection();

        let selected_res = get_resource(&conn, "res").unwrap();
        assert!(selected_res.is_none());
    }

    #[test]
    fn test_delete_basic() {
        let conn = init_db_connection();

        let res = Resource {
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        insert_resource(&conn, &res).unwrap();

        delete_resource(&conn, "res").unwrap();

        let selected_res = get_resource(&conn, "res").unwrap();
        assert!(selected_res.is_none());
    }

    #[test]
    fn test_delete_not_exist() {
        let conn = init_db_connection();
        delete_resource(&conn, "res").unwrap();
    }
}
