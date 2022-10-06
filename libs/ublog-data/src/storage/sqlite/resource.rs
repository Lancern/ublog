use rusqlite::{Connection, Row};
use uuid::Uuid;

use crate::models::Resource;
use crate::storage::sqlite::{SqliteExt, SqliteStorageError};

pub(crate) fn init_db_schema(conn: &Connection) -> Result<(), SqliteStorageError> {
    const INIT_SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS resources (
            id   TEXT NOT NULL PRIMARY KEY,
            name TEXT NOT NULL,
            ty   TEXT NOT NULL,
            data BLOB NOT NULL
        ) WITHOUT ROWID;
    "#;
    conn.execute_batch(INIT_SQL)?;

    Ok(())
}

pub(crate) fn get_resource(
    conn: &Connection,
    uuid: &Uuid,
) -> Result<Option<Resource>, SqliteStorageError> {
    const SELECT_SQL: &str = r#"
        SELECT id, name, ty, data
        FROM resources
        WHERE id == ?;
    "#;

    let uuid_str = format!("{}", uuid.as_hyphenated());
    conn.query_one(SELECT_SQL, (&uuid_str,), create_resource_from_row)
}

pub(crate) fn get_resources(conn: &Connection) -> Result<Vec<Resource>, SqliteStorageError> {
    const SELECT_SQL: &str = r#"
        SELECT id, name, ty
        FROM resources;
    "#;

    conn.query_many(SELECT_SQL, (), create_resources_from_row_no_data)
}

pub(crate) fn get_post_resources(
    conn: &Connection,
    post_slug: &str,
) -> Result<Vec<Resource>, SqliteStorageError> {
    const SELECT_SQL: &str = r#"
        SELECT id, name, ty, data
        FROM posts_resources JOIN resources ON posts_resources.res_id == resources.id
        where posts_resources.post_slug == ?;
    "#;

    conn.query_many(SELECT_SQL, (post_slug,), create_resource_from_row)
}

pub(crate) fn insert_resource(
    conn: &Connection,
    resource: &Resource,
) -> Result<(), SqliteStorageError> {
    const INSERT_SQL: &str = r#"
        INSERT INTO resources (id, name, ty, data)
        VALUES (?, ?, ?, ?);
    "#;

    let uuid_str = format!("{}", resource.id.as_hyphenated());

    conn.execute(
        INSERT_SQL,
        (&uuid_str, &resource.name, &resource.ty, &resource.data),
    )?;
    Ok(())
}

pub(crate) fn delete_resource(conn: &Connection, uuid: &Uuid) -> Result<(), SqliteStorageError> {
    const DELETE_SQL: &str = r#"
        DELETE FROM resources
        WHERE id == ?;
    "#;

    let uuid_str = format!("{}", uuid.as_hyphenated());
    conn.execute(DELETE_SQL, (&uuid_str,))?;

    Ok(())
}

fn create_resource_from_row(row: &Row) -> Result<Resource, SqliteStorageError> {
    let id_str: String = row.get("id")?;
    let id = id_str.parse()?;
    Ok(Resource {
        id,
        name: row.get("name")?,
        ty: row.get("ty")?,
        data: row.get("data")?,
    })
}

fn create_resources_from_row_no_data(row: &Row) -> Result<Resource, SqliteStorageError> {
    let id_str: String = row.get("id")?;
    let id = id_str.parse()?;
    Ok(Resource {
        id,
        name: row.get("name")?,
        ty: row.get("ty")?,
        data: row.get("data")?,
    })
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
            id: Uuid::new_v4(),
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        insert_resource(&conn, &res).unwrap();
    }

    #[test]
    fn test_insert_resource_name_conflict() {
        let conn = init_db_connection();

        let id = Uuid::new_v4();

        let res = Resource {
            id,
            name: String::from("res1"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        insert_resource(&conn, &res).unwrap();

        let res = Resource {
            id,
            name: String::from("res2"),
            ty: String::from("text/css"),
            data: vec![1, 2, 3, 4],
        };
        let insert_res = insert_resource(&conn, &res);
        assert!(insert_res.is_err());
    }

    #[test]
    fn test_select_basic() {
        let conn = init_db_connection();

        let res = Resource {
            id: Uuid::new_v4(),
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        insert_resource(&conn, &res).unwrap();

        let selected_res = get_resource(&conn, &res.id).unwrap().unwrap();
        assert_eq!(res.id, selected_res.id);
        assert_eq!(res.name, selected_res.name);
        assert_eq!(res.ty, selected_res.ty);
        assert_eq!(res.data, selected_res.data);
    }

    #[test]
    fn test_select_not_exist() {
        let conn = init_db_connection();

        let id = Uuid::new_v4();
        let selected_res = get_resource(&conn, &id).unwrap();
        assert!(selected_res.is_none());
    }

    #[test]
    fn test_delete_basic() {
        let conn = init_db_connection();

        let res = Resource {
            id: Uuid::new_v4(),
            name: String::from("res"),
            ty: String::from("text/html"),
            data: vec![0, 1, 2, 3],
        };
        insert_resource(&conn, &res).unwrap();

        delete_resource(&conn, &res.id).unwrap();

        let selected_res = get_resource(&conn, &res.id).unwrap();
        assert!(selected_res.is_none());
    }

    #[test]
    fn test_delete_not_exist() {
        let conn = init_db_connection();
        let id = Uuid::new_v4();
        delete_resource(&conn, &id).unwrap();
    }
}
