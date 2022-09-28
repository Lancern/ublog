use rusqlite::{Connection, Row, Rows, ToSql};
use ublog_doc::DocumentNode;

use crate::models::{Post, PostResource};
use crate::storage::Pagination;

pub(crate) fn init_db_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
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
            res_name  TEXT NOT NULL,
            res_type  TEXT NOT NULL,
            res_data  BLOB NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS posts_resources_idx_uniq ON posts_resources (post_slug, res_name);
    "#;

    conn.execute_batch(INIT_SQL)
}

pub(super) fn get_post(
    conn: &Connection,
    post_slug: &str,
) -> Result<Option<Post>, rusqlite::Error> {
    const SELECT_SQL: &str = r#"
        SELECT title, slug, author, create_timestamp, update_timestamp, category, content
        FROM posts
        WHERE slug == ?;
    "#;

    let mut post = match conn.query_row(SELECT_SQL, (post_slug,), create_post_from_row) {
        Ok(p) => Some(p),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(err) => {
            return Err(err);
        }
    };

    if let Some(post) = post.as_mut() {
        populate_post_tags(&*conn, post)?;
    }

    Ok(post)
}

pub(super) fn get_post_with_resources(
    conn: &Connection,
    post_slug: &str,
) -> Result<Option<(Post, Vec<PostResource>)>, rusqlite::Error> {
    let post = match get_post(conn, post_slug)? {
        Some(post) => post,
        None => {
            return Ok(None);
        }
    };

    const SELECT_RESOURCES_SQL: &str = r#"
        SELECT post_slug, res_name, res_type, res_data
        FROM posts_resources
        where post_slug == ?;
    "#;

    let mut stmt = conn.prepare(SELECT_RESOURCES_SQL).unwrap();
    let post_resources = stmt
        .query_map((post_slug,), create_post_resource_from_row)?
        .collect::<Result<_, _>>()?;

    Ok(Some((post, post_resources)))
}

pub(super) fn get_posts(
    conn: &Connection,
    pagination: &Pagination,
) -> Result<Vec<Post>, rusqlite::Error> {
    const SELECT_SQL: &str = r#"
        SELECT title, slug, author, create_timestamp, update_timestamp, category
        FROM posts
        ORDER BY create_timestamp DESC
        LIMIT ? OFFSET ?;
    "#;

    let limit = pagination.page_size();
    let offset = pagination.skip_count();

    let mut query_stmt = conn.prepare_cached(SELECT_SQL).unwrap();
    let post_rows = query_stmt.query((limit, offset))?;
    let mut posts = create_posts_from_rows(post_rows)?;

    for p in &mut posts {
        populate_post_tags(&*conn, p)?;
    }

    Ok(posts)
}

pub(super) fn get_post_resource(
    conn: &Connection,
    post_slug: &str,
    resource_name: &str,
) -> Result<Option<PostResource>, rusqlite::Error> {
    const SELECT_SQL: &str = r#"
        SELECT post_slug, res_name, res_type, res_data
        FROM post_resources
        WHERE post_slug == ? AND res_name == ?;
    "#;

    conn.query_row(
        SELECT_SQL,
        (post_slug, resource_name),
        create_post_resource_from_row,
    )
    .map(Some)
    .or_else(|err| {
        if let rusqlite::Error::QueryReturnedNoRows = err {
            Ok(None)
        } else {
            Err(err)
        }
    })
}

pub(super) fn insert_post(
    conn: &Connection,
    post: &Post,
    post_resources: &[PostResource],
) -> Result<(), rusqlite::Error> {
    const INSERT_POST_SQL: &str = r#"
        INSERT INTO posts (title, slug, author, create_timestamp, update_timestamp, category, content)
        VALUES (?, ?, ?, ?, ?, ?, ?);
    "#;

    let content_data = serialize_post_content(&post.content);

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

pub(super) fn delete_post(conn: &Connection, post_slug: &str) -> Result<(), rusqlite::Error> {
    const DELETE_SQL: &str = r#"
        DELETE FROM posts
        WHERE slug == ?;
    "#;

    conn.execute(DELETE_SQL, (post_slug,))?;

    Ok(())
}

fn populate_post_tags(conn: &Connection, post: &mut Post) -> Result<(), rusqlite::Error> {
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
) -> Result<(), rusqlite::Error> {
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
    resources: &[PostResource],
) -> Result<(), rusqlite::Error> {
    if resources.is_empty() {
        return Ok(());
    }

    let mut param_values: Vec<&dyn ToSql> = vec![&post_slug];
    param_values.reserve(resources.len());
    for r in resources {
        param_values.push(&r.name);
        param_values.push(&r.ty);
        param_values.push(&r.data);
    }

    let insert_resources_sql = format!(
        r#"
            INSERT INTO posts_resources (post_slug, res_name, res_type, res_data)
            VALUES {};
        "#,
        vec!["(?1, ?, ?, ?)"; resources.len()].join(",")
    );
    conn.execute(&insert_resources_sql, param_values.as_slice())?;

    Ok(())
}

fn serialize_post_content(content: &DocumentNode) -> Vec<u8> {
    bson::to_vec(content).unwrap()
}

fn deserialize_post_content(data: &[u8]) -> DocumentNode {
    bson::from_slice(data).unwrap()
}

fn create_post_from_row(row: &Row) -> Result<Post, rusqlite::Error> {
    let content_data: Vec<u8> = row.get("content")?;
    let content = deserialize_post_content(&content_data);
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

fn create_posts_from_rows(rows: Rows) -> Result<Vec<Post>, rusqlite::Error> {
    rows.mapped(|row| {
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
    })
    .collect()
}

fn create_post_resource_from_row(row: &Row) -> Result<PostResource, rusqlite::Error> {
    Ok(PostResource {
        post_slug: row.get("post_slug")?,
        name: row.get("res_name")?,
        ty: row.get("res_types")?,
        data: row.get("res_data")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn init_db_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db_schema(&conn).unwrap();
        conn
    }

    fn select_tag_names(conn: &Connection, post_slug: &str) -> Vec<String> {
        const SELECT_SQL: &str = r#"
            SELECT tag_name FROM posts_tags
            WHERE post_slug == ?;
        "#;

        let mut select_sql_stmt = conn.prepare(SELECT_SQL).unwrap();
        select_sql_stmt
            .query((post_slug,))
            .unwrap()
            .mapped(|row| row.get(0))
            .collect::<Result<_, rusqlite::Error>>()
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

        let insert_err = insert_post(&conn, &post, &[]).unwrap_err();
        assert_eq!(
            insert_err.sqlite_error_code().unwrap(),
            rusqlite::ErrorCode::ConstraintViolation
        );
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
        assert_eq!(selected_posts.len(), 1);

        let selected_post = &selected_posts[0];
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
