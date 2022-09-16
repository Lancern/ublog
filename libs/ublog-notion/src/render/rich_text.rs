use ublog_doc::{DocumentNode, DocumentNodeTag, InlineStyle};

use crate::api::models::*;

/// Render the given rich text array.
pub fn render_rich_text_array<'a, T>(rt: T) -> DocumentNode
where
    T: IntoIterator<Item = &'a RichText>,
{
    let mut node = DocumentNode::new(DocumentNodeTag::Inline { style: None });
    node.children = rt.into_iter().map(render_rich_text).collect();
    node
}

/// Render the given rich text.
pub fn render_rich_text(rt: &RichText) -> DocumentNode {
    let rendered = match &rt.variants {
        RichTextVariants::Text(text) => {
            if rt.annotations.code {
                render_text_rich_text(text)
            } else {
                render_code_rich_text(text)
            }
        }
        RichTextVariants::Equation(equation) => render_equation_rich_text(equation),
    };

    render_style(&rt.annotations, rendered)
}

/// Render the given rich text into plain text, discarding any style settings.
pub fn render_rich_texts_to_plain_text<'a, I>(rt: I) -> String
where
    I: IntoIterator<Item = &'a RichText>,
{
    let mut rendered = String::new();

    for item in rt {
        rendered.push_str(&item.plain_text);
    }

    rendered
}

fn render_text_rich_text(rt: &TextRichText) -> DocumentNode {
    DocumentNode::new(DocumentNodeTag::InlineText {
        text: rt.content.clone(),
    })
}

fn render_code_rich_text(rt: &TextRichText) -> DocumentNode {
    DocumentNode::new(DocumentNodeTag::InlineCode {
        code: rt.content.clone(),
    })
}

fn render_equation_rich_text(rt: &EquationRichText) -> DocumentNode {
    DocumentNode::new(DocumentNodeTag::InlineEquation {
        expr: rt.expression.clone(),
    })
}

fn render_style(annot: &RichTextAnnotations, inner_rendered: DocumentNode) -> DocumentNode {
    let mut style = InlineStyle::new();

    if annot.bold {
        style.bold = true;
    }

    if annot.italic {
        style.italic = true;
    }

    if annot.underline {
        style.underline = true;
    }

    if annot.strikethrough {
        style.strike_through = true;
    }

    if style == InlineStyle::new() {
        inner_rendered
    } else {
        let mut wrapper = DocumentNode::new(DocumentNodeTag::Inline { style: Some(style) });
        wrapper.children = vec![inner_rendered];
        wrapper
    }
}
