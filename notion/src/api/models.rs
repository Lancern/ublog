use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Database {
    pub id: String,
    pub created_time: String,
    pub created_by: User,
    pub last_edited_time: String,
    pub last_edited_by: User,
    pub title: Vec<RichText>,
    pub description: Vec<RichText>,
    pub icon: FileOrEmoji,
    pub cover: File,
    pub url: String,
    pub properties: HashMap<String, Property>,
    pub archived: bool,
    pub is_inline: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Property {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Page {
    pub id: String,
    pub created_time: String,
    pub created_by: User,
    pub last_edited_time: String,
    pub last_edited_by: User,
    pub archived: bool,
    pub properties: HashMap<String, PropertyValue>,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub name: String,
    pub avatar_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PropertyValue {
    #[serde(rename = "title")]
    Title(Vec<RichText>),

    #[serde(rename = "rich_text")]
    RichText(Vec<RichText>),

    #[serde(rename = "number")]
    Number(f64),

    #[serde(rename = "select")]
    Select(SelectPropertyValue),

    #[serde(rename = "multi_select")]
    MultiSelect(Vec<SelectPropertyValue>),

    #[serde(rename = "date")]
    Date(DatePropertyValue),

    #[serde(rename = "checkbox")]
    Checkbox(bool),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectPropertyValue {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DatePropertyValue {
    pub start: String,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub time_zone: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RichText {
    pub plain_text: String,
    #[serde(default)]
    pub href: Option<String>,
    pub annotations: RichTextAnnotations,
    #[serde(flatten)]
    pub variants: RichTextVariants,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RichTextVariants {
    #[serde(rename = "text")]
    Text(TextRichText),

    #[serde(rename = "equation")]
    Equation(EquationRichText),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RichTextAnnotations {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextRichText {
    pub content: String,
    #[serde(default)]
    pub link: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EquationRichText {
    pub expression: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub id: String,
    pub created_time: String,
    pub created_by: User,
    pub last_edited_time: String,
    pub last_edited_by: String,
    pub archived: bool,
    pub has_children: bool,
    #[serde(flatten)]
    pub variant: BlockVariants,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BlockVariants {
    #[serde(rename = "paragraph")]
    Paragraph(ParagraphBlock),

    #[serde(rename = "heading_1")]
    Heading1(HeadingBlock),

    #[serde(rename = "heading_2")]
    Heading2(HeadingBlock),

    #[serde(rename = "heading_3")]
    Heading3(HeadingBlock),

    #[serde(rename = "callout")]
    Callout(CalloutBlock),

    #[serde(rename = "quote")]
    Quote(QuoteBlock),

    #[serde(rename = "bulleted_list_item")]
    BulletedListItem(ListItemBlock),

    #[serde(rename = "numbered_list_item")]
    NumberedListItem(ListItemBlock),

    #[serde(rename = "code")]
    Code(CodeBlock),

    #[serde(rename = "image")]
    Image(ImageBlock),

    #[serde(rename = "equation")]
    Equation(EquationBlock),

    #[serde(rename = "divider")]
    Divider,

    #[serde(rename = "table")]
    Table(TableBlock),

    #[serde(rename = "table_row")]
    TableRow(TableRowBlock),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParagraphBlock {
    pub rich_text: Vec<RichText>,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeadingBlock {
    pub rich_text: Vec<RichText>,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CalloutBlock {
    pub rich_text: Vec<RichText>,
    pub icon: FileOrEmoji,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuoteBlock {
    pub rich_text: Vec<RichText>,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListItemBlock {
    pub rich_text: Vec<RichText>,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CodeBlock {
    pub rich_text: Vec<RichText>,
    pub caption: Vec<RichText>,
    pub language: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageBlock {
    pub image: File,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EquationBlock {
    pub expression: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TableBlock {
    pub table_width: u32,
    pub has_column_header: bool,
    pub has_row_header: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TableRowBlock {
    pub cells: Vec<Vec<RichText>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum File {
    #[serde(rename = "file")]
    NotionHostedFile { url: String },
    #[serde(rename = "external")]
    ExternalFile { url: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FileOrEmoji {
    #[serde(rename = "file")]
    NotionHostedFile { url: String },
    #[serde(rename = "external")]
    ExternalFile { url: String },
    #[serde(rename = "emoji")]
    Emoji(String),
}
