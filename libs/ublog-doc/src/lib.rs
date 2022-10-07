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

    /// Visit all nodes in the document tree rooted at this document node.
    pub fn visit<V>(&self, visitor: &mut V)
    where
        V: ?Sized + DocumentNodeVisitor,
    {
        visitor.visit(self);
        for child in &self.children {
            child.visit(visitor);
        }
    }

    /// Visit all nodes in the document tree rooted at this document node.
    pub fn visit_mut<V>(&mut self, visitor: &mut V)
    where
        V: ?Sized + DocumentNodeVisitor,
    {
        visitor.visit_mut(self);
        for child in &mut self.children {
            child.visit_mut(visitor);
        }
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

    #[serde(rename = "list", rename_all = "camelCase")]
    List { is_ordered: bool },

    #[serde(rename = "listItem")]
    ListItem,

    #[serde(rename = "code")]
    Code {
        language: String,
        caption: Option<String>,
        code: String,
    },

    #[serde(rename = "equation")]
    Equation {
        expr: String,
        caption: Option<String>,
    },

    #[serde(rename = "image")]
    Image {
        link: DocumentResourceLink,
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
    Inline {
        style: Option<InlineStyle>,
        link: Option<String>,
    },

    #[serde(rename = "inlineText")]
    InlineText { text: String },

    #[serde(rename = "inlineCode")]
    InlineCode { code: String },

    #[serde(rename = "inlineEquation")]
    InlineEquation { expr: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum DocumentResourceLink {
    #[serde(rename = "external")]
    External { url: String },

    #[serde(rename = "embedded")]
    Embedded { uuid: String },
}

/// Style settings of an inlined document tree element.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike_through: bool,
    pub color: Option<String>,
}

impl InlineStyle {
    /// Create a default `InlineStyle` value.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Visitor that visits each document node within a document tree.
pub trait DocumentNodeVisitor {
    fn visit(&mut self, _node: &DocumentNode) {}
    fn visit_mut(&mut self, _node: &mut DocumentNode) {}
}
