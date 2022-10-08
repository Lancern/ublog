use std::sync::Arc;

use axum::extract::{Path, Query};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use http::{HeaderMap, HeaderValue};
use hyper::StatusCode;
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use ublog_data::models::{Post, Resource};
use ublog_data::storage::{PaginatedList, Pagination};
use uuid::Uuid;

use crate::cli::server::ServerContext;

/// Create a router for the server.
pub(super) fn create_router(ctx: Arc<ServerContext>) -> Router {
    Router::new()
        .route("/api/posts", get(get_posts))
        .route("/api/posts/:slug", get(get_post))
        .route("/api/resources/:id", get(get_resource))
        .layer(CorsLayer::new().allow_methods(Any).allow_origin(Any))
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
) -> Result<Json<PaginatedList<Post>>, StatusCode> {
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

async fn get_resource(
    Extension(ctx): Extension<Arc<ServerContext>>,
    Path((id,)): Path<(String,)>,
) -> Result<Blob, StatusCode> {
    let id = Uuid::try_parse(&id).map_err(|_| {
        spdlog::warn!("Invalid resource ID from client: {}", id);
        StatusCode::BAD_REQUEST
    })?;

    ctx.db
        .get_resource(&id)
        .await
        .map_err(|err| {
            spdlog::error!("Get resource from database failed: {} (id {})", err, id);
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
