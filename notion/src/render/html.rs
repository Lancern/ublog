use std::collections::HashMap;

/// A node in the HTML document tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlNode {
    Text(String),
    Element(HtmlElement),
}

impl HtmlNode {
    /// Convert the node itself and all its contents into a string.
    pub fn to_html_str(&self, output: &mut String) {
        match self {
            Self::Text(text) => {
                let escaped = html_escape::encode_text(text);
                output.push_str(&*escaped);
            }
            Self::Element(element) => {
                element.to_html_str(output);
            }
        }
    }
}

/// An XML-like element in the HTML document tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlElement {
    /// The tag name of the element.
    pub tag: &'static str,

    /// The properties of the element.
    pub props: HashMap<String, String>,

    /// The child nodes of the element.
    pub children: Vec<HtmlNode>,
}

impl HtmlElement {
    /// Convert the node itself and all its contents into a string.
    pub fn to_html_str(&self, output: &mut String) {
        let mut tags = vec![String::from(self.tag)];
        for (k, v) in &self.props {
            tags.push(format!("{}=\"{}\"", k, v));
        }

        if self.children.is_empty() {
            output.push_str(&format!("<{}/>", tags.join(" ")));
            return;
        }

        output.push_str(&format!("<{}>", tags.join(" ")));

        for child_node in &self.children {
            child_node.to_html_str(output);
        }

        output.push_str(&format!("</{}>", self.tag));
    }
}

impl HtmlElement {
    /// Create a new HTML element with the given tag name.
    ///
    /// The new element does not have any properties or children.
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            props: HashMap::new(),
            children: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_html_text_node() {
        let node = HtmlNode::Text(String::from("hello"));

        let mut rendered = String::new();
        node.to_html_str(&mut rendered);

        assert_eq!(rendered, "hello");
    }

    #[test]
    fn test_render_html_element_no_children() {
        let mut el = HtmlElement::new("hr");
        el.props
            .insert(String::from("class"), String::from("divider"));

        let mut rendered = String::new();
        el.to_html_str(&mut rendered);

        assert_eq!(rendered, r#"<hr class="divider"/>"#);
    }

    #[test]
    fn test_render_html_element_with_children() {
        let mut el = HtmlElement::new("div");
        el.props
            .insert(String::from("class"), String::from("paragraph"));

        let mut span_el_1 = HtmlElement::new("span");
        span_el_1
            .children
            .push(HtmlNode::Text(String::from("hello")));
        el.children.push(HtmlNode::Element(span_el_1));

        let mut span_el_2 = HtmlElement::new("span");
        span_el_2
            .children
            .push(HtmlNode::Text(String::from("world")));
        el.children.push(HtmlNode::Element(span_el_2));

        let mut rendered = String::new();
        el.to_html_str(&mut rendered);

        assert_eq!(
            rendered,
            r#"<div class="paragraph"><span>hello</span><span>world</span></div>"#
        );
    }
}
