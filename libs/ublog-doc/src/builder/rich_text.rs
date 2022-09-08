//! This module provides builders for rich texts.

use crate::*;

macro_rules! declare_builder_method {
    (
        $(
            $field_name:ident: $field_ty:ty
        ),+
        $(,)?
    ) => {
        $(
            pub fn $field_name(&mut self, value: $field_ty) -> &mut Self {
                self.$field_name = value;
                self
            }
        )+
    };
}

/// Build a vector of rich texts, which can represent the content of some element within the document tree.
#[derive(Clone, Debug, Default)]
pub struct RichTextBuilder {
    spans: Vec<RichText>,
}

impl RichTextBuilder {
    /// Create a new `RichTextBuilder` object.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add plain text to the end of the rich text vector being built.
    pub fn push_text<T>(&mut self, text: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.spans.push(RichText::Text(text.into()));
        self
    }

    /// Add plain text with monospace style to the end of the rich text vector being built.
    pub fn push_code<T>(&mut self, code: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.spans.push(RichText::Code(code.into()));
        self
    }

    /// Add an inline math equation to the end of the rich text vector being built.
    pub fn push_equation<T>(&mut self, expr: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.spans.push(RichText::Equation(expr.into()));
        self
    }

    /// Build a styled rich text span and add it to the end of the rich text vector being built.
    ///
    /// The given function is called to build the styled rich text span.
    pub fn push_styled<F>(&mut self, build_styled: F) -> &mut Self
    where
        F: FnOnce(&mut RichTextStyleBuilder, &mut Self),
    {
        let mut style_builder = RichTextStyleBuilder::new();
        let mut child_builder = Self::new();

        build_styled(&mut style_builder, &mut child_builder);

        let mut styled = StyledRichText {
            bold: false,
            italic: false,
            underline: false,
            strike_through: false,
            color: None,
            content: child_builder.build(),
        };
        style_builder.populate_style(&mut styled);

        self.spans.push(RichText::Styled(styled));
        self
    }

    pub fn build(self) -> Vec<RichText> {
        todo!()
    }
}

/// Build rich text style options.
#[derive(Clone, Debug, Default)]
pub struct RichTextStyleBuilder {
    bold: bool,
    italic: bool,
    underline: bool,
    strike_through: bool,
    monospace: bool,
    color: Option<String>,
}

impl RichTextStyleBuilder {
    /// Create a new `RichTextStyleBuilder` object.
    pub fn new() -> Self {
        Self::default()
    }

    /// Populate the style options contained in the given `StyledRichText` with the options configured in this builder.
    pub fn populate_style(&self, styled: &mut StyledRichText) {
        styled.bold = self.bold;
        styled.italic = self.italic;
        styled.underline = self.underline;
        styled.strike_through = self.strike_through;
        styled.color = self.color.clone();
    }

    declare_builder_method! {
        bold: bool,
        italic: bool,
        underline: bool,
        strike_through: bool,
        monospace: bool,
        color: Option<String>,
    }
}
