import { Feed } from "feed";
import dayjs from "dayjs";

function findSlug(path: string) {
  return path.match(/[^/]+$/)?.[0];
}

export default defineEventHandler(async (event) => {
  const baseUrl = useRuntimeConfig().public.siteUrl;

  const articles = await queryCollection(event, "news").order("date", "DESC").all();

  if (!articles) {
    setResponseStatus(event, 404);
    return `News articles could not be found.`;
  }

  const latestDate = dayjs(articles?.[0].date);

  const author = {
    name: "Modrinth Team",
    link: baseUrl,
  };

  const feed = new Feed({
    title: "Modrinth News",
    description: "Keep up-to-date on the latest news from Modrinth.",
    id: baseUrl,
    link: `${baseUrl}/news`,
    language: "en",
    image: `${baseUrl}/news/thumbnail.jpg`,
    favicon: `${baseUrl}/favicon.ico`,
    copyright: `© ${latestDate.year()} Rinth, Inc.`,
    updated: latestDate.toDate(),
    feedLinks: {
      rss: `${baseUrl}/news/feed/rss`,
      json: `${baseUrl}/news/feed/json`,
      atom: `${baseUrl}/news/feed/atom`,
    },
    author,
  });

  articles?.forEach((article) => {
    feed.addItem({
      title: article.title,
      id: findSlug(article.path),
      link: `${baseUrl}${article.path}/`,
      description: article.summary,
      author: [author],
      date: dayjs(article.date).toDate(),
      image: `${baseUrl}${article.path}/${article.thumbnail}`,
    });
  });

  switch (event?.context?.params?.feed_type.toLowerCase()) {
    case "rss":
      setResponseHeader(event, "Content-Type", "application/rss+xml; charset=utf-8");
      return feed.rss2();
    case "atom":
      setResponseHeader(event, "Content-Type", "application/atom+xml; charset=utf-8");
      return feed.atom1();
    case "json":
      setResponseHeader(event, "Content-Type", "application/feed+json; charset=utf-8");
      return feed.json1();
    default:
      setResponseStatus(event, 400);
      return `'${event?.context?.params?.feed_type}' is an invalid feed type, only 'rss', 'atom' and 'json' are supported`;
  }
});
