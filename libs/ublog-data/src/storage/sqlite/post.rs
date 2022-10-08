use rusqlite::{Connection, Row, ToSql};
use ublog_doc::DocumentNode;
use uuid::Uuid;

use crate::models::{Post, Resource};
use crate::storage::sqlite::{SqliteExt, SqliteStorageError};
use crate::storage::{PaginatedList, Pagination};

pub(crate) fn init_db_schema(conn: &Connection) -> Result<(), SqliteStorageError> {
    const INIT_SQL: &str = r#"
        CREATE TABLE IF NOT EXISTS posts (
            slug             TEXT NOT NULL PRIMARY KEY,
            title            TEXT NOT NULL,
            author           TEXT NOT NULL,
            create_timestamp INTEGER NOT NULL,
            update_timestamp INTEGER NOT NULL,
            category         TEXT NOT NULL,
            content          BLOB NOT NULL
        );

        CREATE INDEX IF NOT EXISTS posts_idx_ts       ON posts (create_timestamp DESC);
        CREATE INDEX IF NOT EXISTS posts_idx_category ON posts (category);

        CREATE TABLE IF NOT EXISTS posts_tags (
            post_slug TEXT NOT NULL REFERENCES posts(slug) ON DELETE CASCADE,
            tag_name  TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS        posts_tags_idx_tag_name ON posts_tags (tag_name);
        CREATE UNIQUE INDEX IF NOT EXISTS posts_tags_idx_uniq     ON posts_tags (post_slug, tag_name);

        CREATE TABLE IF NOT EXISTS posts_resources (
            post_slug TEXT NOT NULL REFERENCES posts(slug) ON DELETE CASCADE,
            res_id    TEXT NOT NULL REFERENCES resources(id)
        );

        CREATE UNIQUE INDEX IF NOT EXISTS posts_resources_idx_uniq ON posts_resources (post_slug, res_id);
    "#;

    conn.execute_batch(INIT_SQL)?;

    Ok(())
}

pub(super) fn get_post(
    conn: &Connection,
    post_slug: &str,
) -> Result<Option<Post>, SqliteStorageError> {
    const SELECT_SQL: &str = r#"
        SELECT title, slug, author, create_timestamp, update_timestamp, category, content
        FROM posts
        WHERE slug == ?;
    "#;

    let mut post = conn.query_one(SELECT_SQL, (post_slug,), create_post_from_row)?;
    if let Some(post) = post.as_mut() {
        populate_post_tags(conn, post)?;
    }

    Ok(post)
}

pub(super) fn get_post_with_resources(
    conn: &Connection,
    post_slug: &str,
) -> Result<Option<(Post, Vec<Resource>)>, SqliteStorageError> {
    let post = match get_post(conn, post_slug)? {
        Some(post) => post,
        None => {
            return Ok(None);
        }
    };
    let post_resources = crate::storage::sqlite::resource::get_post_resources(conn, post_slug)?;

    Ok(Some((post, post_resources)))
}

pub(super) fn get_posts(
    conn: &Connection,
    pagination: &Pagination,
) -> Result<PaginatedList<Post>, SqliteStorageError> {
    const SELECT_SQL: &str = r#"
        SELECT title, slug, author, create_timestamp, update_timestamp, category
        FROM posts
        ORDER BY create_timestamp DESC
        LIMIT ? OFFSET ?;
    "#;

    const SELECT_COUNT_SQL: &str = r#"
        SELECT count(*) AS cnt FROM posts;
    "#;

    let limit = pagination.page_size();
    let offset = pagination.skip_count();

    let total_count: usize = conn
        .query_one(SELECT_COUNT_SQL, (), |row| row.get(0).map_err(From::from))?
        .unwrap();

    let mut posts =
        conn.query_many(SELECT_SQL, (limit, offset), create_post_from_row_no_content)?;
    for p in &mut posts {
        populate_post_tags(conn, p)?;
    }

    Ok(PaginatedList {
        objects: posts,
        total_count,
    })
}

pub(super) fn insert_post(
    conn: &Connection,
    post: &Post,
    post_resources: &[Resource],
) -> Result<(), SqliteStorageError> {
    const INSERT_POST_SQL: &str = r#"
        INSERT INTO posts (title, slug, author, create_timestamp, update_timestamp, category, content)
        VALUES (?, ?, ?, ?, ?, ?, ?);
    "#;

    let content_data = bson::to_vec(&post.content).unwrap();

    // Insert the post object into the database.
    conn.execute(
        INSERT_POST_SQL,
        (
            &post.title,
            &post.slug,
            &post.author,
            post.create_timestamp,
            post.update_timestamp,
            &post.category,
            &content_data,
        ),
    )?;

    // Insert tags into the database.
    if !post.tags.is_empty() {
        crate::storage::sqlite::post::insert_post_tags(conn, &post.slug, &post.tags)?;
    }

    // Insert post resources into the database.
    crate::storage::sqlite::post::insert_post_resources(conn, &post.slug, post_resources)?;

    Ok(())
}

pub(super) fn delete_post(conn: &Connection, post_slug: &str) -> Result<(), SqliteStorageError> {
    const DELETE_SQL: &str = r#"
        DELETE FROM posts
        WHERE slug == ?;
    "#;

    delete_post_resources(conn, post_slug)?;

    conn.execute(DELETE_SQL, (post_slug,))?;

    Ok(())
}

fn populate_post_tags(conn: &Connection, post: &mut Post) -> Result<(), SqliteStorageError> {
    const SELECT_SQL: &str = r#"
        SELECT tag_name FROM posts_tags
        WHERE post_slug == ?;
    "#;

    let mut select_stmt = conn.prepare_cached(SELECT_SQL).unwrap();
    let rows = select_stmt.query((&post.slug,))?;

    post.tags = rows
        .mapped(|row| row.get(0))
        .collect::<Result<Vec<String>, rusqlite::Error>>()?;

    Ok(())
}

fn insert_post_tags(
    conn: &Connection,
    post_slug: &str,
    tags: &[String],
) -> Result<(), SqliteStorageError> {
    if tags.is_empty() {
        return Ok(());
    }

    let mut param_values: Vec<&dyn ToSql> = vec![&post_slug];
    param_values.reserve(tags.len());
    for t in tags {
        param_values.push(t);
    }

    let insert_tags_sql = format!(
        r#"
            INSERT INTO posts_tags (post_slug, tag_name)
            VALUES {};
        "#,
        vec!["(?1, ?)"; tags.len()].join(",")
    );
    conn.execute(&insert_tags_sql, param_values.as_slice())?;

    Ok(())
}

fn insert_post_resources(
    conn: &Connection,
    post_slug: &str,
    resources: &[Resource],
) -> Result<(), SqliteStorageError> {
    for res in resources {
        crate::storage::sqlite::resource::insert_resource(conn, res)?;

        const INSERT_RELATION_SQL: &str = r#"
            INSERT INTO posts_resources (post_slug, res_id)
            VALUES (?, ?);
        "#;
        let res_id = format!("{}", res.id.as_hyphenated());
        conn.execute(INSERT_RELATION_SQL, (post_slug, &res_id))?;
    }

    Ok(())
}

fn delete_post_resources(conn: &Connection, post_slug: &str) -> Result<(), SqliteStorageError> {
    const SELECT_RES_ID_SQL: &str = r#"
        SELECT res_id
        FROM posts_resources
        WHERE post_slug == ?;
    "#;
    let res_ids = conn.query_many(SELECT_RES_ID_SQL, (post_slug,), |row| {
        let res_id_str: String = row.get("res_id")?;
        let res_id = Uuid::try_parse(&res_id_str)?;
        Ok(res_id)
    })?;

    for res_id in res_ids {
        crate::storage::sqlite::resource::delete_resource(conn, &res_id)?;
    }

    Ok(())
}

fn create_post_from_row(row: &Row) -> Result<Post, SqliteStorageError> {
    let content_data: Vec<u8> = row.get("content")?;
    let content = bson::from_slice(&content_data)?;
    Ok(Post {
        title: row.get("title")?,
        slug: row.get("slug")?,
        author: row.get("author")?,
        create_timestamp: row.get("create_timestamp")?,
        update_timestamp: row.get("update_timestamp")?,
        category: row.get("category")?,
        tags: Vec::new(),
        content,
    })
}

fn create_post_from_row_no_content(row: &Row) -> Result<Post, SqliteStorageError> {
    Ok(Post {
        title: row.get("title")?,
        slug: row.get("slug")?,
        author: row.get("author")?,
        create_timestamp: row.get("create_timestamp")?,
        update_timestamp: row.get("update_timestamp")?,
        category: row.get("category")?,
        tags: Vec::new(),
        content: DocumentNode::new_empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn init_db_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        init_db_schema(&conn).unwrap();
        crate::storage::sqlite::resource::init_db_schema(&conn).unwrap();

        conn
    }

    fn select_tag_names(conn: &Connection, post_slug: &str) -> Vec<String> {
        const SELECT_SQL: &str = r#"
            SELECT tag_name FROM posts_tags
            WHERE post_slug == ?;
        "#;

        conn.query_many(SELECT_SQL, (post_slug,), |row| {
            row.get(0).map_err(From::from)
        })
        .unwrap()
    }

    #[test]
    fn test_insert_post_basic() {
        let conn = init_db_connection();

        let post = Post {
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: Vec::new(),
            content: DocumentNode::new_empty(),
        };
        insert_post(&conn, &post, &[]).unwrap();
    }

    #[test]
    fn test_insert_post_conflict_slug() {
        let conn = init_db_connection();

        let post = Post {
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: Vec::new(),
            content: DocumentNode::new_empty(),
        };
        insert_post(&conn, &post, &[]).unwrap();

        let insert_res = insert_post(&conn, &post, &[]);
        assert!(insert_res.is_err());
    }

    #[test]
    fn test_insert_post_tags() {
        let conn = init_db_connection();

        let post = Post {
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: vec![String::from("tag1"), String::from("tag2")],
            content: DocumentNode::new_empty(),
        };
        insert_post(&conn, &post, &[]).unwrap();

        let tags: HashSet<_> = select_tag_names(&conn, &post.slug).into_iter().collect();
        let expected_tags: HashSet<_> = vec![String::from("tag1"), String::from("tag2")]
            .into_iter()
            .collect();
        assert_eq!(tags, expected_tags);
    }

    #[test]
    fn test_select_one_post_basic() {
        let conn = init_db_connection();

        let post = Post {
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: vec![String::from("tag1"), String::from("tag2")],
            content: DocumentNode::new_empty(),
        };
        insert_post(&conn, &post, &[]).unwrap();

        let selected_post = get_post(&conn, "slug").unwrap().unwrap();
        assert_eq!(post.title, selected_post.title);
        assert_eq!(post.slug, selected_post.slug);
        assert_eq!(post.author, selected_post.author);
        assert_eq!(post.create_timestamp, selected_post.create_timestamp);
        assert_eq!(post.update_timestamp, selected_post.update_timestamp);
        assert_eq!(post.category, selected_post.category);
        assert_eq!(post.tags, selected_post.tags);
    }

    #[test]
    fn test_select_one_post_not_exist() {
        let conn = init_db_connection();

        let selected_post = get_post(&conn, "slug").unwrap();
        assert!(selected_post.is_none());
    }

    #[test]
    fn test_select_many_basic() {
        let conn = init_db_connection();

        let post1 = Post {
            title: String::from("title"),
            slug: String::from("slug1"),
            author: String::from("msr"),
            create_timestamp: 30,
            update_timestamp: 30,
            category: String::from("category"),
            tags: vec![String::from("tag1"), String::from("tag2")],
            content: DocumentNode::new_empty(),
        };
        insert_post(&conn, &post1, &[]).unwrap();

        let post2 = Post {
            slug: String::from("slug2"),
            create_timestamp: 20,
            update_timestamp: 20,
            ..post1.clone()
        };
        insert_post(&conn, &post2, &[]).unwrap();

        let post3 = Post {
            slug: String::from("slug3"),
            create_timestamp: 10,
            update_timestamp: 10,
            ..post1
        };
        insert_post(&conn, &post3, &[]).unwrap();

        let selected_posts = get_posts(&conn, &Pagination::from_page_and_size(2, 1)).unwrap();
        assert_eq!(selected_posts.objects.len(), 1);
        assert_eq!(selected_posts.total_count, 3);

        let selected_post = &selected_posts.objects[0];
        assert_eq!(post2.title, selected_post.title);
        assert_eq!(post2.slug, selected_post.slug);
        assert_eq!(post2.author, selected_post.author);
        assert_eq!(post2.create_timestamp, selected_post.create_timestamp);
        assert_eq!(post2.update_timestamp, selected_post.update_timestamp);
        assert_eq!(post2.category, selected_post.category);
        assert_eq!(post2.tags, selected_post.tags);
    }

    #[test]
    fn test_delete_basic() {
        let conn = init_db_connection();

        let post = Post {
            title: String::from("title"),
            slug: String::from("slug"),
            author: String::from("msr"),
            create_timestamp: 0,
            update_timestamp: 0,
            category: String::from("category"),
            tags: Vec::new(),
            content: DocumentNode::new_empty(),
        };
        insert_post(&conn, &post, &[]).unwrap();

        delete_post(&conn, "slug").unwrap();

        let selected_post = get_post(&conn, "slug").unwrap();
        assert!(selected_post.is_none());
    }

    #[test]
    fn test_delete_not_exist() {
        let conn = init_db_connection();
        delete_post(&conn, "slug").unwrap();
    }
}
