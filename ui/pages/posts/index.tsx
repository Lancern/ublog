import type { GetServerSidePropsContext, GetServerSidePropsResult } from "next";
import { useRouter } from "next/router";
import type { ParsedUrlQuery } from "querystring";

import Paginator from "../../components/Paginator";
import PostList from "../../components/PostList";
import { getPosts } from "../../data/api";
import type { Post } from "../../data/model";

interface PostsPageProps {
  page: number;
  totalPages: number;
  posts: Post[];
}

export default function PostsPage({ page, totalPages, posts }: PostsPageProps): JSX.Element {
  const router = useRouter();

  const setPage = (newPage: number) => {
    router.push(`/posts?page=${newPage}`);
  };

  return (
    <div>
      <PostList posts={posts} />
      <Paginator currentPage={page} maxPage={totalPages} onPageChanged={setPage} />
    </div>
  );
}

export async function getServerSideProps(
  context: GetServerSidePropsContext
): Promise<GetServerSidePropsResult<PostsPageProps>> {
  const ITEMS_PER_PAGE = 10;

  const page = getPageFromQuery(context.query);
  const posts = await getPosts({
    page,
    itemsPerPage: ITEMS_PER_PAGE,
  });

  const totalPages = Math.ceil(posts.totalCount / ITEMS_PER_PAGE);
  if (page > totalPages) {
    return {
      notFound: true,
    };
  }

  return {
    props: {
      page,
      totalPages,
      posts: posts.objects,
    },
  };
}

function getPageFromQuery(query: ParsedUrlQuery): number {
  const pageQuery = query.page;

  let page = 1;
  if (pageQuery !== undefined) {
    if (Array.isArray(pageQuery)) {
      page = parseInt(pageQuery[0]);
    } else {
      page = parseInt(pageQuery);
    }
  }
  if (isNaN(page)) {
    page = 1;
  }

  return page;
}
