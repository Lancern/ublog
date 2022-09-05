use std::sync::RwLock;

use rusqlite::{Connection, Row};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::models::Model;

/// A commit object.
///
/// A commit object represents a unit of change to the blog content.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Commit {
    /// ID of the commit.
    ///
    /// A commit's ID is the SHA256 digest of the commit's metadata and data.
    pub id: Vec<u8>,

    /// The creation timestamp of the commit.
    pub timestamp: i64,

    /// The previous commit ID.
    pub prev_commit_id: Vec<u8>,

    /// The serialized data of the commit.
    pub data: Vec<u8>,
}

impl Commit {
    /// Create a new commit object that contains serialized form of the given payload.
    pub fn new<T, U>(payload: &T, prev_commit_id: U) -> Self
    where
        T: ?Sized + Serialize,
        U: Into<Vec<u8>>,
    {
        let payload_data = bincode::serialize(payload).unwrap();
        let prev_commit_id = prev_commit_id.into();
        let timestamp = OffsetDateTime::now_utc().unix_timestamp();

        let mut commit = Self {
            id: Vec::new(),
            timestamp,
            prev_commit_id,
            data: payload_data,
        };
        let commit_digest_data = bincode::serialize(&commit).unwrap();
        let commit_digest = {
            let mut hasher = Sha256::new();
            hasher.update(&commit_digest_data);
            hasher.finalize()
        };

        commit.id = Vec::from(commit_digest.as_slice());

        commit
    }

    /// Deserialize the commit's data and get its payload object.
    pub fn get<T>(&self) -> Result<T, bincode::Error>
    where
        T: DeserializeOwned,
    {
        bincode::deserialize(&self.data)
    }
}

impl Model for Commit {
    const OBJECT_NAME: &'static str = "commit";

    type SelectKey = ();
    type UpdateMask = ();

    fn init_db_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        const INIT_SQL: &str = r#"
            CREATE TABLE IF NOT EXISTS commits (
                id             BLOB NOT NULL,
                timestamp      INTEGER NOT NULL,
                prev_commit_id BLOB NOT NULL,
                data           BLOB NOT NULL
            );

            CREATE INDEX IF NOT EXISTS commits_idx_timestamp ON commits (timestamp ASC);
        "#;

        conn.execute(INIT_SQL, ())?;
        Ok(())
    }

    fn insert_into(&mut self, conn: &RwLock<Connection>) -> Result<(), rusqlite::Error> {
        const INSERT_SQL: &str = r#"
            INSERT INTO commits (id, timestamp, prev_commit_id, data)
            VALUES (?, ?, ?, ?);
        "#;

        let conn = conn.read().unwrap();

        conn.execute(
            INSERT_SQL,
            (&self.id, self.timestamp, &self.prev_commit_id, &self.data),
        )?;
        Ok(())
    }

    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let commit = Self {
            id: row.get("id")?,
            timestamp: row.get("timestamp")?,
            prev_commit_id: row.get("prev_commit_id")?,
            data: row.get("data")?,
        };
        Ok(commit)
    }
}

pub(crate) trait CommitModelExt: Model {
    fn select_many_from(
        conn: &RwLock<Connection>,
        starting_timestamp: i64,
    ) -> Result<Vec<Self>, rusqlite::Error>;
}

impl CommitModelExt for Commit {
    fn select_many_from(
        conn: &RwLock<Connection>,
        starting_timestamp: i64,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        const SELECT_SQL: &str = r#"
            SELECT id, timestamp, prev_commit_id, data FROM commits
            WHERE timestamp >= ?
            ORDER BY timestamp ASC;
        "#;

        let conn = conn.read().unwrap();
        let mut stmt = conn.prepare_cached(SELECT_SQL).unwrap();
        let commits = stmt
            .query_map((starting_timestamp,), Self::from_row)?
            .collect::<Result<_, _>>()?;

        Ok(commits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct MockPayload {
        data: i32,
    }

    fn init_db_connection() -> RwLock<Connection> {
        let conn = Connection::open_in_memory().unwrap();
        Commit::init_db_schema(&conn).unwrap();
        RwLock::new(conn)
    }

    #[test]
    fn test_new_commit_and_get() {
        let payload = MockPayload { data: 42 };
        let commit = Commit::new(&payload, []);

        let deserialized_payload = commit.get::<MockPayload>().unwrap();
        assert_eq!(payload, deserialized_payload);
    }

    #[test]
    fn test_insert_commit_basic() {
        let conn = init_db_connection();

        let mut commit = Commit {
            id: Vec::new(),
            timestamp: 100,
            prev_commit_id: Vec::new(),
            data: Vec::new(),
        };
        commit.insert_into(&conn).unwrap();
    }

    #[test]
    fn test_select_commit_basic() {
        let conn = init_db_connection();

        let mut commit1 = Commit {
            id: vec![1, 2],
            timestamp: 100,
            prev_commit_id: Vec::new(),
            data: vec![10, 20],
        };
        let mut commit2 = Commit {
            id: vec![3, 4],
            timestamp: 200,
            prev_commit_id: vec![1, 2],
            data: vec![30, 40],
        };
        let mut commit3 = Commit {
            id: vec![5, 6],
            timestamp: 300,
            prev_commit_id: vec![3, 4],
            data: vec![50, 60],
        };
        commit1.insert_into(&conn).unwrap();
        commit2.insert_into(&conn).unwrap();
        commit3.insert_into(&conn).unwrap();

        let commits = <Commit as CommitModelExt>::select_many_from(&conn, 200).unwrap();
        assert_eq!(commits.len(), 2);

        assert_eq!(commits[0].id, vec![3, 4]);
        assert_eq!(commits[0].timestamp, 200);
        assert_eq!(commits[0].prev_commit_id, vec![1, 2]);
        assert_eq!(commits[0].data, vec![30, 40]);

        assert_eq!(commits[1].id, vec![5, 6]);
        assert_eq!(commits[1].timestamp, 300);
        assert_eq!(commits[1].prev_commit_id, vec![3, 4]);
        assert_eq!(commits[1].data, vec![50, 60]);
    }
}
