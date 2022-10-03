use std::sync::Arc;

use axum::extract::{Path, Query};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use http::{HeaderMap, HeaderValue};
use hyper::StatusCode;
use serde::Deserialize;
use ublog_data::models::{Post, PostResource, Resource};
use ublog_data::storage::Pagination;

use crate::cli::server::ServerContext;

/// Create a router for the server.
pub(super) fn create_router(ctx: Arc<ServerContext>) -> Router {
    Router::new()
        .route("/api/posts", get(get_posts))
        .route("/api/posts/:slug", get(get_post))
        .route("/api/posts/:slug/resources/:name", get(get_post_resource))
        .route("/api/resources/:name", get(get_resource))
        .layer(Extension(ctx))
}

#[derive(Clone, Debug, Deserialize)]
struct PaginationParams {
    #[serde(default)]
    page: Option<usize>,
    #[serde(default)]
    items: Option<usize>,
}

const DEFAULT_PAGE: usize = 1;
const DEFAULT_ITEMS_PER_PAGE: usize = 20;

async fn get_posts(
    Extension(ctx): Extension<Arc<ServerContext>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<Vec<Post>>, StatusCode> {
    let page = pagination.page.unwrap_or(DEFAULT_PAGE);
    let items = pagination.items.unwrap_or(DEFAULT_ITEMS_PER_PAGE);
    let pagination = Pagination::from_page_and_size(page, items);

    ctx.db
        .get_posts(&pagination)
        .await
        .map(Json)
        .map_err(|err| {
            spdlog::error!(
                "Get posts list from database failed: {} (page {}, items {})",
                err,
                page,
                items
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn get_post(
    Extension(ctx): Extension<Arc<ServerContext>>,
    Path((slug,)): Path<(String,)>,
) -> Result<Json<Post>, StatusCode> {
    ctx.db
        .get_post(&slug)
        .await
        .map_err(|err| {
            spdlog::error!("Get post from database failed: {} (slug {})", err, slug);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .and_then(|post| post.ok_or(StatusCode::NOT_FOUND).map(Json))
}

async fn get_post_resource(
    Extension(ctx): Extension<Arc<ServerContext>>,
    Path((slug, name)): Path<(String, String)>,
) -> Result<Blob, StatusCode> {
    ctx.db
        .get_post_resource(&slug, &name)
        .await
        .map_err(|err| {
            spdlog::error!(
                "Get post resource from database failed: {} (slug {}, name {})",
                err,
                slug,
                name
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .and_then(|resource| resource.ok_or(StatusCode::NOT_FOUND).map(From::from))
}

async fn get_resource(
    Extension(ctx): Extension<Arc<ServerContext>>,
    Path((name,)): Path<(String,)>,
) -> Result<Blob, StatusCode> {
    ctx.db
        .get_resource(&name)
        .await
        .map_err(|err| {
            spdlog::error!("Get resource from database failed: {} (name {})", err, name);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .and_then(|resource| resource.ok_or(StatusCode::NOT_FOUND).map(From::from))
}

#[derive(Clone, Debug)]
struct Blob {
    content_type: String,
    data: Vec<u8>,
}

impl From<Resource> for Blob {
    fn from(res: Resource) -> Self {
        Self {
            content_type: res.ty,
            data: res.data,
        }
    }
}

impl From<PostResource> for Blob {
    fn from(res: PostResource) -> Self {
        Self {
            content_type: res.ty,
            data: res.data,
        }
    }
}

impl IntoResponse for Blob {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            HeaderValue::from_str(&self.content_type).unwrap(),
        );

        (headers, self.data).into_response()
    }
}