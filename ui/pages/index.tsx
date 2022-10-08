import { GetStaticPropsResult } from "next";
import getConfig from "next/config";
import Head from "next/head";
import Link from "next/link";

import PostList from "../components/PostList";
import { getPosts } from "../data/api";
import { Post } from "../data/model";

interface HomeProps {
  posts: Post[];
}

export default function Home({ posts }: HomeProps) {
  const owner = getConfig().publicRuntimeConfig.owner;
  return (
    <>
      <Head>
        <title>{`${owner}'s Blog`}</title>
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <div>
        <PostList posts={posts} />
        <div className="flex justify-center my-8">
          <Link href="/posts">
            <button className="block bg-gray-200 dark:bg-gray-800 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-700 transition rounded-lg font-bold px-4 py-2">
              More articles →
            </button>
          </Link>
        </div>
      </div>
    </>
  );
}

export async function getStaticProps(): Promise<GetStaticPropsResult<HomeProps>> {
  const posts = await getPosts({ itemsPerPage: 5 });
  return {
    props: {
      posts: posts.objects,
    },
    revalidate: 60,
  };
}
