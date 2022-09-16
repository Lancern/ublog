use time::format_description::well_known::Iso8601;
use time::Date;
use ublog_data::models::Post;
use ublog_doc::DocumentNode;

use crate::api::models::{Database, Page, PropertyValue};
use crate::api::{
    NotionApi, QueryDatabaseFilter, QueryDatabaseParams, QueryDatabasePropertyFilter,
    QueryDatabaseSort,
};
use crate::blog::{InvalidSchemaError, NotionBlogError, NotionPost};

/// Validate posts database schema.
pub async fn validate_posts_db_schema<T>(
    api: &NotionApi,
    posts_db_id: T,
) -> Result<(), NotionBlogError>
where
    T: AsRef<str>,
{
    let posts_db_id = posts_db_id.as_ref();
    let db = api.get_database(posts_db_id).await?;

    validate_posts_db_schema_on(&db)?;

    Ok(())
}

/// Get a [`QueryDatabaseParams`] object that can be used for querying post pages from the posts database.
pub fn get_query_posts_db_params() -> QueryDatabaseParams {
    QueryDatabaseParams {
        filter: Some(QueryDatabaseFilter::Property(
            QueryDatabasePropertyFilter::checkbox_checked(PUBLISHED_PROPERTY.name),
        )),
        sorts: vec![QueryDatabaseSort::descending_on(CREATE_DATE_PROPERTY.name)],
    }
}

/// Create a [`Post`] object from the corresponding Notion page.
pub fn create_post_from_page(page: &Page) -> Result<NotionPost, NotionBlogError> {
    let title = TITLE_PROPERTY.get_str_value(page);
    let slug = SLUG_PROPERTY.get_str_value(page);
    let author = AUTHOR_PROPERTY.get_str_value(page);
    let create_timestamp = CREATE_DATE_PROPERTY.get_timestamp_value(page);
    let update_timestamp = UPDATE_DATE_PROPERTY.get_timestamp_value(page);
    let category = CATEGORY_PROPERTY.get_str_value(page);
    let tags = TAGS_PROPERTY.get_str_list_value(page);

    let post = NotionPost {
        notion_page_id: page.id.clone(),
        post: Post {
            id: 0,
            title,
            slug,
            author,
            create_timestamp,
            update_timestamp,
            category,
            tags,
            views: 0,
            content: DocumentNode::new_empty(),
        },
    };
    Ok(post)
}

fn validate_posts_db_schema_on(db: &Database) -> Result<(), InvalidSchemaError> {
    for prop_desc in SCHEMA_PROPERTIES {
        match db.properties.get(prop_desc.name) {
            Some(prop) => {
                let expected_type_name = prop_desc.expected_type.name();
                if prop.ty != expected_type_name {
                    return Err(InvalidSchemaError::invalid_property_type(
                        prop_desc.name,
                        prop_desc.expected_type.name(),
                        prop.ty.clone(),
                    ));
                }
            }
            None => {
                return Err(InvalidSchemaError::missing_property(prop_desc.name));
            }
        }
    }

    Ok(())
}

macro_rules! declare_property_desc {
    ( $const_ident:ident, $name:ident, $type:ident ) => {
        const $const_ident: SchemaPropertyDescriptor = SchemaPropertyDescriptor {
            name: stringify!($name),
            expected_type: NotionPropertyTypes::$type,
        };
    };
}

declare_property_desc!(TITLE_PROPERTY, title, Title);
declare_property_desc!(SLUG_PROPERTY, slug, RichText);
declare_property_desc!(AUTHOR_PROPERTY, author, People);
declare_property_desc!(CREATE_DATE_PROPERTY, create_date, Date);
declare_property_desc!(UPDATE_DATE_PROPERTY, update_date, Date);
declare_property_desc!(CATEGORY_PROPERTY, category, Select);
declare_property_desc!(TAGS_PROPERTY, tags, MultiSelect);
declare_property_desc!(PUBLISHED_PROPERTY, published, Checkbox);

const SCHEMA_PROPERTIES: &[SchemaPropertyDescriptor] = &[
    TITLE_PROPERTY,
    SLUG_PROPERTY,
    AUTHOR_PROPERTY,
    CREATE_DATE_PROPERTY,
    UPDATE_DATE_PROPERTY,
    CATEGORY_PROPERTY,
    TAGS_PROPERTY,
    PUBLISHED_PROPERTY,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SchemaPropertyDescriptor {
    name: &'static str,
    expected_type: NotionPropertyTypes,
}

impl SchemaPropertyDescriptor {
    fn get_str_value(&self, page: &Page) -> String {
        let prop = self.get_property_value(page);
        match prop {
            PropertyValue::Title(rt) | PropertyValue::RichText(rt) => {
                crate::render::rich_text::render_rich_texts_to_plain_text(rt)
            }
            PropertyValue::Select(sel) => sel.name.clone(),
            _ => unreachable!(),
        }
    }

    fn get_str_list_value(&self, page: &Page) -> Vec<String> {
        let prop = self.get_property_value(page);
        match prop {
            PropertyValue::MultiSelect(sel) => sel.iter().map(|s| s.name.clone()).collect(),
            _ => unreachable!(),
        }
    }

    fn get_timestamp_value(&self, page: &Page) -> i64 {
        let prop = self.get_property_value(page);
        let date_prop = match prop {
            PropertyValue::Date(date) => date,
            _ => unreachable!(),
        };

        let date = Date::parse(&date_prop.start, &Iso8601::DEFAULT).unwrap();
        date.midnight().assume_utc().unix_timestamp()
    }

    fn get_property_value<'p>(&self, page: &'p Page) -> &'p PropertyValue {
        page.properties.get(self.name).unwrap()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NotionPropertyTypes {
    Title,
    RichText,
    Select,
    MultiSelect,
    Date,
    People,
    Checkbox,
}

impl NotionPropertyTypes {
    fn name(&self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::RichText => "rich_text",
            Self::Select => "select",
            Self::MultiSelect => "multi_select",
            Self::Date => "date",
            Self::People => "people",
            Self::Checkbox => "checkbox",
        }
    }
}
