pub mod block_tree;
pub mod models;
mod requests;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;

use async_recursion::async_recursion;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use spdlog::Logger;

use crate::api::block_tree::RawBlockTree;
use crate::api::models::{Block, Database, Page};
use crate::api::requests::NotionRequestExecutor;

/// Result type of Notion APIs.
pub type NotionApiResult<T> = Result<T, NotionApiError>;

/// Provide access to the Notion public API.
pub struct NotionApi {
    logger: Logger,
    exec: NotionRequestExecutor,
}

impl NotionApi {
    const BASE_URL: &'static str = "https://api.notion.com";

    /// Create a new NotionApi with the given access token.
    pub fn new<T>(token: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            logger: crate::create_logger("NotionApi"),
            exec: NotionRequestExecutor::new(token),
        }
    }

    /// Get Notion database schema.
    pub async fn get_database<T>(&self, database_id: T) -> NotionApiResult<Database>
    where
        T: AsRef<str>,
    {
        let database_id = database_id.as_ref();
        spdlog::trace!(logger: self.logger, "get database: {}", database_id);

        let url = format!("{}/v1/databases/{}", Self::BASE_URL, database_id);
        let request = self.exec.build_notion_request(Method::GET, url).build()?;
        let db = self.exec.execute(request).await?.json().await?;
        Ok(db)
    }

    /// Query Notion database entries.
    pub async fn query_database<T>(
        &self,
        database_id: T,
        params: &QueryDatabaseParams,
    ) -> NotionApiResult<Vec<Page>>
    where
        T: AsRef<str>,
    {
        let database_id = database_id.as_ref();
        spdlog::trace!(logger: self.logger, "query database: {}", database_id);

        let url = format!("{}/v1/databases/{}/query", Self::BASE_URL, database_id);

        self.get_paginated_list(|pagination| {
            let params = params.clone();
            let url = url.clone();
            async move {
                let payload = QueryDatabasePayload { pagination, params };
                let request = self
                    .exec
                    .build_notion_request(Method::POST, url)
                    .json(&payload)
                    .build()?;
                let page = self.exec.execute(request).await?.json().await?;
                Ok(page)
            }
        })
        .await
    }

    /// Get the block with the given block ID.
    pub async fn get_block<T>(&self, block_id: T) -> NotionApiResult<Block>
    where
        T: AsRef<str>,
    {
        let block_id = block_id.as_ref();
        spdlog::trace!(logger: self.logger, "get block: {}", block_id);

        let url = format!("{}/v1/blocks/{}", Self::BASE_URL, block_id);
        let request = self.exec.build_notion_request(Method::GET, url).build()?;
        let block = self.exec.execute(request).await?.json().await?;
        Ok(block)
    }

    /// Get child blocks of the specified block.
    pub async fn get_block_children<T>(&self, block_id: T) -> NotionApiResult<Vec<Block>>
    where
        T: AsRef<str>,
    {
        let block_id = block_id.as_ref();
        spdlog::trace!(logger: self.logger, "get block children: {}", block_id);

        let url = format!("{}/v1/blocks/{}/children", Self::BASE_URL, block_id);

        self.get_paginated_list(|pagination| {
            let url = url.clone();
            async move {
                let mut request_builder = self
                    .exec
                    .build_notion_request(Method::GET, url)
                    .query(&[("page_size", &format!("{}", pagination.page_size))]);

                if let Some(start_cursor) = &pagination.start_cursor {
                    request_builder = request_builder.query(&[("start_cursor", start_cursor)]);
                }

                let request = request_builder.build()?;

                let page = self.exec.execute(request).await?.json().await?;
                Ok(page)
            }
        })
        .await
    }

    /// Get a raw block tree rooted at the specified block.
    pub async fn get_block_tree<T>(&self, root_block_id: T) -> NotionApiResult<RawBlockTree>
    where
        T: AsRef<str>,
    {
        let root_block_id = root_block_id.as_ref();
        spdlog::trace!(logger: self.logger, "get block tree: {}", root_block_id);

        let root_block = self.get_block(root_block_id).await?;
        self.get_block_tree_impl(root_block).await
    }

    /// Get a list of raw block trees that represents the whole contents of the specified page.
    pub async fn get_page_content<T>(&self, page_id: T) -> NotionApiResult<Vec<RawBlockTree>>
    where
        T: AsRef<str>,
    {
        let page_id = page_id.as_ref();
        spdlog::trace!(logger: self.logger, "get page content: {}", page_id);

        let direct_blocks = self.get_block_children(page_id).await?;
        let get_tree_futures = direct_blocks
            .into_iter()
            .map(|blk| self.get_block_tree_impl(blk));
        futures::future::join_all(get_tree_futures)
            .await
            .into_iter()
            .collect::<Result<_, _>>()
    }

    async fn get_paginated_list<T, S, F>(&self, mut fetch_page: S) -> NotionApiResult<Vec<T>>
    where
        S: FnMut(NotionPagination) -> F,
        F: Future<Output = NotionApiResult<NotionPaginatedListPage<T>>>,
    {
        const DEFAULT_PAGE_SIZE: u64 = 100;
        let mut pagination = NotionPagination {
            start_cursor: None,
            page_size: DEFAULT_PAGE_SIZE,
        };

        let mut results = Vec::new();
        let mut has_more = true;
        while has_more {
            let mut page = fetch_page(pagination).await?;
            results.append(&mut page.results);

            has_more = page.has_more;
            pagination = NotionPagination {
                start_cursor: page.next_cursor,
                page_size: DEFAULT_PAGE_SIZE,
            };
        }

        Ok(results)
    }

    #[async_recursion]
    async fn get_block_tree_impl(&self, root_block: Block) -> NotionApiResult<RawBlockTree> {
        let child_blocks = self.get_block_children(&root_block.id).await?;

        let mut get_child_tree_futures = Vec::with_capacity(child_blocks.len());
        for child_blk in child_blocks {
            get_child_tree_futures.push(self.get_block_tree_impl(child_blk));
        }

        let mut tree = RawBlockTree::new(root_block);
        tree.children = futures::future::join_all(get_child_tree_futures)
            .await
            .into_iter()
            .collect::<Result<_, _>>()?;

        Ok(tree)
    }
}

impl Debug for NotionApi {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotionApi")
            .field("exec", &self.exec)
            .finish()
    }
}

/// Error returned from Notion APIs.
#[derive(Debug)]
pub enum NotionApiError {
    /// Network errors.
    Network(reqwest::Error),

    /// Notion errors.
    Notion(NotionError),
}

impl Display for NotionApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(err) => write!(f, "network error: {}", err),
            Self::Notion(err) => write!(f, "notion error: {}", err),
        }
    }
}

impl Error for NotionApiError {}

impl From<reqwest::Error> for NotionApiError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err)
    }
}

impl From<NotionError> for NotionApiError {
    fn from(err: NotionError) -> Self {
        Self::Notion(err)
    }
}

/// Errors returned by Notion services.
#[derive(Clone, Debug)]
pub struct NotionError {
    /// Error code.
    pub code: String,

    /// Error message.
    pub message: String,
}

impl Display for NotionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// Parameters for querying Notion database entries.
#[derive(Clone, Debug, Default, Serialize)]
pub struct QueryDatabaseParams {
    /// An optional query filter.
    pub filter: Option<QueryDatabaseFilter>,

    /// Specify how the entries should be sorted before pagination.
    pub sorts: Vec<QueryDatabaseSort>,
}

/// Query filter on Notion database entries.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum QueryDatabaseFilter {
    /// Filter on a specific database property.
    Property(QueryDatabasePropertyFilter),

    /// Compound filter `or`.
    Or(Vec<QueryDatabaseFilter>),

    /// Compound filter `and`.
    And(Vec<QueryDatabaseFilter>),
}

/// Specify how database entries should be sorted.
#[derive(Clone, Debug, Serialize)]
pub struct QueryDatabaseSort {
    /// The target property on which the database entries should be sorted.
    pub property: String,

    direction: &'static str,
}

impl QueryDatabaseSort {
    const DIRECTION_ASCENDING: &'static str = "ascending";
    const DIRECTION_DESCENDING: &'static str = "descending";

    /// Create a new `QueryDatabaseSort` object that specifies an ascending order on the specified database property.
    pub fn ascending_on<P>(property: P) -> Self
    where
        P: Into<String>,
    {
        Self {
            property: property.into(),
            direction: Self::DIRECTION_ASCENDING,
        }
    }

    /// Create a new `QueryDatabaseSort` object that specifies a descending order on the specified database property.
    pub fn descending_on<P>(property: P) -> Self
    where
        P: Into<String>,
    {
        Self {
            property: property.into(),
            direction: Self::DIRECTION_DESCENDING,
        }
    }
}

/// Database entry filter that filters on the values of a specific database property.
#[derive(Clone, Debug, Serialize)]
pub struct QueryDatabasePropertyFilter {
    /// The target property.
    pub property: String,

    /// Actual filter definitions corresponding to different types of properties.
    #[serde(flatten)]
    pub variant: QueryDatabasePropertyFilterVariants,
}

impl QueryDatabasePropertyFilter {
    /// Create a new `QueryDatabasePropertyFilter` that selects database entries whose specified checkbox property is
    /// checked.
    pub fn checkbox_checked<T>(property_name: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            property: property_name.into(),
            variant: QueryDatabasePropertyFilterVariants::Checkbox(
                QueryDatabaseCheckboxFilter::Equals(true),
            ),
        }
    }
}

/// Provide actual filter definitions corresponding to different types of properties.
#[derive(Clone, Debug, Serialize)]
pub enum QueryDatabasePropertyFilterVariants {
    /// Filter on a checkbox property.
    #[serde(rename = "checkbox")]
    Checkbox(QueryDatabaseCheckboxFilter),
}

/// A database property filter that filters on a checkbox property.
#[derive(Clone, Debug, Serialize)]
pub enum QueryDatabaseCheckboxFilter {
    #[serde(rename = "equals")]
    Equals(bool),
    #[serde(rename = "does_not_equal")]
    DoesNotEqual(bool),
}

#[derive(Clone, Debug, Serialize)]
struct NotionPagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    start_cursor: Option<String>,
    page_size: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct NotionPaginatedListPage<T> {
    has_more: bool,
    #[serde(default)]
    next_cursor: Option<String>,
    results: Vec<T>,
}

#[derive(Clone, Debug, Serialize)]
struct QueryDatabasePayload {
    #[serde(flatten)]
    pagination: NotionPagination,
    #[serde(flatten)]
    params: QueryDatabaseParams,
}
