import getConfig from "next/config";
import Head from "next/head";
import Link from "next/link";
import { useEffect, useState } from "react";
import Loading from "../components/Loading";

import PostList from "../components/PostList";
import { getPosts } from "../data/api";
import { Post } from "../data/model";

export default function Home() {
  const [posts, setPosts] = useState<Post[] | null>(null);

  useEffect(() => {
    getPosts({ itemsPerPage: 5 }).then(setPosts);
  }, []);

  const profile = getConfig().publicRuntimeConfig.profile;

  let postListEl: JSX.Element;
  if (posts === null) {
    postListEl = <Loading />;
  } else {
    postListEl = <PostList posts={posts} />;
  }

  return (
    <>
      <Head>
        <title>{`${profile.nickname}'s Blog`}</title>
        <link rel="icon" href="/favicon.ico" />
      </Head>

      {postListEl}
      <div className="flex justify-center my-8">
        <Link href="/posts">
          <button className="block bg-gray-200 dark:bg-gray-800 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-700 transition rounded-lg font-bold px-4 py-2">
            More articles →
          </button>
        </Link>
      </div>
    </>
  );
}
