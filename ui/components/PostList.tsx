import Link from "next/link";
import { useEffect, useState } from "react";

import { Post } from "../data/model";

export interface PostListProps {
  posts: Post[];
}

export default function PostList({ posts }: PostListProps): JSX.Element {
  return (
    <div>
      {posts.map((post, idx) => (
        <div className="border-b border-b-slate-200 dark:border-b-slate-800 last:border-b-0" key={idx}>
          <PostListItem post={post} />
        </div>
      ))}
    </div>
  );
}

interface PostListItemProps {
  post: Post;
}

function PostListItem({ post }: PostListItemProps): JSX.Element {
  const [dateString, setDateString] = useState<string>("");

  useEffect(() => {
    // This code must be done in client-side since toLocaleDateString depends on the execution environment.
    const ts = post.updateTimestamp;
    const edited = post.createTimestamp !== post.updateTimestamp;

    let date = new Date(ts * 1000).toLocaleDateString();
    if (edited) {
      date += " (edited)";
    }

    setDateString(date);
  }, [post.createTimestamp, post.updateTimestamp]);

  const tagsString = post.tags.map((t) => `#${t}`).join(", ");
  const href = `/posts/${post.slug}`;
  return (
    <div className="py-8">
      <article>
        <Link href={href}>
          <h1 className="dark:text-white text-2xl font-bold mb-2 cursor-pointer">{post.title}</h1>
        </Link>
        <div className="text-gray-800 dark:text-gray-200 flex items-center my-2">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            strokeWidth="1.5"
            stroke="currentColor"
            className="w-6 h-6"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 012.25-2.25h13.5A2.25 2.25 0 0121 7.5v11.25m-18 0A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75m-18 0v-7.5A2.25 2.25 0 015.25 9h13.5A2.25 2.25 0 0121 11.25v7.5m-9-6h.008v.008H12v-.008zM12 15h.008v.008H12V15zm0 2.25h.008v.008H12v-.008zM9.75 15h.008v.008H9.75V15zm0 2.25h.008v.008H9.75v-.008zM7.5 15h.008v.008H7.5V15zm0 2.25h.008v.008H7.5v-.008zm6.75-4.5h.008v.008h-.008v-.008zm0 2.25h.008v.008h-.008V15zm0 2.25h.008v.008h-.008v-.008zm2.25-4.5h.008v.008H16.5v-.008zm0 2.25h.008v.008H16.5V15z"
            />
          </svg>
          <span className="block ml-1">{dateString}</span>
        </div>
        <div className="text-slate-500 my-2">
          {post.category} | {tagsString}
        </div>
        <Link href={href}>
          <button className="block dark:text-slate-200 mt-4 font-bold">Read more →</button>
        </Link>
      </article>
    </div>
  );
}
