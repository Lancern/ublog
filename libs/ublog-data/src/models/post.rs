use std::borrow::Borrow;
use std::sync::RwLock;

use rusqlite::{Connection, Row, Rows, ToSql};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ublog_doc::DocumentNode;

use crate::db::Pagination;

use crate::models::Model;

/// A blog post.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Post {
    /// A globally unique ID that identifies the post.
    pub id: i64,

    /// The post's title.
    pub title: String,

    /// The post's slug.
    pub slug: String,

    /// The post's author.
    pub author: String,

    /// Unix timestamp of the post's creation time, in UTC time zone.
    pub create_timestamp: i64,

    /// Unix timestamp of the post's last update time, in UTC time zone.
    pub update_timestamp: i64,

    /// The post's category.
    pub category: String,

    /// The post's tags.
    pub tags: Vec<String>,

    /// Number of views of the post.
    pub views: u64,

    /// Content of the post.
    pub content: DocumentNode,
}

impl Post {
    /// Get the post's creation time.
    pub fn create_time(&self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.create_timestamp).unwrap()
    }

    /// Get the post's last update time.
    pub fn update_time(&self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.update_timestamp).unwrap()
    }

    fn serialize_content(&self) -> Vec<u8> {
        bincode::serialize(&self.content).unwrap()
    }

    fn deserialize_content(data: &[u8]) -> DocumentNode {
        bincode::deserialize(data).unwrap()
    }
}

impl Model for Post {
    const OBJECT_NAME: &'static str = "post";

    type SelectKey = str;

    fn init_db_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        const INIT_SQL: &str = r#"
            CREATE TABLE IF NOT EXISTS posts (
                id               INTEGER PRIMARY KEY,
                title            TEXT NOT NULL,
                slug             TEXT NOT NULL,
                author           TEXT NOT NULL,
                create_timestamp INTEGER NOT NULL,
                update_timestamp INTEGER NOT NULL,
                category         TEXT NOT NULL,
                views            INTEGER NOT NULL,
                content          BLOB NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS posts_idx_slug     ON posts (slug);
            CREATE INDEX IF NOT EXISTS        posts_idx_ts       ON posts (create_timestamp DESC);
            CREATE INDEX IF NOT EXISTS        posts_idx_category ON posts (category);
            CREATE INDEX IF NOT EXISTS        posts_idx_views    ON posts (views DESC);

            CREATE TABLE IF NOT EXISTS posts_tags (
                post_id  TEXT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
                tag_name TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS        posts_tags_idx_tag_name ON posts_tags (tag_name);
            CREATE UNIQUE INDEX IF NOT EXISTS posts_tags_idx_uniq     ON posts_tags (post_id, tag_name);
        "#;

        conn.execute_batch(INIT_SQL)
    }

    fn select_one_from<K>(conn: &RwLock<Connection>, key: &K) -> Result<Self, rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>,
    {
        const SELECT_SQL: &str = r#"
            SELECT id, title, slug, author, create_timestamp, update_timestamp, category, views, content
            FROM posts
            WHERE slug == ?;
        "#;

        let conn = conn.read().unwrap();

        let slug: &str = key.borrow();
        let mut post = conn.query_row(SELECT_SQL, (slug,), Self::from_row)?;

        select_tags_for_post(&*conn, &mut post)?;

        Ok(post)
    }

    fn select_many_from(
        conn: &RwLock<Connection>,
        pagination: &Pagination,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        const SELECT_SQL: &str = r#"
            SELECT id, title, slug, author, create_timestamp, update_timestamp, category, views
            FROM posts
            ORDER BY create_timestamp DESC
            LIMIT ? OFFSET ?;
        "#;

        let conn = conn.read().unwrap();

        let limit = pagination.page_size();
        let offset = pagination.skip_count();

        let mut query_stmt = conn.prepare_cached(SELECT_SQL).unwrap();
        let post_rows = query_stmt.query((limit, offset))?;
        let mut posts = Self::from_rows(post_rows)?;

        for p in &mut posts {
            select_tags_for_post(&*conn, p)?;
        }

        Ok(posts)
    }

    fn insert_into(&mut self, conn: &RwLock<Connection>) -> Result<(), rusqlite::Error> {
        const INSERT_POST_SQL: &str = r#"
            INSERT INTO posts (title, slug, author, create_timestamp, update_timestamp, category, views, content)
            VALUES (?, ?, ?, ?, ?, ?, 0, ?);
        "#;

        let mut conn = conn.write().unwrap();
        let trans = conn.transaction()?;

        let create_timestamp = now_utc_unix_timestamp();
        let update_timestamp = create_timestamp;

        let content_data = self.serialize_content();

        // Insert the post object into the database.
        trans.execute(
            INSERT_POST_SQL,
            (
                &self.title,
                &self.slug,
                &self.author,
                create_timestamp,
                update_timestamp,
                &self.category,
                &content_data,
            ),
        )?;
        self.id = trans.last_insert_rowid();
        self.create_timestamp = create_timestamp;
        self.update_timestamp = update_timestamp;
        self.views = 0;

        // Insert tags into the database.
        if !self.tags.is_empty() {
            insert_post_tags(&trans, self.id, &self.tags)?;
        }

        trans.commit()?;

        Ok(())
    }

    fn update_into(&mut self, conn: &RwLock<Connection>) -> Result<(), rusqlite::Error> {
        const QUERY_ID_SQL: &str = r#"
            SELECT id FROM posts WHERE slug == ?;
        "#;

        const UPDATE_POST_SQL: &str = r#"
            UPDATE posts
            SET
                title = ?,
                author = ?,
                create_timestamp = ?,
                update_timestamp = ?,
                category = ?,
                content = ?
            WHERE
                slug == ?;
        "#;

        let update_timestamp = now_utc_unix_timestamp();

        let mut conn = conn.write().unwrap();
        let trans = conn.transaction()?;

        // Query post ID.
        let post_id = trans.query_row(QUERY_ID_SQL, (&self.slug,), |row| row.get("id"))?;
        self.id = post_id;

        // Update the post object itself.
        let content_data = self.serialize_content();
        let rows_updated = trans.execute(
            UPDATE_POST_SQL,
            (
                &self.title,
                &self.author,
                &self.create_timestamp,
                &self.update_timestamp,
                &self.category,
                &content_data,
                &self.slug,
            ),
        )?;
        if rows_updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        // Update the post's tags, if any.
        // Delete all old tags.
        const DELETE_TAGS_SQL: &str = r#"
            DELETE FROM posts_tags
            WHERE post_id == ?;
        "#;
        trans.execute(DELETE_TAGS_SQL, (&post_id,))?;

        // Insert all new tags.
        insert_post_tags(&trans, post_id, &self.tags)?;

        trans.commit()?;

        self.update_timestamp = update_timestamp;
        Ok(())
    }

    fn delete_from<K>(conn: &RwLock<Connection>, key: &K) -> Result<(), rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>,
    {
        const DELETE_SQL: &str = r#"
            DELETE FROM posts
            WHERE slug == ?;
        "#;

        let conn = conn.read().unwrap();

        let slug: &str = key.borrow();
        conn.execute(DELETE_SQL, (slug,))?;

        Ok(())
    }

    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let content_data: Vec<u8> = row.get("content")?;
        let content = Self::deserialize_content(&content_data);
        Ok(Post {
            id: row.get("id")?,
            title: row.get("title")?,
            slug: row.get("slug")?,
            author: row.get("author")?,
            create_timestamp: row.get("create_timestamp")?,
            update_timestamp: row.get("update_timestamp")?,
            category: row.get("category")?,
            tags: Vec::new(),
            views: row.get("views")?,
            content,
        })
    }

    fn from_rows(rows: Rows) -> Result<Vec<Self>, rusqlite::Error> {
        rows.mapped(|row| {
            Ok(Post {
                id: row.get("id")?,
                title: row.get("title")?,
                slug: row.get("slug")?,
                author: row.get("author")?,
                create_timestamp: row.get("create_timestamp")?,
                update_timestamp: row.get("update_timestamp")?,
                category: row.get("category")?,
                tags: Vec::new(),
                views: row.get("views")?,
                content: DocumentNode::default(),
            })
        })
        .collect()
    }
}

/// A resource object that is attached to a post.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PostResource {
    /// ID of the associated post.
    pub post_id: i64,

    /// Name of the resource.
    pub name: String,

    /// MIME type of the resource.
    pub ty: String,

    /// Data of the resource.
    pub data: Vec<u8>,
}

impl Model for PostResource {
    const OBJECT_NAME: &'static str = "post_resource";

    type SelectKey = (i64, String);

    fn init_db_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        const INIT_SQL: &str = r#"
            CREATE TABLE IF NOT EXISTS posts_resources (
                post_id  INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
                res_name TEXT NOT NULL,
                res_type TEXT NOT NULL,
                res_data BLOB NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS posts_resources_idx_name_uniq ON posts_resources (post_id, res_name);
        "#;
        conn.execute_batch(INIT_SQL)
    }

    fn select_one_from<K>(conn: &RwLock<Connection>, key: &K) -> Result<Self, rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>,
    {
        const SELECT_SQL: &str = r#"
            SELECT post_id, res_name, res_type, res_data FROM posts_resources
            WHERE post_id == ? AND res_name == ?;
        "#;

        let (post_id, res_name) = key.borrow();

        let conn = conn.read().unwrap();
        conn.query_row(SELECT_SQL, (post_id, res_name), Self::from_row)
    }

    fn insert_into(&mut self, conn: &RwLock<Connection>) -> Result<(), rusqlite::Error> {
        const INSERT_SQL: &str = r#"
            INSERT INTO posts_resources (post_id, res_name, res_type, res_data)
            VALUES (?, ?, ?, ?);
        "#;

        let conn = conn.read().unwrap();
        conn.execute(INSERT_SQL, (self.post_id, &self.name, &self.ty, &self.data))?;
        Ok(())
    }

    fn delete_from<K>(conn: &RwLock<Connection>, key: &K) -> Result<(), rusqlite::Error>
    where
        K: ?Sized + Borrow<Self::SelectKey>,
    {
        const DELETE_SQL: &str = r#"
            DELETE FROM posts_resources
            WHERE post_id == ? AND res_name == ?;
        "#;

        let conn = conn.read().unwrap();

        let (post_id, res_name) = key.borrow();
        conn.execute(DELETE_SQL, (post_id, res_name))?;
        Ok(())
    }

    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            post_id: row.get("post_id")?,
            name: row.get("res_name")?,
            ty: row.get("res_ty")?,
            data: row.get("res_data")?,
        })
    }

    fn from_rows(mut rows: Rows) -> Result<Vec<Self>, rusqlite::Error> {
        let mut ret = Vec::new();

        while let Some(row) = rows.next()? {
            ret.push(Self {
                post_id: row.get("post_id")?,
                name: row.get("res_name")?,
                ty: row.get("res_ty")?,
                data: Vec::new(),
            });
        }

        Ok(ret)
    }
}

pub(crate) trait PostModelExt: Model {
    fn update_views_into<K>(
        conn: &RwLock<Connection>,
        key: &K,
        views: u64,
    ) -> Result<(), rusqlite::Error>
    where
        K: ?Sized + Borrow<<Self as Model>::SelectKey>;
}

impl PostModelExt for Post {
    fn update_views_into<K>(
        conn: &RwLock<Connection>,
        key: &K,
        views: u64,
    ) -> Result<(), rusqlite::Error>
    where
        K: ?Sized + Borrow<<Self as Model>::SelectKey>,
    {
        let slug: &str = key.borrow();

        const UPDATE_SQL: &str = r#"
            UPDATE posts
            SET views = ?
            WHERE slug == ?;
        "#;

        let conn = conn.read().unwrap();
        let updated_rows = conn.execute(UPDATE_SQL, (views, slug))?;
        if updated_rows == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }
}

fn now_utc_unix_timestamp() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

fn select_tags_for_post(conn: &Connection, post: &mut Post) -> Result<(), rusqlite::Error> {
    const SELECT_SQL: &str = r#"
        SELECT tag_name FROM posts_tags
        WHERE post_id == ?;
    "#;

    let mut select_stmt = conn.prepare_cached(SELECT_SQL).unwrap();
    let rows = select_stmt.query((post.id,))?;

    post.tags = rows
        .mapped(|row| row.get(0))
        .collect::<Result<Vec<String>, rusqlite::Error>>()?;

    Ok(())
}

fn insert_post_tags(
    conn: &Connection,
    post_id: i64,
    tags: &[String],
) -> Result<(), rusqlite::Error> {
    if tags.is_empty() {
        return Ok(());
    }

    let column_parameters: Vec<&'static str> = vec!["(?1, ?)"; tags.len()];

    let mut param_values: Vec<&dyn ToSql> = vec![&post_id];
    param_values.reserve(tags.len());
    for t in tags {
        param_values.push(t);
    }

    let insert_tags_sql = format!(
        r#"
            INSERT INTO posts_tags (post_id, tag_name)
            VALUES {};
        "#,
        column_parameters.join(",")
    );

    conn.execute(&insert_tags_sql, param_values.as_slice())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn init_db_connection() -> RwLock<Connection> {
        let conn = Connection::open_in_memory().unwrap();
        Post::init_db_schema(&conn).unwrap();
        RwLock::new(conn)
    }

    fn select_tag_names(conn: &RwLock<Connection>, post_id: i64) -> Vec<String> {
        const SELECT_SQL: &str = r#"
            SELECT tag_name FROM posts_tags
            WHERE post_id == ?;
        "#;

        let conn = conn.read().unwrap();
        let mut select_sql_stmt = conn.prepare(SELECT_SQL).unwrap();
        select_sql_stmt
            .query((post_id,))
            .unwrap()
            .mapped(|row| row.get(0))
            .collect::<Result<_, rusqlite::Error>>()
            .unwrap()
    }

    #[test]
    fn test_insert_post_basic() {
        let conn = init_db_connection();

        let mut post = Post {
            id: 42,
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: Vec::new(),
            views: 100,
            content: DocumentNode::default(),
        };
        post.insert_into(&conn).unwrap();

        assert_ne!(post.id, 42); // post.id should be updated to the ID of the post, which may not be 42.
        assert_ne!(post.create_timestamp, 0);
        assert_ne!(post.update_timestamp, 0);
    }

    #[test]
    fn test_insert_post_conflict_slug() {
        let conn = init_db_connection();

        let mut post = Post {
            id: 0,
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: Vec::new(),
            views: 100,
            content: DocumentNode::default(),
        };
        post.insert_into(&conn).unwrap();

        let insert_err = post.insert_into(&conn).unwrap_err();
        assert_eq!(
            insert_err.sqlite_error_code().unwrap(),
            rusqlite::ErrorCode::ConstraintViolation
        );
    }

    #[test]
    fn test_insert_post_tags() {
        let conn = init_db_connection();

        let mut post = Post {
            id: 0,
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: vec![String::from("tag1"), String::from("tag2")],
            views: 100,
            content: DocumentNode::default(),
        };
        post.insert_into(&conn).unwrap();

        let tags: HashSet<_> = select_tag_names(&conn, post.id).into_iter().collect();
        let expected_tags: HashSet<_> = vec![String::from("tag1"), String::from("tag2")]
            .into_iter()
            .collect();
        assert_eq!(tags, expected_tags);
    }

    #[test]
    fn test_select_one_post_basic() {
        let conn = init_db_connection();

        let mut post = Post {
            id: 0,
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: vec![String::from("tag1"), String::from("tag2")],
            views: 0,
            content: DocumentNode::default(),
        };
        post.insert_into(&conn).unwrap();

        let selected_post = Post::select_one_from(&conn, "slug").unwrap();
        assert_eq!(post.id, selected_post.id);
        assert_eq!(post.title, selected_post.title);
        assert_eq!(post.slug, selected_post.slug);
        assert_eq!(post.author, selected_post.author);
        assert_eq!(post.create_timestamp, selected_post.create_timestamp);
        assert_eq!(post.update_timestamp, selected_post.update_timestamp);
        assert_eq!(post.category, selected_post.category);
        assert_eq!(post.tags, selected_post.tags);
        assert_eq!(post.views, selected_post.views);
    }

    #[test]
    fn test_select_one_post_not_exist() {
        let conn = init_db_connection();

        let select_err = Post::select_one_from(&conn, "slug").unwrap_err();
        match select_err {
            rusqlite::Error::QueryReturnedNoRows => {}
            e => {
                panic!("Unexpected error returned: {}", e);
            }
        }
    }

    #[test]
    fn test_select_many_basic() {
        let conn = init_db_connection();

        let mut post1 = Post {
            id: 0,
            title: String::from("title"),
            slug: String::from("slug1"),
            author: String::from("msr"),
            create_timestamp: 30,
            update_timestamp: 30,
            category: String::from("category"),
            tags: vec![String::from("tag1"), String::from("tag2")],
            views: 0,
            content: DocumentNode::default(),
        };
        post1.insert_into(&conn).unwrap();

        let mut post2 = Post {
            slug: String::from("slug2"),
            create_timestamp: 20,
            update_timestamp: 20,
            ..post1.clone()
        };
        post2.insert_into(&conn).unwrap();

        let mut post3 = Post {
            slug: String::from("slug3"),
            create_timestamp: 10,
            update_timestamp: 10,
            ..post1.clone()
        };
        post3.insert_into(&conn).unwrap();

        let selected_posts =
            Post::select_many_from(&conn, &Pagination::from_page_and_size(2, 1)).unwrap();
        assert_eq!(selected_posts.len(), 1);

        let selected_post = &selected_posts[0];
        assert_eq!(post2.id, selected_post.id);
        assert_eq!(post2.title, selected_post.title);
        assert_eq!(post2.slug, selected_post.slug);
        assert_eq!(post2.author, selected_post.author);
        assert_eq!(post2.create_timestamp, selected_post.create_timestamp);
        assert_eq!(post2.update_timestamp, selected_post.update_timestamp);
        assert_eq!(post2.category, selected_post.category);
        assert_eq!(post2.tags, selected_post.tags);
        assert_eq!(post2.views, selected_post.views);
    }

    #[test]
    fn test_update_basic() {
        let conn = init_db_connection();

        let mut post = Post {
            id: 0,
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 30,
            update_timestamp: 30,
            category: String::from("category"),
            tags: vec![String::from("tag1"), String::from("tag2")],
            views: 0,
            content: DocumentNode::default(),
        };
        post.insert_into(&conn).unwrap();

        let mut update_post = Post {
            id: 0,
            title: String::from("title2"),
            slug: String::from("slug"),
            author: String::from("msr2"),
            create_timestamp: 30,
            update_timestamp: 30,
            category: String::from("category2"),
            tags: vec![String::from("tag1"), String::from("tag3")],
            views: 0,
            content: DocumentNode::default(),
        };
        update_post.update_into(&conn).unwrap();

        let selected_post = Post::select_one_from(&conn, "slug").unwrap();
        assert_eq!(selected_post.title, "title2");
        assert_eq!(selected_post.author, "msr2");
        assert_eq!(selected_post.category, "category2");

        let selected_post_tags: HashSet<_> = selected_post.tags.into_iter().collect();
        let expected_post_tags: HashSet<_> = vec![String::from("tag1"), String::from("tag3")]
            .into_iter()
            .collect();
        assert_eq!(selected_post_tags, expected_post_tags);
    }

    #[test]
    fn test_update_not_exist() {
        let conn = init_db_connection();

        let mut post = Post {
            id: 0,
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 30,
            update_timestamp: 30,
            category: String::from("category"),
            tags: Vec::new(),
            views: 0,
            content: DocumentNode::default(),
        };
        let update_err = post.update_into(&conn).unwrap_err();

        match update_err {
            rusqlite::Error::QueryReturnedNoRows => {}
            e => {
                panic!("Unexpected error returned: {}", e);
            }
        }
    }

    #[test]
    fn test_delete_basic() {
        let conn = init_db_connection();

        let mut post = Post {
            id: 0,
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: Vec::new(),
            views: 0,
            content: DocumentNode::default(),
        };
        post.insert_into(&conn).unwrap();

        Post::delete_from(&conn, "slug").unwrap();

        let select_err = Post::select_one_from(&conn, "slug").unwrap_err();
        match select_err {
            rusqlite::Error::QueryReturnedNoRows => {}
            e => {
                panic!("Unexpected error returned: {}", e);
            }
        }
    }

    #[test]
    fn test_delete_not_exist() {
        let conn = init_db_connection();
        Post::delete_from(&conn, "slug").unwrap();
    }

    #[test]
    fn test_update_views_count() {
        let conn = init_db_connection();

        let mut post = Post {
            id: 0,
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: Vec::new(),
            views: 0,
            content: DocumentNode::default(),
        };
        post.insert_into(&conn).unwrap();

        Post::update_views_into(&conn, "slug", 100).unwrap();

        let conn = conn.read().unwrap();

        const SELECT_VIEWS_SQL: &str = r#"
            SELECT views FROM posts
            WHERE slug == ?;
        "#;
        let updated_views: u64 = conn
            .query_row(SELECT_VIEWS_SQL, ("slug",), |row| row.get(0))
            .unwrap();

        assert_eq!(updated_views, 100);
    }
}
