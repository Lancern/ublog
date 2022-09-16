//! This crate defines the document tree.

use serde::{Deserialize, Serialize};

/// A node on the document tree.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DocumentNode {
    /// Tag of the node.
    ///
    /// A node's tag specifies the type of the node and any associated properties specific to the node's type.
    pub tag: DocumentNodeTag,

    /// The child nodes.
    pub children: Vec<DocumentNode>,
}

impl DocumentNode {
    /// Create a new document tree node with the given node tag.
    pub fn new(tag: DocumentNodeTag) -> Self {
        Self {
            tag,
            children: Vec::new(),
        }
    }

    /// Create a new document tree node that represents the root node of an empty document tree.
    pub fn new_empty() -> Self {
        Self::new(DocumentNodeTag::Root)
    }
}

/// A document tree node's tag.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DocumentNodeTag {
    #[serde(rename = "root")]
    Root,

    #[serde(rename = "paragraph")]
    Paragraph,

    #[serde(rename = "heading")]
    Heading { level: i32 },

    #[serde(rename = "callout")]
    Callout { emoji: Option<String> },

    #[serde(rename = "quote")]
    Quote,

    #[serde(rename = "list")]
    List {
        #[serde(rename = "isOrdered")]
        is_ordered: bool,
    },

    #[serde(rename = "listItem")]
    ListItem,

    #[serde(rename = "code")]
    Code {
        language: String,
        caption: Option<String>,
    },

    #[serde(rename = "equation")]
    Equation {
        expr: String,
        caption: Option<String>,
    },

    #[serde(rename = "image")]
    Image {
        url: String,
        caption: Option<String>,
    },

    #[serde(rename = "table")]
    Table { caption: Option<String> },

    #[serde(rename = "tableRow")]
    TableRow,

    #[serde(rename = "tableCell")]
    TableCell,

    #[serde(rename = "divider")]
    Divider,

    #[serde(rename = "inline")]
    Inline { style: Option<InlineStyle> },

    #[serde(rename = "inlineText")]
    InlineText { text: String },

    #[serde(rename = "inlineCode")]
    InlineCode { code: String },

    #[serde(rename = "inlineEquation")]
    InlineEquation { expr: String },
}

/// Style settings of an inlined document tree element.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    #[serde(rename = "strikethrough")]
    pub strike_through: bool,
    pub color: Option<String>,
}

impl InlineStyle {
    pub fn new() -> Self {
        Self::default()
    }
}
