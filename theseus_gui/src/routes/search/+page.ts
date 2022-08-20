import type { PageLoad } from './$types';

/** @type {import('./index').PageLoad} */
export async function load({ fetch }): PageLoad {
  const response = await fetch(`https://api.modrinth.com/v2/search?query=&limit=10&offset=0&index=relevance`);

  return {
    projects: response.ok && (await response.json()).hits
  };
}
