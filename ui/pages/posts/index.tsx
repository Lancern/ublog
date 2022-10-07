import { useEffect, useState } from "react";

import Loading from "../../components/Loading";
import Paginator from "../../components/Paginator";
import PostList from "../../components/PostList";
import { getPosts } from "../../data/api";
import { Post } from "../../data/model";

export default function PostsPage(): JSX.Element {
  const [page, setPage] = useState<number>(1);
  const [posts, setPosts] = useState<Post[] | null>(null);

  useEffect(() => {
    setPosts(null);
    getPosts({ page, itemsPerPage: 10 }).then(setPosts);
  }, [page]);

  let postListEl: JSX.Element | null = null;
  if (posts === null) {
    postListEl = <Loading />;
  } else {
    postListEl = <PostList posts={posts} />;
  }

  return (
    <div className="my-8">
      {postListEl}
      <Paginator currentPage={page} maxPage={2} onPageChanged={setPage} />
    </div>
  );
}
