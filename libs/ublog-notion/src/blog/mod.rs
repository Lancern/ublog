pub mod schema;

use std::error::Error;
use std::fmt::{Display, Formatter};

use ublog_data::models::Post;

use crate::api::{NotionApi, NotionApiError};

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

    crate::blog::schema::validate_posts_db_schema(api, posts_db_id).await?;

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
    let raw_content_trees = api.get_page_content(&post.notion_page_id).await?;
    let content_tree = crate::api::block_tree::normalize(raw_content_trees);

    // TODO: implement get_post_content function.
    // Render the content tree to document tree.
    todo!()
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
}

impl Display for NotionBlogError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotionApi(err) => write!(f, "Notion API error: {}", err),
            Self::InvalidSchema(err) => write!(f, "schema validation failed: {}", err),
        }
    }
}

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
