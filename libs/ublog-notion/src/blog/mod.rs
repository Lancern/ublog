pub mod schema;

use std::error::Error;
use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;
use spdlog::Logger;
use ublog_data::models::{Post, PostResource};
use ublog_doc::{DocumentNode, DocumentNodeTag, DocumentNodeVisitor, DocumentResourceLink};
use uuid::Uuid;

use crate::api::{NotionApi, NotionApiError};

lazy_static! {
    static ref LOGGER: Logger = crate::create_logger("NotionBlog");
}

/// Get a list of posts from the specified Notion database.
///
/// The content of the returned posts are not fetched. To fetch the content of an individual post, use the
/// [`get_post_content`] function.
pub async fn get_posts<T>(
    api: &NotionApi,
    posts_db_id: T,
) -> Result<Vec<NotionPost>, NotionBlogError>
where
    T: AsRef<str>,
{
    let posts_db_id = posts_db_id.as_ref();
    spdlog::trace!(logger: LOGGER, "get posts: {}", posts_db_id);

    crate::blog::schema::validate_posts_db_schema(api, posts_db_id).await?;
    spdlog::debug!(
        logger: LOGGER,
        "successfully validated Notion blog database schema for database {}",
        posts_db_id
    );

    let query_posts_params = crate::blog::schema::get_query_posts_db_params();
    let posts_pages = api.query_database(posts_db_id, &query_posts_params).await?;

    posts_pages
        .iter()
        .map(crate::blog::schema::create_post_from_page)
        .collect::<Result<_, _>>()
        .map_err(NotionBlogError::from)
}

/// Get the content of the specified post from the corresponding Notion page.
pub async fn get_post_content(
    api: &NotionApi,
    post: &mut NotionPost,
) -> Result<(), NotionBlogError> {
    spdlog::trace!(
        logger: LOGGER,
        "get post content: {} - {}",
        post.post.slug,
        post.notion_page_id
    );

    let raw_content_trees = api.get_page_content(&post.notion_page_id).await?;
    let content_tree = crate::api::block_tree::normalize(raw_content_trees);

    post.post.content = crate::render::block::render_block_tree(&content_tree);

    Ok(())
}

/// Extract all referenced resources by the specified Notion post.
///
/// This function also updates the corresponding documentation node to refer to the extracted resources.
pub async fn extract_notion_resources(
    post: &mut NotionPost,
) -> Result<Vec<PostResource>, NotionBlogError> {
    spdlog::trace!(
        logger: LOGGER,
        "extract notion resources: {} - {}",
        post.post.slug,
        post.notion_page_id
    );

    #[derive(Default)]
    struct Visitor {
        resources: Vec<NotionResource>,
    }

    impl DocumentNodeVisitor for Visitor {
        fn visit_mut(&mut self, node: &mut DocumentNode) {
            if let Some((link, url)) = extract_notion_res_link_in_doc_node(node) {
                if let Some(resource) = NotionResource::new(url) {
                    let name = resource.name.clone();
                    self.resources.push(resource);
                    *link = DocumentResourceLink::Embedded { name };
                }
            }
        }
    }

    let mut visitor = Visitor::default();
    post.post.content.visit_mut(&mut visitor);

    let resources = futures::future::join_all(
        visitor
            .resources
            .into_iter()
            .map(|res| fetch_notion_resource(&post.post.slug, res)),
    )
    .await
    .into_iter()
    .collect::<Result<_, _>>()?;

    Ok(resources)
}

/// A post published via Notion.
#[derive(Clone, Debug)]
pub struct NotionPost {
    /// The Notion page ID corresponding to the post.
    pub notion_page_id: String,

    /// The post data model.
    pub post: Post,
}

/// Error type used in the notion blog module.
#[derive(Debug)]
pub enum NotionBlogError {
    /// Error originating from the Notion API.
    NotionApi(NotionApiError),

    /// Notion database schema validation errors.
    InvalidSchema(InvalidSchemaError),

    /// Error originating from the HTTP client.
    Http(reqwest::Error),
}

impl Display for NotionBlogError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotionApi(err) => write!(f, "Notion API error: {}", err),
            Self::InvalidSchema(err) => write!(f, "schema validation failed: {}", err),
            Self::Http(err) => write!(f, "HTTP error: {}", err),
        }
    }
}

impl Error for NotionBlogError {}

impl From<NotionApiError> for NotionBlogError {
    fn from(err: NotionApiError) -> Self {
        Self::NotionApi(err)
    }
}

impl From<InvalidSchemaError> for NotionBlogError {
    fn from(err: InvalidSchemaError) -> Self {
        Self::InvalidSchema(err)
    }
}

impl From<reqwest::Error> for NotionBlogError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err)
    }
}

/// Represent Notion database schema validation errors.
#[derive(Debug)]
pub enum InvalidSchemaError {
    /// A required property is missing from the database.
    MissingProperty(InvalidSchemaMissingPropertyError),

    /// A property has unexpected type.
    InvalidPropertyType(InvalidSchemaPropertyTypeError),
}

impl InvalidSchemaError {
    /// Create an `InvalidSchemaError` that represents the specified property is missing from the Notion database.
    pub fn missing_property<T>(prop: T) -> Self
    where
        T: Into<String>,
    {
        Self::MissingProperty(InvalidSchemaMissingPropertyError { prop: prop.into() })
    }

    /// Create an `InvalidSchemaError` that represents the specified property has unexpected type.
    pub fn invalid_property_type<T1, T2, T3>(prop: T1, expected_type: T2, actual_type: T3) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
        T3: Into<String>,
    {
        Self::InvalidPropertyType(InvalidSchemaPropertyTypeError {
            prop: prop.into(),
            expected_type: expected_type.into(),
            actual_type: actual_type.into(),
        })
    }
}

impl Display for InvalidSchemaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingProperty(err) => write!(f, "{}", err),
            Self::InvalidPropertyType(err) => write!(f, "{}", err),
        }
    }
}

impl Error for InvalidSchemaError {}

/// Error that represents a required property is missing from Notion database.
#[derive(Debug)]
pub struct InvalidSchemaMissingPropertyError {
    /// Name of the missing property.
    pub prop: String,
}

impl Display for InvalidSchemaMissingPropertyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "missing property \"{}\"", self.prop)
    }
}

impl Error for InvalidSchemaMissingPropertyError {}

/// Error that represents a property has unexpected type.
#[derive(Debug)]
pub struct InvalidSchemaPropertyTypeError {
    /// Name of the property.
    pub prop: String,

    /// The expected data type of the target property.
    pub expected_type: String,

    /// The actual data type of the target property.
    pub actual_type: String,
}

impl Display for InvalidSchemaPropertyTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "property \"{}\" should have type \"{}\", but actually it has type \"{}\"",
            self.prop, self.expected_type, self.actual_type
        )
    }
}

impl Error for InvalidSchemaPropertyTypeError {}

fn extract_notion_res_link_in_doc_node(
    node: &mut DocumentNode,
) -> Option<(&mut DocumentResourceLink, String)> {
    macro_rules! return_notion_link {
        ( $link:expr ) => {
            if let DocumentResourceLink::External { url } = $link {
                let url = url.clone();
                return Some(($link, url));
            }
        };
    }

    if let DocumentNodeTag::Image { link, .. } = &mut node.tag {
        return_notion_link!(link);
    }

    None
}

struct NotionResource {
    url: String,
    name: String,
}

impl NotionResource {
    fn new<T>(url: T) -> Option<Self>
    where
        T: Into<String>,
    {
        let url = url.into();
        let name = Uuid::new_v4().to_string();

        Some(Self { url, name })
    }
}

async fn fetch_notion_resource<T>(
    post_slug: T,
    resource: NotionResource,
) -> Result<PostResource, reqwest::Error>
where
    T: Into<String>,
{
    let post_slug = post_slug.into();
    spdlog::trace!(
        "fetch notion resource: {}/{} from {}",
        post_slug,
        resource.name,
        resource.url
    );

    let response = reqwest::get(&resource.url).await?;
    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| String::from("application/octet-stream"));
    let data = response.bytes().await?.into_iter().collect();

    Ok(PostResource {
        post_slug,
        name: resource.name,
        ty: content_type,
        data,
    })
}
