import { forwardRef, LegacyRef, MutableRefObject, useEffect } from "react";

export class TocContext {
  private readonly _referenceElements: Map<string, TocReferenceState>;

  public constructor() {
    this._referenceElements = new Map();
  }

  public forEachTocReference(callback: (headingEl: DocumentHeadingElement, tocItemEl: TocItemElement) => void) {
    this._referenceElements.forEach((state) => {
      if (state.documentHeadingElement === null || state.tocItemElement === null) {
        return;
      }
      callback(state.documentHeadingElement, state.tocItemElement);
    });
  }

  public collectTocReferences(): TocReferenceItem[] {
    const items: TocReferenceItem[] = [];
    this.forEachTocReference((headingEl, tocItemEl) => {
      items.push({
        tocItemElement: tocItemEl,
        documentHeadingElement: headingEl,
      });
    });

    items.sort((lhs, rhs) => {
      const lhsY = lhs.documentHeadingElement.getBoundingClientRect().top;
      const rhsY = rhs.documentHeadingElement.getBoundingClientRect().top;
      return lhsY - rhsY;
    });

    return items;
  }

  public mountDocumentHeadingElement(targetId: string, el: DocumentHeadingElement) {
    const state = this._referenceElements.get(targetId);
    if (state) {
      state.documentHeadingElement = el;
    } else {
      this._referenceElements.set(targetId, {
        documentHeadingElement: el,
        tocItemElement: null,
      });
    }
  }

  public mountTocItemElement(targetId: string, el: TocItemElement) {
    const state = this._referenceElements.get(targetId);
    if (state) {
      state.tocItemElement = el;
    } else {
      this._referenceElements.set(targetId, {
        documentHeadingElement: null,
        tocItemElement: el,
      });
    }
  }
}

export type TocItemElement = HTMLAnchorElement;
export type DocumentHeadingElement = HTMLHeadingElement;

export interface TocReferenceItem {
  tocItemElement: TocItemElement;
  documentHeadingElement: DocumentHeadingElement;
}

export interface TocEntry {
  title: string;
  level: number;
  targetId: string;
}

export interface TocNavProps {
  entries: TocEntry[];
  ctx?: MutableRefObject<TocContext>;
}

export function TocNav({ entries, ctx }: TocNavProps): JSX.Element {
  const NavItem = forwardRef(TocNavItem);
  return (
    <nav>
      {entries.map((entry, idx) => (
        <NavItem
          key={idx}
          entry={entry}
          ref={(el) => {
            if (ctx) {
              ctx.current.mountTocItemElement(entry.targetId, el);
            }
          }}
        />
      ))}
    </nav>
  );
}

export interface TocScrollSpyProps {
  ctx: MutableRefObject<TocContext>;
}

export function TocScrollSpy({ ctx }: TocScrollSpyProps): JSX.Element {
  useEffect(() => {
    const scrollSpyUpdate = createTocScrollSpyUpdateCallback(ctx);
    window.addEventListener("scroll", scrollSpyUpdate);
    return () => {
      window.removeEventListener("scroll", scrollSpyUpdate);
    };
  }, [ctx]);

  return <></>;
}

interface TocReferenceState {
  documentHeadingElement: DocumentHeadingElement | null;
  tocItemElement: TocItemElement | null;
}

interface TocNavItemProps {
  entry: TocEntry;
}

function TocNavItem({ entry }: TocNavItemProps, ref?: LegacyRef<TocItemElement | null>): JSX.Element {
  const indentStyle = ["pl-4", "pl-8", "pl-12"][entry.level - 1];
  return (
    <a
      className="block first:pt-0 last:pb-0 border-l-2 border-l-transparent py-1 hover:text-blue-600 dark:hover:text-blue-400 text-sm"
      href={`#${entry.targetId}`}
      ref={ref}
    >
      <div className={indentStyle}>{entry.title}</div>
    </a>
  );
}

function createTocScrollSpyUpdateCallback(ctx: MutableRefObject<TocContext>): (this: Window, ev: Event) => void {
  let activeItem: TocReferenceItem | null = null;
  const updateActiveItem = (newActiveItem: TocReferenceItem | null) => {
    const activeTocItemClassList = ["font-black", "border-l-black", "dark:border-l-white"];
    const inactiveTocItemClassList = ["border-l-transparent"];

    if (newActiveItem === activeItem) {
      return;
    }

    if (activeItem !== null && activeItem.tocItemElement !== null) {
      activeItem.tocItemElement.classList.remove(...activeTocItemClassList);
      activeItem.tocItemElement.classList.add(...inactiveTocItemClassList);
    }

    if (newActiveItem !== null && newActiveItem.tocItemElement !== null) {
      newActiveItem.tocItemElement.classList.add(...activeTocItemClassList);
      newActiveItem.tocItemElement.classList.remove(...inactiveTocItemClassList);
    }

    activeItem = newActiveItem;
  };

  return function (this: Window) {
    const viewportHeight = this.visualViewport.height;
    let nextActiveItem: TocReferenceItem | null = null;

    const tocReferences = ctx.current.collectTocReferences();
    for (const referenceItem of tocReferences) {
      if (referenceItem.documentHeadingElement === null) {
        continue;
      }

      const boundingClientRect = referenceItem.documentHeadingElement.getBoundingClientRect();
      if (boundingClientRect.top <= viewportHeight / 2) {
        nextActiveItem = referenceItem;
      }
      if (boundingClientRect.bottom >= 0) {
        break;
      }
    }

    updateActiveItem(nextActiveItem);
  };
}
