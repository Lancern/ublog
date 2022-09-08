//! This crate defines the document tree.

pub mod builder;

use serde::{Deserialize, Serialize};

/// A node within the document tree.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Node {
    #[serde(rename = "document")]
    Document(DocumentNode),

    #[serde(rename = "paragraph")]
    Paragraph(ParagraphNode),

    #[serde(rename = "heading")]
    Heading(HeadingNode),

    #[serde(rename = "callout")]
    Callout(CalloutNode),

    #[serde(rename = "quote")]
    Quote(QuoteNode),

    #[serde(rename = "list")]
    List(ListNode),

    #[serde(rename = "code")]
    Code(CodeNode),

    #[serde(rename = "equation")]
    Equation(EquationNode),

    #[serde(rename = "table")]
    Table(TableNode),

    #[serde(rename = "divider")]
    Divider,
}

/// The root node of a document tree, which represents the whole document.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DocumentNode {
    /// Child nodes of this document node.
    pub children: Vec<Node>,
}

/// A document node that represents a paragraph.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParagraphNode {
    /// Content of this paragraph.
    pub content: Vec<RichText>,
}

/// A document node that represents a section heading.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeadingNode {
    /// Level of the heading.
    pub level: u8,

    /// Content of the heading.
    pub heading: Vec<RichText>,
}

/// A document node that represents a callout block.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CalloutNode {
    /// Emoji of the callout block.
    pub emoji: String,

    /// Content within the callout block.
    pub content: Vec<RichText>,
}

/// A document node that represents a quote block.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QuoteNode {
    /// Content of the quote block.
    pub content: Vec<RichText>,
}

/// A document node that represents an ordered or unordered list.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListNode {
    /// Is this list an ordered list?
    pub is_ordered: bool,

    /// Items within this list.
    pub items: Vec<ListItem>,
}

/// A list item.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListItem {
    /// Content of the list item.
    pub content: Vec<RichText>,

    /// Indented child lists within the list item.
    pub children: Vec<ListNode>,
}

/// A document node that represents a code block.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CodeNode {
    /// Language of code.
    pub lang: String,

    /// Code represented as a string.
    pub code: String,
}

/// A document node that represents a math equation block.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EquationNode {
    /// LaTeX expression of the equation.
    pub expr: String,
}

/// A document node that represents a table.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TableNode {
    /// Is the first row of the table a heading row?
    pub head_row: Option<TableRow>,

    /// Rows contained in the table.
    pub rows: Vec<TableRow>,
}

/// A row within a table.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TableRow {
    /// Cells within the row.
    pub cells: Vec<TableCell>,
}

/// A cell within a table row.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TableCell {
    /// Content of the table cell.
    pub content: Vec<Node>,
}

/// A span of rich text.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RichText {
    /// A span of plain text.
    Text(String),

    /// A span of plain text in monospace style.
    Code(String),

    /// An inline math equation.
    Equation(String),

    /// A span of rich text that shares the same set of style options.
    Styled(StyledRichText),
}

/// Style options of a span of rich text.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StyledRichText {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike_through: bool,
    pub color: Option<String>,
    pub content: Vec<RichText>,
}
