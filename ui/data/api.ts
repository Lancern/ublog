import getConfig from "next/config";

import { Post } from "./model";

export interface Pagination {
  page?: number;
  itemsPerPage?: number;
}

const DEFAULT_PAGE = 1;
const DEFAULT_ITEMS_PER_PAGE = 20;

export async function getPosts(pagination?: Pagination): Promise<Post[]> {
  const page = pagination?.page ?? DEFAULT_PAGE;
  const items = pagination?.itemsPerPage ?? DEFAULT_ITEMS_PER_PAGE;

  const response = await getApi("/api/posts", { page, items });
  return await response.json();
}

export async function getPost(slug: string): Promise<Post | null> {
  const response = await getApi(`/api/posts/${slug}`);
  if (response.status === 404) {
    return null;
  }

  return await response.json();
}

export function getResourceUrl(uuid: string): URL {
  return getApiUrl(`/api/resources/${uuid}`);
}

async function getApi(path: string, queries?: object): Promise<Response> {
  const url = getApiUrl(path, queries);
  return await fetch(url, { method: "GET" });
}

function getApiUrl(path: string, queries?: object): URL {
  const { dataServerUrl, uiServerUrl } = getConfig().publicRuntimeConfig;
  const serverUrl: string = dataServerUrl ?? uiServerUrl;

  const url = new URL(path, serverUrl);

  if (queries !== undefined) {
    for (const queryKey of Object.getOwnPropertyNames(queries)) {
      const queryValue = queries[queryKey];
      url.searchParams.set(queryKey, `${queryValue}`);
    }
  }

  return url;
}
