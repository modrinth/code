import { defineContentConfig, defineCollection, z } from "@nuxt/content";

export default defineContentConfig({
  collections: {
    news: defineCollection({
      type: "page",
      source: "news/article/*.md",
      schema: z.object({
        title: z.string(),
        short_title: z.string(),
        summary: z.string(),
        short_summary: z.string(),
        thumbnail: z.string(),
        date: z.date(),
      }),
    }),
  },
});
