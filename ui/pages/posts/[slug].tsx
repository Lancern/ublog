import { GetStaticPathsResult, GetStaticPropsContext, GetStaticPropsResult } from "next";
import { useRouter } from "next/router";
import { useRef } from "react";

import Document from "../../components/Document";
import Loading from "../../components/Loading";
import { TocContext, TocEntry, TocNav, TocScrollSpy } from "../../components/Toc";
import { getPost } from "../../data/api";
import { DocumentNode, Post } from "../../data/model";

interface PostPageProps {
  post: Post;
}

export default function PostPage({ post }: PostPageProps): JSX.Element {
  const router = useRouter();
  const tocCtxRef = useRef(new TocContext());

  if (!router.isReady || router.isFallback) {
    return (
      <div className="my-8">
        <Loading />
      </div>
    );
  }

  let dateString: string;
  if (post.createTimestamp === post.updateTimestamp) {
    dateString = new Date(post.createTimestamp * 1000).toLocaleDateString();
  } else {
    dateString = new Date(post.updateTimestamp * 1000).toLocaleDateString() + " (edited)";
  }

  const tocInfo = getDocumentTocInfo(post.content);
  return (
    <div className="dark:text-gray-200 selection:bg-gray-700 selection:text-white dark:selection:bg-gray-200 dark:selection:text-black">
      <header>
        <h1 className="font-bold text-4xl mb-8">{post.title}</h1>
        <div className="flex items-center text-slate-600 dark:text-slate-400 text-sm my-2">
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
        <div className="text-slate-600 dark:text-slate-400 text-sm my-2">
          {post.category} | {post.tags.map((tag) => "#" + tag).join(", ")}
        </div>
      </header>
      <hr className="my-8 dark:border-slate-700" />
      <div className="flex my-8 md:gap-x-6">
        <div className="md:w-3/4">
          <Document root={post.content} headingIdMap={tocInfo.headingIdMap} tocCtx={tocCtxRef} />
        </div>
        <div className="hidden md:block md:w-1/4">
          <div className="sticky top-28">
            <TocNav entries={tocInfo.entries} ctx={tocCtxRef} />
          </div>
        </div>
        <TocScrollSpy ctx={tocCtxRef} />
      </div>
    </div>
  );
}

export async function getStaticProps({ params }: GetStaticPropsContext): Promise<GetStaticPropsResult<PostPageProps>> {
  const slug = params.slug as string;
  const post = await getPost(slug);

  return {
    props: { post },
    revalidate: 60,
  };
}

export async function getStaticPaths(): Promise<GetStaticPathsResult> {
  return {
    paths: [],
    fallback: true,
  };
}

interface DocumentTocInfo {
  entries: TocEntry[];
  headingIdMap: Map<DocumentNode, string>;
}

function getDocumentTocInfo(root: DocumentNode): DocumentTocInfo {
  let nextHeadingCount = 0;
  const getDocumentTocInfoImpl = (root: DocumentNode, tocInfo: DocumentTocInfo) => {
    if (root.tag.type === "heading") {
      nextHeadingCount++;

      const headingTitle = renderPlaintext(root);
      const headingId = `${nextHeadingCount}-${encodeURIComponent(headingTitle)}`;
      tocInfo.headingIdMap.set(root, headingId);
      tocInfo.entries.push({
        title: headingTitle,
        level: root.tag.level,
        targetId: headingId,
      });
    }

    for (const child of root.children) {
      getDocumentTocInfoImpl(child, tocInfo);
    }
  };

  const tocInfo = {
    entries: [],
    headingIdMap: new Map(),
  };
  getDocumentTocInfoImpl(root, tocInfo);

  return tocInfo;
}

function renderPlaintext(root: DocumentNode): string {
  switch (root.tag.type) {
    case "inlineText":
      return root.tag.text;

    case "inlineCode":
      return root.tag.code;

    case "inlineEquation":
      return root.tag.expr;

    default:
      return root.children.map(renderPlaintext).join();
  }
}
