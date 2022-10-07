import { MutableRefObject, PropsWithChildren } from "react";
import { BlockMath, InlineMath } from "react-katex";
import SyntaxHighlighter from "react-syntax-highlighter";
import { atomDark } from "react-syntax-highlighter/dist/cjs/styles/prism";
import "katex/dist/katex.min.css";

import { getResourceUrl } from "../data/api";
import { DocumentNode, DocumentResourceLink, InlineStyle } from "../data/model";
import { TocContext } from "./Toc";

export interface DocumentProps {
  root: DocumentNode;
  headingIdMap?: Map<DocumentNode, string>;
  tocCtx?: MutableRefObject<TocContext>;
}

export default function Document({ root, headingIdMap, tocCtx }: DocumentProps): JSX.Element {
  const getRenderedChildren = () => {
    return root.children.map((node, idx) => (
      <Document key={idx} root={node} headingIdMap={headingIdMap} tocCtx={tocCtx} />
    ));
  };

  switch (root.tag.type) {
    case "root":
      return <main>{getRenderedChildren()}</main>;

    case "paragraph":
      return <p className="first:mt-0 last:mb-0 my-4">{getRenderedChildren()}</p>;

    case "heading":
      return (
        <Heading id={headingIdMap?.get(root)} level={root.tag.level} tocCtx={tocCtx}>
          {getRenderedChildren()}
        </Heading>
      );

    case "callout":
      return <Callout emoji={root.tag.emoji}>{getRenderedChildren()}</Callout>;

    case "quote":
      return <Quote>{getRenderedChildren()}</Quote>;

    case "list":
      if (root.tag.isOrdered) {
        return <ol className="list-decimal first:mt-0 last:mb-0 my-4">{getRenderedChildren()}</ol>;
      } else {
        return <ul className="list-disc first:mt-0 last:mb-0 my-4">{getRenderedChildren()}</ul>;
      }

    case "listItem":
      return <li>{getRenderedChildren()}</li>;

    case "code":
      return <Code language={root.tag.language} code={root.tag.code} caption={root.tag.caption} />;

    case "equation":
      return <Equation expr={root.tag.expr} caption={root.tag.caption} />;

    case "image":
      return <Image resourceLink={root.tag.link} caption={root.tag.caption} />;

    case "table":
      return <Table caption={root.tag.caption}>{getRenderedChildren()}</Table>;

    case "tableRow":
      return <tr>{getRenderedChildren()}</tr>;

    case "tableCell":
      return <td>{getRenderedChildren()}</td>;

    case "divider":
      return <hr />;

    case "inline":
      return (
        <Inline style={root.tag.style} link={root.tag.link}>
          {getRenderedChildren()}
        </Inline>
      );

    case "inlineText":
      return <span>{root.tag.text}</span>;

    case "inlineCode":
      return <code className="px-2 py-1 rounded-md bg-gray-200 dark:bg-gray-700 text-sm">{root.tag.code}</code>;

    case "inlineEquation":
      return <InlineMath math={root.tag.expr} />;
  }
}

interface HeadingProps {
  level: number;
  id?: string;
  tocCtx?: MutableRefObject<TocContext>;
}

function Heading({ level, id, tocCtx, children }: PropsWithChildren<HeadingProps>): JSX.Element {
  const headingRefCallback = (el: HTMLHeadingElement) => {
    if (tocCtx && id) {
      tocCtx.current.mountDocumentHeadingElement(id, el);
    }
  };

  switch (level) {
    case 1:
      return (
        <h2 id={id} className="first:mt-0 last:mb-0 text-3xl font-bold my-8" ref={headingRefCallback}>
          {children}
        </h2>
      );
    case 2:
      return (
        <h3 id={id} className="first:mt-0 last:mb-0 text-2xl font-bold my-8" ref={headingRefCallback}>
          {children}
        </h3>
      );
    case 3:
      return (
        <h4 id={id} className="first:mt-0 last:mb-0 text-xl font-bold my-8" ref={headingRefCallback}>
          {children}
        </h4>
      );
    case 4:
      return (
        <h5 id={id} className="first:mt-0 last:mb-0 text-lg font-bold my-4" ref={headingRefCallback}>
          {children}
        </h5>
      );
    default:
      return (
        <h6 id={id} className="first:mt-0 last:mb-0 font-bold my-4" ref={headingRefCallback}>
          {children}
        </h6>
      );
  }
}

interface CalloutProps {
  emoji: string | null;
}

function Callout({ emoji, children }: PropsWithChildren<CalloutProps>): JSX.Element {
  return (
    <div className="flex bg-gray-200 first:mt-0 last:mb-0 my-4">
      <div className="flex-none w-16">
        <span className="block">{emoji}</span>
      </div>
      <div className="flex-grow">{children}</div>
    </div>
  );
}

function Quote({ children }: PropsWithChildren<{}>): JSX.Element {
  return (
    <div className="border-l-8 border-l-black bg-gray-200 first:mt-0 last:mb-0 my-4 pl-8 pr-4 py-4">{children}</div>
  );
}

interface CodeProps {
  language: string;
  code: string;
  caption: string | null;
}

function Code({ language, code, caption }: CodeProps): JSX.Element {
  const prismLanguage = CODE_LANGUAGE_MAP[language] ?? language;
  return (
    <WithCaption caption={caption}>
      <div className="first:mt-0 last:mb-0 my-4">
        <SyntaxHighlighter style={atomDark} language={prismLanguage}>
          {code}
        </SyntaxHighlighter>
      </div>
    </WithCaption>
  );
}

interface EquationProps {
  expr: string;
  caption: string | null;
}

function Equation({ expr, caption }: EquationProps): JSX.Element {
  return (
    <WithCaption caption={caption}>
      <BlockMath math={expr} />
    </WithCaption>
  );
}

interface ImageProps {
  resourceLink: DocumentResourceLink;
  caption: string | null;
}

function Image({ resourceLink, caption }: ImageProps) {
  let href: string;
  switch (resourceLink.type) {
    case "external":
      href = resourceLink.url;
      break;

    case "embedded":
      href = getResourceUrl(resourceLink.uuid).toString();
      break;
  }

  return (
    <WithCaption caption={caption}>
      <div className="flex justify-center">
        <img className="max-w-full" src={href} alt={caption ?? ""} />
      </div>
    </WithCaption>
  );
}

interface TableProps {
  caption: string | null;
}

function Table({ caption, children }: PropsWithChildren<TableProps>): JSX.Element {
  return (
    <WithCaption caption={caption} captionAtTop>
      <table>{children}</table>
    </WithCaption>
  );
}

interface InlineProps {
  style: InlineStyle | null;
  link: string | null;
}

function Inline({ style, link, children }: PropsWithChildren<InlineProps>): JSX.Element {
  const classNames = [];

  if (style) {
    if (style.bold) {
      classNames.push("font-bold");
    }
    if (style.italic) {
      classNames.push("italic");
    }
    if (style.underline) {
      classNames.push("underline");
    }
    if (style.strikeThrough) {
      classNames.push("strikeThrough");
    }

    if (style.color !== null) {
      const colorStyle: string | undefined = COLOR_STYLE_MAP[style.color];
      if (colorStyle !== undefined) {
        classNames.push(colorStyle);
      }
    }
  }

  const className = classNames.join(" ");

  if (link !== null) {
    return (
      <a className={className} href={link} target="_blank">
        {children}
      </a>
    );
  } else {
    return <span className={className}>{children}</span>;
  }
}

interface WithCaptionProps {
  caption: string | null;
  captionAtTop?: boolean;
}

function WithCaption({ caption, captionAtTop, children }: PropsWithChildren<WithCaptionProps>): JSX.Element {
  let captionElement = <></>;
  if (caption !== null) {
    captionElement = <div className="text-center text-gray-600 text-sm">{caption}</div>;
  }

  if (captionAtTop) {
    return (
      <div>
        {captionElement}
        {children}
      </div>
    );
  } else {
    return (
      <div>
        {children}
        {captionElement}
      </div>
    );
  }
}

const CODE_LANGUAGE_MAP: { [lang: string]: string } = {
  "c++": "cpp",
  "c#": "csharp",
  "f#": "fsharp",
  "objective-c": "objectivec",
  "plain text": "text",
  shell: "bash",
  "vb.net": "vbnet",
  "visual basic": "visualBasic",
  webassembly: "wasm",
  xml: "xmlDoc",
  "java/c/c++/c#": "text",
};

const COLOR_STYLE_MAP: { [color: string]: string } = {
  default: "text-black dark:text-white",
  gray: "text-gray-400 dark:text-gray-600",
  brown: "text-amber-800 dark:text-amber-700",
  orange: "text-orange-600 dark:text-orange-400",
  yellow: "text-yellow-400 dark:text-yellow-600",
  green: "text-green-600 dark:text-green-400",
  blue: "text-blue-600 dark:text-blue-400",
  purple: "text-purple-600 dark:text-purple-400",
  pink: "text-pink-600 dark:text-pink-400",
  red: "text-red-600 dark:text-red-400",
  gray_background: "bg-gray-200 dark:bg-gray-700",
  brown_background: "bg-amber-700",
  orange_background: "bg-orange-200 dark:bg-orange-700",
  yellow_background: "bg-yellow-200 dark:bg-yellow-700",
  green_background: "bg-green-200 dark:bg-green-700",
  blue_background: "bg-blue-200 dark:bg-blue-700",
  purple_background: "bg-purple-200 dark:bg-purple-700",
  pink_background: "bg-pink-200 dark:bg-pink-700",
  red_background: "bg-red-200 dark:bg-pink-700",
};
