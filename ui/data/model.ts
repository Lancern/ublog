export interface Post {
  title: string;
  slug: string;
  author: string;
  createTimestamp: number;
  updateTimestamp: number;
  category: string;
  tags: string[];
  content: DocumentNode;
}

export interface DocumentNode {
  tag: DocumentNodeTag;
  children: DocumentNode[];
}

export type DocumentNodeTag =
  | DocumentNodeRootTag
  | DocumentNodeParagraphTag
  | DocumentNodeHeadingTag
  | DocumentNodeCalloutTag
  | DocumentNodeQuoteTag
  | DocumentNodeListTag
  | DocumentNodeListItemTag
  | DocumentNodeCodeTag
  | DocumentNodeEquationTag
  | DocumentNodeImageTag
  | DocumentNodeTableTag
  | DocumentNodeTableRowTag
  | DocumentNodeTableCellTag
  | DocumentNodeDividerTag
  | DocumentNodeInlineTag
  | DocumentNodeInlineTextTag
  | DocumentNodeInlineCodeTag
  | DocumentNodeInlineEquationTag;

export interface DocumentNodeRootTag {
  type: "root";
}

export interface DocumentNodeParagraphTag {
  type: "paragraph";
}

export interface DocumentNodeHeadingTag {
  type: "heading";
  level: number;
}

export interface DocumentNodeCalloutTag {
  type: "callout";
  emoji: string | null;
}

export interface DocumentNodeQuoteTag {
  type: "quote";
}

export interface DocumentNodeListTag {
  type: "list";
  isOrdered: boolean;
}

export interface DocumentNodeListItemTag {
  type: "listItem";
}

export interface DocumentNodeCodeTag {
  type: "code";
  language: string;
  caption: string | null;
  code: string;
}

export interface DocumentNodeEquationTag {
  type: "equation";
  expr: string;
  caption: string | null;
}

export interface DocumentNodeImageTag {
  type: "image";
  link: DocumentResourceLink;
  caption: string | null;
}

export interface DocumentNodeTableTag {
  type: "table";
  caption: string | null;
}

export interface DocumentNodeTableRowTag {
  type: "tableRow";
}

export interface DocumentNodeTableCellTag {
  type: "tableCell";
}

export interface DocumentNodeDividerTag {
  type: "divider";
}

export interface DocumentNodeInlineTag {
  type: "inline";
  style: InlineStyle | null;
  link: string | null;
}

export interface DocumentNodeInlineTextTag {
  type: "inlineText";
  text: string;
}

export interface DocumentNodeInlineCodeTag {
  type: "inlineCode";
  code: string;
}

export interface DocumentNodeInlineEquationTag {
  type: "inlineEquation";
  expr: string;
}

export type DocumentResourceLink = DocumentResourceExternalLink | DocumentResourceEmbeddedLink;

export interface DocumentResourceExternalLink {
  type: "external";
  url: string;
}

export interface DocumentResourceEmbeddedLink {
  type: "embedded";
  uuid: string;
}

export interface InlineStyle {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strikeThrough: boolean;
  color: string | null;
}
