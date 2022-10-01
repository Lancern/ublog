use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Database {
    pub id: String,
    pub created_time: String,
    pub last_edited_time: String,
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
    pub last_edited_time: String,
    pub archived: bool,
    pub properties: HashMap<String, PropertyValue>,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PropertyValue {
    #[serde(rename = "title")]
    Title { title: Vec<RichText> },

    #[serde(rename = "rich_text")]
    RichText { rich_text: Vec<RichText> },

    #[serde(rename = "number")]
    Number { number: f64 },

    #[serde(rename = "select")]
    Select { select: SelectPropertyValue },

    #[serde(rename = "multi_select")]
    MultiSelect {
        multi_select: Vec<SelectPropertyValue>,
    },

    #[serde(rename = "date")]
    Date { date: DatePropertyValue },

    #[serde(rename = "checkbox")]
    Checkbox { checkbox: bool },
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
#[serde(tag = "type")]
pub enum RichTextVariants {
    #[serde(rename = "text")]
    Text { text: TextRichText },

    #[serde(rename = "equation")]
    Equation { equation: EquationRichText },
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
    pub link: Option<Link>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EquationRichText {
    pub expression: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Link {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub id: String,
    pub created_time: String,
    pub last_edited_time: String,
    pub archived: bool,
    pub has_children: bool,
    #[serde(flatten)]
    pub variant: BlockVariants,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum BlockVariants {
    #[serde(rename = "paragraph")]
    Paragraph { paragraph: ParagraphBlock },

    #[serde(rename = "heading_1")]
    Heading1 { heading_1: HeadingBlock },

    #[serde(rename = "heading_2")]
    Heading2 { heading_2: HeadingBlock },

    #[serde(rename = "heading_3")]
    Heading3 { heading_3: HeadingBlock },

    #[serde(rename = "callout")]
    Callout { callout: CalloutBlock },

    #[serde(rename = "quote")]
    Quote { quote: QuoteBlock },

    #[serde(rename = "bulleted_list_item")]
    BulletedListItem { bulleted_list_item: ListItemBlock },

    #[serde(rename = "numbered_list_item")]
    NumberedListItem { numbered_list_item: ListItemBlock },

    #[serde(rename = "code")]
    Code { code: CodeBlock },

    #[serde(rename = "image")]
    Image { image: File },

    #[serde(rename = "equation")]
    Equation { equation: EquationBlock },

    #[serde(rename = "divider")]
    Divider,

    #[serde(rename = "table")]
    Table { table: TableBlock },

    #[serde(rename = "table_row")]
    TableRow { table_row: TableRowBlock },
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
#[serde(tag = "type")]
pub enum File {
    #[serde(rename = "file")]
    NotionHostedFile { file: NotionHostedFile },

    #[serde(rename = "external")]
    ExternalFile { external: ExternalFile },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum FileOrEmoji {
    #[serde(rename = "file")]
    NotionHostedFile { file: NotionHostedFile },

    #[serde(rename = "external")]
    ExternalFile { external: ExternalFile },

    #[serde(rename = "emoji")]
    Emoji { emoji: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotionHostedFile {
    pub url: String,
    pub expiry_time: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExternalFile {
    pub url: String,
}
