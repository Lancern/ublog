#[cfg(feature = "remote-storage")]
pub mod remote;
pub mod sqlite;
pub mod sync;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Commit, Delta, Post, Resource};

/// Provide storage for databases.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Error type used by this storage type.
    type Error: std::error::Error;

    async fn insert_post(
        &self,
        post: &Post,
        post_resources: &[Resource],
    ) -> Result<(), Self::Error>;
    async fn update_post(
        &self,
        post: &Post,
        post_resources: &[Resource],
    ) -> Result<(), Self::Error>;
    async fn delete_post(&self, post_slug: &str) -> Result<(), Self::Error>;
    async fn get_post(&self, post_slug: &str) -> Result<Option<Post>, Self::Error>;
    async fn get_post_with_resources(
        &self,
        post_slug: &str,
    ) -> Result<Option<(Post, Vec<Resource>)>, Self::Error>;
    async fn get_posts(&self, pagination: &Pagination) -> Result<PaginatedList<Post>, Self::Error>;

    async fn insert_resource(&self, resource: &Resource) -> Result<(), Self::Error>;
    async fn delete_resource(&self, resource_id: &Uuid) -> Result<(), Self::Error>;
    async fn get_resource(&self, resource_id: &Uuid) -> Result<Option<Resource>, Self::Error>;
    async fn get_resources(&self) -> Result<Vec<Resource>, Self::Error>;

    async fn get_commits_since(&self, since_timestamp: i64) -> Result<Vec<Commit>, Self::Error>;
    async fn get_latest_commit(&self) -> Result<Option<Commit>, Self::Error>;

    async fn apply_delta(&self, delta: &Delta) -> Result<(), Self::Error>;
}

/// Pagination parameters.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Pagination {
    page: usize,
    page_size: usize,
}

impl Pagination {
    /// Create a new [`Pagination`] object.
    ///
    /// `page` gives the page number starting from 1. `page_size` gives the number of items displayed on each page.
    pub fn from_page_and_size(page: usize, page_size: usize) -> Self {
        assert!(page > 0);
        assert!(page_size > 0);
        assert!((page - 1).checked_mul(page_size).is_some());

        Self { page, page_size }
    }

    /// Get the page number. Page numbers start from 1.
    pub fn page(&self) -> usize {
        self.page
    }

    /// Get the number of items on each page.
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// Get the number of items before the first element of the specified page.
    pub fn skip_count(&self) -> usize {
        (self.page - 1) * self.page_size
    }
}

/// A paginated list.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaginatedList<T> {
    /// Objects that are listed in the requested page.
    pub objects: Vec<T>,

    /// The total number of objects regardless of the pagination.
    #[serde(rename = "totalCount")]
    pub total_count: usize,
}
