use std::error::Error;

use ublog_data::db::Database;
use ublog_data::models::Resource;
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

    let diff_posts = filter_diff_posts(posts, &db).await?;
    let new_posts = diff_posts.iter().filter(|p| p.is_new()).count();
    let updated_posts = diff_posts.iter().filter(|p| p.is_updated()).count();
    spdlog::info!(
        "{} diff posts found: {} new, {} updated",
        diff_posts.len(),
        new_posts,
        updated_posts
    );

    futures::future::join_all(
        diff_posts
            .into_iter()
            .map(|post| apply_diff_post(post, &notion_api, &db)),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

async fn filter_diff_posts<S>(
    posts: Vec<NotionPost>,
    db: &Database<S>,
) -> Result<Vec<DiffPost>, Box<dyn Error>>
where
    S: Storage,
{
    let task = futures::future::join_all(posts.into_iter().map(|p| async {
        match db.get_post(&p.post.slug).await {
            Ok(Some(post)) => {
                if p.post.update_timestamp > post.update_timestamp {
                    Ok(Some(DiffPost::Updated(p)))
                } else {
                    Ok(None)
                }
            }
            Ok(None) => Ok(Some(DiffPost::New(p))), // db.get_post returns None indicating the post does not exist
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

#[derive(Debug)]
enum DiffPost {
    New(NotionPost),
    Updated(NotionPost),
}

impl DiffPost {
    fn is_new(&self) -> bool {
        matches!(self, Self::New(_))
    }

    fn is_updated(&self) -> bool {
        matches!(self, Self::Updated(_))
    }

    fn post(&self) -> &NotionPost {
        match self {
            Self::New(p) => p,
            Self::Updated(p) => p,
        }
    }

    fn post_mut(&mut self) -> &mut NotionPost {
        match self {
            Self::New(p) => p,
            Self::Updated(p) => p,
        }
    }
}

async fn apply_diff_post<S>(
    mut post: DiffPost,
    api: &NotionApi,
    db: &Database<S>,
) -> Result<(), Box<dyn Error>>
where
    S: Storage,
{
    fallible_step!(
        format!("fetch content of post {}", post.post().post.slug),
        ublog_notion::blog::get_post_content(api, post.post_mut()).await
    );

    let resources = fallible_step!(
        format!("extract resources in post {}", post.post().post.slug),
        ublog_notion::blog::extract_notion_resources(post.post_mut()).await
    );

    match &post {
        DiffPost::New(p) => insert_post(p, &resources, db).await,
        DiffPost::Updated(p) => update_post(p, &resources, db).await,
    }
}

async fn update_post<S>(
    post: &NotionPost,
    resources: &[Resource],
    db: &Database<S>,
) -> Result<(), Box<dyn Error>>
where
    S: Storage,
{
    fallible_step!(
        format!("update post {}", post.post.slug),
        db.update_post(&post.post, resources).await
    );

    spdlog::info!("Updated post: {} - {}", post.post.slug, post.notion_page_id);
    for r in resources {
        spdlog::info!(
            "Updated post resource: {}/{} - {}",
            post.post.slug,
            r.name,
            r.ty
        );
    }

    Ok(())
}

async fn insert_post<S>(
    post: &NotionPost,
    resources: &[Resource],
    db: &Database<S>,
) -> Result<(), Box<dyn Error>>
where
    S: Storage,
{
    fallible_step!(
        format!("insert post {}", post.post.slug),
        db.insert_post(&post.post, resources).await
    );

    spdlog::info!("New post: {} - {}", post.post.slug, post.notion_page_id);
    for r in resources {
        spdlog::info!(
            "New post resource: {}/{} - {}",
            post.post.slug,
            r.name,
            r.ty
        );
    }

    Ok(())
}
