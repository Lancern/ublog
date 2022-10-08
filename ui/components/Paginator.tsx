import { PropsWithChildren } from "react";

export interface PaginatorProps {
  currentPage: number;
  maxPage: number;
  onPageChanged?: (newPage: number) => void;
}

export default function Paginator({ currentPage, maxPage, onPageChanged }: PaginatorProps): JSX.Element {
  const shownPagesSet = new Set([1, 2, 3, currentPage - 1, currentPage, currentPage + 1, maxPage]);
  const shownPages = Array.from(shownPagesSet).filter((page) => page >= 1 && page <= maxPage);
  shownPages.sort((lhs, rhs) => lhs - rhs);

  const items: JSX.Element[] = [];

  items.push(
    <PaginatorButton key={-1} disabled={currentPage <= 1} targetPage={currentPage - 1} onPageChanged={onPageChanged}>
      Back
    </PaginatorButton>
  );

  let nextKey = 0;
  shownPages.forEach((page, idx) => {
    if (idx > 0 && page - shownPages[idx - 1] > 1) {
      items.push(<PaginatorSeparator key={nextKey} />);
      nextKey++;
    }

    items.push(
      <PaginatorButton key={nextKey} active={page === currentPage} targetPage={page} onPageChanged={onPageChanged}>
        {page}
      </PaginatorButton>
    );
    nextKey++;
  });

  items.push(
    <PaginatorButton
      key={-2}
      disabled={currentPage >= maxPage}
      targetPage={currentPage + 1}
      onPageChanged={onPageChanged}
    >
      Next
    </PaginatorButton>
  );

  return <nav className="flex justify-center items-center border-t border-gray-200">{items}</nav>;
}

interface PaginatorButtonProps {
  targetPage: number;
  active?: boolean;
  disabled?: boolean;
  onPageChanged?: (newPage: number) => void;
}

function PaginatorButton({
  targetPage,
  active,
  disabled,
  onPageChanged,
  children,
}: PropsWithChildren<PaginatorButtonProps>): JSX.Element {
  const invokePageChanged = (page: number) => {
    if (onPageChanged) {
      onPageChanged(page);
    }
  };

  if (disabled) {
    return (
      <div className="flex justify-center p-2 min-w-[2.5rem] text-gray-600 dark:text-gray-400 cursor-not-allowed">
        {children}
      </div>
    );
  }

  if (active) {
    return (
      <div className="flex justify-center p-2 min-w-[2.5rem] bg-gray-800 dark:bg-gray-200 text-white dark:text-black">
        {children}
      </div>
    );
  }

  return (
    <button
      className="block p-2 min-w-[2.5rem] dark:text-white hover:bg-gray-300 dark:hover:bg-gray-700 focus:bg-gray-600 dark:focus:bg-gray-400 focus:text-white dark:focus:text-black focus:ring-4 focus:ring-gray-400 dark:focus:ring-gray-600 transition"
      onClick={() => invokePageChanged(targetPage)}
    >
      {children}
    </button>
  );
}

function PaginatorSeparator(): JSX.Element {
  return <div className="flex justify-center p-2 min-w-[2.5rem]">...</div>;
}
