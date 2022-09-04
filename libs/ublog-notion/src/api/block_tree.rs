use crate::api::models::Block;

use super::models::BlockVariants;

/// A node on the normalized block tree.
///
/// A normalized block tree differs from a raw block tree (see [`RawBlockTree`]) in:
/// - A root node that represents the entire page is added in the normalized block tree;
/// - Adjacent blocks that represent list items of the same list type are grouped into special list node in the
///   normalized block tree.
///
/// To convert a raw block tree into a block tree, use the [`normalize`] function.
#[derive(Clone, Debug)]
pub struct BlockTree {
    /// Variant of this node.
    pub variant: BlockTreeNodeVariants,

    /// Child nodes of this node on the block tree.
    pub children: Vec<BlockTree>,
}

impl BlockTree {
    /// Create a new block tree node that represents the root of a page.
    pub fn new_page_root() -> Self {
        Self {
            variant: BlockTreeNodeVariants::PageRoot,
            children: Vec::new(),
        }
    }

    /// Create a new block tree node that represents a block in the document.
    pub fn new_block(block: Block) -> Self {
        Self {
            variant: BlockTreeNodeVariants::Block(block),
            children: Vec::new(),
        }
    }

    /// Create a new block tree node that represents a bulleted list.
    pub fn new_bulleted_list() -> Self {
        Self {
            variant: BlockTreeNodeVariants::BulletedList,
            children: Vec::new(),
        }
    }

    /// Create a new block tree node that represents a numbered list.
    pub fn new_numbered_list() -> Self {
        Self {
            variant: BlockTreeNodeVariants::NumberedList,
            children: Vec::new(),
        }
    }

    /// Get the block contained in this block tree node, if any.
    pub fn block(&self) -> Option<&Block> {
        if let BlockTreeNodeVariants::Block(block) = &self.variant {
            Some(block)
        } else {
            None
        }
    }

    /// Get the block contained in this block tree node, if any.
    pub fn block_mut(&mut self) -> Option<&mut Block> {
        if let BlockTreeNodeVariants::Block(block) = &mut self.variant {
            Some(block)
        } else {
            None
        }
    }
}

/// Variants of a normalized block tree node.
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum BlockTreeNodeVariants {
    /// Node that represents a page.
    PageRoot,

    /// Node that represents a single block in the document.
    Block(Block),

    /// Node that represents a bulleted list in the document.
    BulletedList,

    /// Node that represents a numbered list in the document.
    NumberedList,
}

impl BlockTreeNodeVariants {
    /// Determine whether this block tree node variant is the page root.
    pub fn is_page_root(&self) -> bool {
        matches!(self, Self::PageRoot)
    }

    /// Determine whether this block tree node variant is a block node.
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block(_))
    }

    /// Determine whether this block tree node variant is a bulleted list node.
    pub fn is_bulleted_list(&self) -> bool {
        matches!(self, Self::BulletedList)
    }

    /// Determine whether this block tree node variant is a numbered list node.
    pub fn is_numbered_list(&self) -> bool {
        matches!(self, Self::NumberedList)
    }
}

/// A node in the raw block tree.
///
/// A raw block tree can be directly built from responses from Notion APIs. However, raw block trees are not well suited
/// to be rendered into HTML documents. Instead, before rendering, we "normalize" raw block trees into a block tree (see
/// [`BlockTree`]) and then render the block tree into HTML documents.
///
/// To normalize raw block trees into a block tree, use the [`normalize`] function.
#[derive(Clone, Debug)]
pub struct RawBlockTree {
    /// Block contained in the node.
    pub block: Block,

    /// Child nodes of this raw block tree node.
    pub children: Vec<RawBlockTree>,
}

impl RawBlockTree {
    /// Create a new raw block tree node.
    pub fn new(block: Block) -> Self {
        Self {
            block,
            children: Vec::new(),
        }
    }
}

/// Normalize raw block trees into a block tree.
pub fn normalize(raw: Vec<RawBlockTree>) -> BlockTree {
    let mut root = BlockTree::new_page_root();
    normalize_as_children(raw, &mut root);
    root
}

fn normalize_as_children(raw: Vec<RawBlockTree>, root: &mut BlockTree) {
    let mut active_tree: Option<BlockTree> = None;

    for raw_tree in raw {
        let mut tree = BlockTree::new_block(raw_tree.block);
        normalize_as_children(raw_tree.children, &mut tree);

        let block = tree.block().unwrap();
        match (&block.variant, active_tree.as_ref().map(|t| &t.variant)) {
            (BlockVariants::BulletedListItem(_), Some(BlockTreeNodeVariants::BulletedList))
            | (BlockVariants::NumberedListItem(_), Some(BlockTreeNodeVariants::NumberedList)) => {
                active_tree.as_mut().unwrap().children.push(tree);
                continue;
            }
            _ => {}
        }

        if let Some(active_tree) = active_tree.take() {
            root.children.push(active_tree);
        }

        let mut list_node = match &block.variant {
            BlockVariants::BulletedListItem(_) => BlockTree::new_bulleted_list(),
            BlockVariants::NumberedListItem(_) => BlockTree::new_numbered_list(),
            _ => {
                root.children.push(tree);
                continue;
            }
        };
        list_node.children.push(tree);
        active_tree = Some(list_node);
    }

    if let Some(tree) = active_tree {
        root.children.push(tree);
    }
}
