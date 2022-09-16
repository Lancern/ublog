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
    Root,
    Paragraph,
    Heading {
        level: i32,
    },
    Callout {
        emoji: Option<String>,
    },
    Quote,
    List {
        is_ordered: bool,
    },
    ListItem,
    Code {
        language: String,
        caption: Option<String>,
    },
    Equation {
        expr: String,
        caption: Option<String>,
    },
    Image {
        url: String,
        caption: Option<String>,
    },
    Table {
        caption: Option<String>,
    },
    TableRow,
    TableCell,
    Divider,
    Inline {
        style: Option<InlineStyle>,
    },
    InlineText {
        text: String,
    },
    InlineCode {
        code: String,
    },
    InlineEquation {
        expr: String,
    },
}

/// Style settings of an inlined document tree element.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike_through: bool,
    pub color: Option<String>,
}

impl InlineStyle {
    pub fn new() -> Self {
        Self::default()
    }
}
