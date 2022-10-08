import { useEffect, useState } from "react";

import Loading from "../../components/Loading";
import Paginator from "../../components/Paginator";
import PostList from "../../components/PostList";
import { getPosts } from "../../data/api";
import { PaginatedList, Post } from "../../data/model";

export default function PostsPage(): JSX.Element {
  const [page, setPage] = useState<number>(1);
  const [posts, setPosts] = useState<PaginatedList<Post> | null>(null);

  const ITEMS_PER_PAGE = 10;

  useEffect(() => {
    setPosts(null);
    getPosts({ page, itemsPerPage: ITEMS_PER_PAGE }).then(setPosts);
  }, [page]);

  if (posts === null) {
    return (
      <div>
        <Loading />
      </div>
    );
  } else {
    const totalPages = Math.ceil(posts.totalCount / ITEMS_PER_PAGE);
    return (
      <div>
        <PostList posts={posts.objects} />
        <Paginator currentPage={page} maxPage={totalPages} onPageChanged={setPage} />
      </div>
    );
  }
}
