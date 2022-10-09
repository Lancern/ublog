import { GetStaticPropsResult } from "next";

import Document from "../components/Document";
import { getPost } from "../data/api";
import { Post } from "../data/model";

interface AboutPageProps {
  post: Post;
}

export default function AboutPage({ post }: AboutPageProps): JSX.Element {
  return (
    <div>
      <Document root={post.content} />
    </div>
  );
}

export async function getStaticProps(): Promise<GetStaticPropsResult<AboutPageProps>> {
  const post = await getPost("about");
  return {
    props: { post },
    revalidate: 60,
  };
}
