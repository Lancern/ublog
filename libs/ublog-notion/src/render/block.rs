use crate::api::block_tree::{BlockTree, BlockTreeNodeVariants};
use crate::api::models::*;
use crate::render::html::{HtmlElement, HtmlNode};

/// Render a block tree into an HTML node.
pub fn render_block_tree(bt: &BlockTree) -> HtmlNode {
    let mut node = match &bt.variant {
        BlockTreeNodeVariants::PageRoot => HtmlNode::Element(HtmlElement::new("main")),
        BlockTreeNodeVariants::Block(block) => render_block(block),
        BlockTreeNodeVariants::BulletedList => HtmlNode::Element(HtmlElement::new("ul")),
        BlockTreeNodeVariants::NumberedList => HtmlNode::Element(HtmlElement::new("ol")),
    };

    if let HtmlNode::Element(element) = &mut node {
        for child_block in &bt.children {
            let child_node = render_block_tree(child_block);
            element.children.push(child_node);
        }
    }

    node
}

/// Render a Notion block into an HTML node.
pub fn render_block(b: &Block) -> HtmlNode {
    match &b.variant {
        BlockVariants::Paragraph(para) => render_paragraph_block(para),
        BlockVariants::Heading1(heading) => render_heading_block(heading, 1),
        BlockVariants::Heading2(heading) => render_heading_block(heading, 2),
        BlockVariants::Heading3(heading) => render_heading_block(heading, 3),
        BlockVariants::Callout(callout) => render_callout_block(callout),
        BlockVariants::Quote(quote) => render_quote_block(quote),
        BlockVariants::BulletedListItem(item) => render_list_item_block(item),
        BlockVariants::NumberedListItem(item) => render_list_item_block(item),
        BlockVariants::Code(code) => render_code_block(code),
        BlockVariants::Image(image) => render_image_block(image),
        BlockVariants::Equation(equation) => render_equation_block(equation),
        BlockVariants::Divider => render_divider_block(),
        BlockVariants::Table(table) => render_table_block(table),
        BlockVariants::TableRow(table_row) => render_table_row_block(table_row),
    }
}

fn render_paragraph_block(b: &ParagraphBlock) -> HtmlNode {
    let mut p_el = HtmlElement::new("p");

    let class = format!(
        "paragraph {}",
        crate::render::styles::get_color_style(&b.color)
    );
    p_el.props.insert(String::from("class"), class);
    p_el.children = render_rich_text_array(&b.rich_text);

    HtmlNode::Element(p_el)
}

fn render_heading_block(b: &HeadingBlock, level: i32) -> HtmlNode {
    debug_assert!((1..=3).contains(&level));

    const HEADING_TAGS: [&str; 4] = ["", "h1", "h2", "h3"];

    let mut h_el = HtmlElement::new(HEADING_TAGS[level as usize]);

    let class = format!(
        "heading {}",
        crate::render::styles::get_color_style(&b.color)
    );
    h_el.props.insert(String::from("class"), class);
    h_el.children = render_rich_text_array(&b.rich_text);

    HtmlNode::Element(h_el)
}

fn render_callout_block(b: &CalloutBlock) -> HtmlNode {
    let mut div_el = HtmlElement::new("div");
    let class = format!(
        "callout {}",
        crate::render::styles::get_color_style(&b.color)
    );
    div_el.props.insert(String::from("class"), class);

    if let FileOrEmoji::Emoji(emoji) = &b.icon {
        let mut icon_div_el = HtmlElement::new("div");
        icon_div_el
            .props
            .insert(String::from("class"), String::from("callout-icon"));
        icon_div_el.children.push(HtmlNode::Text(emoji.clone()));
        div_el.children.push(HtmlNode::Element(icon_div_el));
    }

    let mut content_div_el = HtmlElement::new("div");
    content_div_el
        .props
        .insert(String::from("class"), String::from("callout-content"));
    content_div_el.children = render_rich_text_array(&b.rich_text);
    div_el.children.push(HtmlNode::Element(content_div_el));

    HtmlNode::Element(div_el)
}

fn render_quote_block(b: &QuoteBlock) -> HtmlNode {
    let mut div_el = HtmlElement::new("div");

    let class = format!("quote {}", crate::render::styles::get_color_style(&b.color));
    div_el.props.insert(String::from("class"), class);
    div_el.children = render_rich_text_array(&b.rich_text);

    HtmlNode::Element(div_el)
}

fn render_list_item_block(b: &ListItemBlock) -> HtmlNode {
    let mut li_el = HtmlElement::new("li");

    let class = format!(
        "list-item {}",
        crate::render::styles::get_color_style(&b.color)
    );
    li_el.props.insert(String::from("class"), class);
    li_el.children = render_rich_text_array(&b.rich_text);

    HtmlNode::Element(li_el)
}

fn render_code_block(b: &CodeBlock) -> HtmlNode {
    let mut div_el = HtmlElement::new("div");
    div_el
        .props
        .insert(String::from("class"), String::from("code"));

    let mut caption_div_el = HtmlElement::new("div");
    caption_div_el
        .props
        .insert(String::from("class"), String::from("code-caption"));
    caption_div_el.children = render_rich_text_array(&b.caption);
    div_el.children.push(HtmlNode::Element(caption_div_el));

    let mut pre_el = HtmlElement::new("pre");
    let pre_el_class = format!("code-content {}", b.language);
    pre_el.props.insert(String::from("class"), pre_el_class);

    let code = b.rich_text.iter().map(|rt| rt.plain_text.clone()).collect();
    pre_el.children.push(HtmlNode::Text(code));

    div_el.children.push(HtmlNode::Element(pre_el));

    HtmlNode::Element(div_el)
}

fn render_image_block(b: &ImageBlock) -> HtmlNode {
    let mut image_el = HtmlElement::new("img");

    let image_url = match &b.image {
        File::ExternalFile { url } => url,
        File::NotionHostedFile { url } => url,
    };

    image_el
        .props
        .insert(String::from("class"), String::from("image"));
    image_el
        .props
        .insert(String::from("src"), image_url.clone());

    HtmlNode::Element(image_el)
}

fn render_equation_block(b: &EquationBlock) -> HtmlNode {
    let mut div_el = HtmlElement::new("div");
    div_el
        .props
        .insert(String::from("class"), String::from("equation"));
    div_el.children.push(HtmlNode::Text(b.expression.clone()));

    HtmlNode::Element(div_el)
}

fn render_divider_block() -> HtmlNode {
    HtmlNode::Element(HtmlElement::new("hr"))
}

fn render_table_block(_b: &TableBlock) -> HtmlNode {
    let mut table_el = HtmlElement::new("table");
    table_el
        .props
        .insert(String::from("class"), String::from("table"));

    HtmlNode::Element(table_el)
}

fn render_table_row_block(b: &TableRowBlock) -> HtmlNode {
    let mut tr_el = HtmlElement::new("tr");
    tr_el
        .props
        .insert(String::from("class"), String::from("table-row"));

    for cell_rt in &b.cells {
        let mut td_el = HtmlElement::new("td");
        td_el.children = render_rich_text_array(cell_rt);
        tr_el.children.push(HtmlNode::Element(td_el));
    }

    HtmlNode::Element(tr_el)
}

fn render_rich_text_array<'a, T>(rt: T) -> Vec<HtmlNode>
where
    T: IntoIterator<Item = &'a RichText>,
{
    rt.into_iter()
        .map(crate::render::rich_text::render_rich_text)
        .collect()
}
