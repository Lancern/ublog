use ublog_doc::{DocumentNode, DocumentNodeTag, DocumentResourceLink};

use crate::api::block_tree::{BlockTree, BlockTreeNodeVariants};
use crate::api::models::*;

/// Render a block tree into a document tree node.
pub fn render_block_tree(bt: &BlockTree) -> DocumentNode {
    let mut node = match &bt.variant {
        BlockTreeNodeVariants::PageRoot => DocumentNode::new_empty(),
        BlockTreeNodeVariants::Block(block) => render_block(block),
        BlockTreeNodeVariants::BulletedList => {
            DocumentNode::new(DocumentNodeTag::List { is_ordered: false })
        }
        BlockTreeNodeVariants::NumberedList => {
            DocumentNode::new(DocumentNodeTag::List { is_ordered: true })
        }
    };

    for child in &bt.children {
        let child_node = render_block_tree(child);
        node.children.push(child_node);
    }

    node
}

/// Render a Notion block into a document tree node.
pub fn render_block(b: &Block) -> DocumentNode {
    match &b.variant {
        BlockVariants::Paragraph { paragraph } => render_paragraph_block(paragraph),
        BlockVariants::Heading1 { heading_1 } => render_heading_block(heading_1, 1),
        BlockVariants::Heading2 { heading_2 } => render_heading_block(heading_2, 2),
        BlockVariants::Heading3 { heading_3 } => render_heading_block(heading_3, 3),
        BlockVariants::Callout { callout } => render_callout_block(callout),
        BlockVariants::Quote { quote } => render_quote_block(quote),
        BlockVariants::BulletedListItem { bulleted_list_item } => {
            render_list_item_block(bulleted_list_item)
        }
        BlockVariants::NumberedListItem { numbered_list_item } => {
            render_list_item_block(numbered_list_item)
        }
        BlockVariants::Code { code } => render_code_block(code),
        BlockVariants::Image { image } => render_image_block(image),
        BlockVariants::Equation { equation } => render_equation_block(equation),
        BlockVariants::Divider => render_divider_block(),
        BlockVariants::Table { table } => render_table_block(table),
        BlockVariants::TableRow { table_row } => render_table_row_block(table_row),
    }
}

fn render_paragraph_block(b: &ParagraphBlock) -> DocumentNode {
    render_rich_text_container_block(&b.rich_text, DocumentNodeTag::Paragraph)
}

fn render_heading_block(b: &HeadingBlock, level: i32) -> DocumentNode {
    debug_assert!((1..=3).contains(&level));

    render_rich_text_container_block(&b.rich_text, DocumentNodeTag::Heading { level })
}

fn render_callout_block(b: &CalloutBlock) -> DocumentNode {
    let emoji = match &b.icon {
        FileOrEmoji::Emoji { emoji } => Some(emoji.clone()),
        _ => None,
    };

    render_rich_text_container_block(&b.rich_text, DocumentNodeTag::Callout { emoji })
}

fn render_quote_block(b: &QuoteBlock) -> DocumentNode {
    render_rich_text_container_block(&b.rich_text, DocumentNodeTag::Quote)
}

fn render_list_item_block(b: &ListItemBlock) -> DocumentNode {
    render_rich_text_container_block(&b.rich_text, DocumentNodeTag::ListItem)
}

fn render_code_block(b: &CodeBlock) -> DocumentNode {
    let caption = crate::render::rich_text::render_rich_texts_to_plain_text(&b.caption);
    let code = crate::render::rich_text::render_rich_texts_to_plain_text(&b.rich_text);
    DocumentNode::new(DocumentNodeTag::Code {
        language: b.language.clone(),
        caption: Some(caption),
        code,
    })
}

fn render_image_block(image_file: &File) -> DocumentNode {
    let image_url = match &image_file {
        File::ExternalFile { external } => &external.url,
        File::NotionHostedFile { file } => &file.url,
    };

    DocumentNode::new(DocumentNodeTag::Image {
        link: DocumentResourceLink::External {
            url: image_url.clone(),
        },
        caption: None,
    })
}

fn render_equation_block(b: &EquationBlock) -> DocumentNode {
    DocumentNode::new(DocumentNodeTag::Equation {
        expr: b.expression.clone(),
        caption: None,
    })
}

fn render_divider_block() -> DocumentNode {
    DocumentNode::new(DocumentNodeTag::Divider)
}

fn render_table_block(_b: &TableBlock) -> DocumentNode {
    DocumentNode::new(DocumentNodeTag::Table { caption: None })
}

fn render_table_row_block(b: &TableRowBlock) -> DocumentNode {
    let mut node = DocumentNode::new(DocumentNodeTag::TableRow);

    for cell_rt in &b.cells {
        let cell_node = render_rich_text_container_block(cell_rt, DocumentNodeTag::TableCell);
        node.children.push(cell_node);
    }

    node
}

fn render_rich_text_container_block(
    rt: &[RichText],
    block_node_tag: DocumentNodeTag,
) -> DocumentNode {
    let mut node = DocumentNode::new(block_node_tag);
    let content = crate::render::rich_text::render_rich_text_array(rt);
    node.children = vec![content];
    node
}
