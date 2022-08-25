use crate::api::models::*;
use crate::render::html::{HtmlElement, HtmlNode};

/// Render the given rich text into HTML.
pub fn render_rich_text(rt: &RichText) -> HtmlNode {
    let mut node = match &rt.variants {
        RichTextVariants::Text(text) => {
            if rt.annotations.code {
                render_text_rich_text(text)
            } else {
                render_code_rich_text(text)
            }
        }
        RichTextVariants::Equation(equation) => render_equation_rich_text(equation),
    };

    set_rich_text_annotation_styles(&mut node, &rt.annotations);

    HtmlNode::Element(node)
}

fn render_text_rich_text(rt: &TextRichText) -> HtmlElement {
    create_inline_element("span", rt.content.clone())
}

fn render_code_rich_text(rt: &TextRichText) -> HtmlElement {
    create_inline_element("code", rt.content.clone())
}

fn render_equation_rich_text(rt: &EquationRichText) -> HtmlElement {
    create_inline_element("span", format!("${}$", rt.expression))
}

fn create_inline_element<T>(tag: &'static str, inner_text: T) -> HtmlElement
where
    T: Into<String>,
{
    let mut span = HtmlElement::new(tag);
    span.children.push(HtmlNode::Text(inner_text.into()));
    span
}

fn set_rich_text_annotation_styles(span_element: &mut HtmlElement, annot: &RichTextAnnotations) {
    let color_style = crate::render::styles::get_color_style(&annot.color);

    let mut styles = vec![color_style];

    if annot.bold {
        styles.push(String::from("bold"));
    }

    if annot.italic {
        styles.push(String::from("italic"));
    }

    if annot.strikethrough {
        styles.push(String::from("strikethrough"));
    }

    if annot.underline {
        styles.push(String::from("underline"));
    }

    span_element
        .props
        .insert(String::from("class"), styles.join(" "));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_rich_text_annotation_styles() {
        let mut node = create_inline_element("span", "");
        let annot = RichTextAnnotations {
            bold: true,
            italic: true,
            strikethrough: true,
            underline: true,
            code: false,
            color: String::from("gray_background"),
        };

        set_rich_text_annotation_styles(&mut node, &annot);

        assert_eq!(
            node.props.get("class").unwrap(),
            "color-gray-background bold italic strikethrough underline"
        );
    }
}
