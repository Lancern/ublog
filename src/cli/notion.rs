use std::error::Error;

use ublog_data::db::Database;
use ublog_data::storage::sqlite::SqliteStorage;
use ublog_data::storage::Storage;
use ublog_notion::api::NotionApi;
use ublog_notion::blog::NotionPost;

use crate::{fallible_step, FetchNotionArgs};

pub(crate) async fn fetch_notion(args: &FetchNotionArgs) -> Result<(), Box<dyn Error>> {
    let notion_api = NotionApi::new(&args.token);

    let db_storage = fallible_step!(
        "initialize database storage",
        SqliteStorage::new_file(&args.database)
    );
    let db = Database::new(db_storage);

    let posts = fallible_step!(
        "fetch posts list",
        ublog_notion::blog::get_posts(&notion_api, &args.notion_database_id).await
    );
    spdlog::info!(
        "{} posts listed in the target Notion database.",
        posts.len()
    );

    let new_posts = filter_new_posts(posts, &db).await?;
    spdlog::info!("{} new posts found.", new_posts.len());

    futures::future::join_all(new_posts.into_iter().map(|post| insert_post(post, &db)))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

async fn filter_new_posts<S>(
    posts: Vec<NotionPost>,
    db: &Database<S>,
) -> Result<Vec<NotionPost>, Box<dyn Error>>
where
    S: Storage,
{
    let task = futures::future::join_all(posts.into_iter().map(|p| async {
        match db.get_post(&p.post.slug).await {
            Ok(Some(_)) => Ok(None), // db.get_post returns Some indicating the post already exists
            Ok(None) => Ok(Some(p)), // db.get_post returns None indicating the post does not exist
            Err(err) => Err(err),
        }
    }));

    let filtered_posts = task
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| Box::<dyn Error>::from(format!("{}", err)))?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    Ok(filtered_posts)
}

async fn insert_post<S>(mut post: NotionPost, db: &Database<S>) -> Result<(), Box<dyn Error>>
where
    S: Storage,
{
    let resources = fallible_step!(
        format!("extract resources in post {}", post.post.slug),
        ublog_notion::blog::extract_notion_resources(&mut post).await
    );

    fallible_step!(
        format!("insert post {}", post.post.slug),
        db.insert_post(&post.post, &resources).await
    );

    spdlog::info!("New post: {} - {}", post.post.slug, post.notion_page_id);
    for r in &resources {
        spdlog::info!(
            "New post resource: {}/{} - {}",
            post.post.slug,
            r.name,
            r.ty
        );
    }

    Ok(())
}
