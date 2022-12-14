use rusqlite::{Connection, Row, ToSql};

use crate::models::{Commit, CommitPayload};
use crate::storage::sqlite::{SqliteExt, SqliteStorageError};

pub(crate) fn init_db_schema(conn: &Connection) -> Result<(), SqliteStorageError> {
    const INIT_SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS commits (
            id             BLOB NOT NULL,
            timestamp      INTEGER NOT NULL,
            prev_commit_id BLOB NOT NULL,
            payload        BLOB NOT NULL
        );

        CREATE INDEX IF NOT EXISTS commits_idx_timestamp ON commits (timestamp ASC);
    "#;

    conn.execute(INIT_SQL, ())?;
    Ok(())
}

pub(crate) fn get_latest_commit(conn: &Connection) -> Result<Option<Commit>, SqliteStorageError> {
    const SELECT_SQL: &str = r#"
        SELECT id, timestamp, prev_commit_id, payload
        FROM commits
        ORDER BY timestamp DESC
        LIMIT 1;
    "#;

    conn.query_one(SELECT_SQL, (), create_commit_from_row)
}

pub(crate) fn get_commits(
    conn: &Connection,
    since_timestamp: i64,
) -> Result<Vec<Commit>, SqliteStorageError> {
    const SELECT_SQL: &str = r#"
        SELECT id, timestamp, prev_commit_id, payload
        FROM commits
        WHERE timestamp >= ?
        ORDER BY timestamp ASC;
    "#;

    conn.query_many(SELECT_SQL, (since_timestamp,), create_commit_from_row)
}

pub(crate) fn insert_commit(conn: &Connection, commit: &Commit) -> Result<(), SqliteStorageError> {
    insert_commits(conn, std::slice::from_ref(commit))
}

pub(crate) fn insert_commits(
    conn: &Connection,
    commits: &[Commit],
) -> Result<(), SqliteStorageError> {
    let mut serialized_payload_data = Vec::with_capacity(commits.len());
    for c in commits {
        let payload = serialize_commit_payload(&c.payload);
        serialized_payload_data.push(payload);
    }

    let mut commit_sql_params: Vec<&dyn ToSql> = Vec::with_capacity(commits.len() * 4);
    for (c, payload_data) in commits.iter().zip(serialized_payload_data.iter()) {
        commit_sql_params.push(&c.id);
        commit_sql_params.push(&c.timestamp);
        commit_sql_params.push(&c.prev_commit_id);
        commit_sql_params.push(payload_data);
    }

    let insert_sql = format!(
        r#"
            INSERT INTO commits (id, timestamp, prev_commit_id, payload)
            VALUES {};
        "#,
        vec!["(?, ?, ?, ?)"; commits.len()].join(","),
    );
    conn.execute(&insert_sql, commit_sql_params.as_slice())?;

    Ok(())
}

fn create_commit_from_row(row: &Row) -> Result<Commit, SqliteStorageError> {
    let payload_data: Vec<u8> = row.get("payload")?;
    let payload = deserialize_commit_payload(&payload_data);

    let commit = Commit {
        id: row.get("id")?,
        timestamp: row.get("timestamp")?,
        prev_commit_id: row.get("prev_commit_id")?,
        payload,
    };
    Ok(commit)
}

fn serialize_commit_payload(payload: &CommitPayload) -> Vec<u8> {
    bson::to_vec(payload).unwrap()
}

fn deserialize_commit_payload(data: &[u8]) -> CommitPayload {
    bson::from_slice(data).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::CreatePostCommitPayload;

    fn init_db_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_insert_commit_basic() {
        let conn = init_db_connection();

        let commit = Commit {
            id: Vec::new(),
            timestamp: 100,
            prev_commit_id: Vec::new(),
            payload: CommitPayload::CreatePost(CreatePostCommitPayload {
                slug: String::from("slug"),
            }),
        };
        insert_commit(&conn, &commit).unwrap();
    }

    #[test]
    fn test_select_commit_basic() {
        let conn = init_db_connection();

        let commit1 = Commit {
            id: vec![1, 2],
            timestamp: 100,
            prev_commit_id: Vec::new(),
            payload: CommitPayload::CreatePost(CreatePostCommitPayload {
                slug: String::from("slug"),
            }),
        };
        let commit2 = Commit {
            id: vec![3, 4],
            timestamp: 200,
            prev_commit_id: vec![1, 2],
            payload: CommitPayload::CreatePost(CreatePostCommitPayload {
                slug: String::from("slug"),
            }),
        };
        let commit3 = Commit {
            id: vec![5, 6],
            timestamp: 300,
            prev_commit_id: vec![3, 4],
            payload: CommitPayload::CreatePost(CreatePostCommitPayload {
                slug: String::from("slug"),
            }),
        };
        insert_commit(&conn, &commit1).unwrap();
        insert_commit(&conn, &commit2).unwrap();
        insert_commit(&conn, &commit3).unwrap();

        let commits = get_commits(&conn, 200).unwrap();
        assert_eq!(commits.len(), 2);

        assert_eq!(commits[0].id, vec![3, 4]);
        assert_eq!(commits[0].timestamp, 200);
        assert_eq!(commits[0].prev_commit_id, vec![1, 2]);

        assert_eq!(commits[1].id, vec![5, 6]);
        assert_eq!(commits[1].timestamp, 300);
        assert_eq!(commits[1].prev_commit_id, vec![3, 4]);
    }
}
