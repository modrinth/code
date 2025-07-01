<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import { ChevronRightIcon, RssIcon, GitGraphIcon } from "@modrinth/assets";
import dayjs from "dayjs";
import { articles as rawArticles } from "@modrinth/blog";
import { computed, ref } from "vue";
import NewsletterButton from "~/components/ui/NewsletterButton.vue";
import NewsArticleCard from "~/components/ui/news/NewsArticleCard.vue";

const articles = ref(
  rawArticles
    .map((article) => ({
      ...article,
      path: `/news/article/${article.slug}`,
      thumbnail: article.thumbnail
        ? `/news/article/${article.slug}/thumbnail.webp`
        : `/news/default.jpg`,
      title: article.title,
      summary: article.summary,
      date: article.date,
    }))
    .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime()),
);

const featuredArticle = computed(() => articles.value?.[0]);
const config = useRuntimeConfig();

useSeoMeta({
  title: "Modrinth News",
  ogTitle: "Modrinth News",
  description: "Keep up-to-date on the latest news from Modrinth.",
  ogDescription: "Keep up-to-date on the latest news from Modrinth.",
  ogType: "website",
  ogImage: () => `${config.public.siteUrl}/news/thumbnail.jpg`,
  twitterCard: "summary_large_image",
  twitterImage: () => `${config.public.siteUrl}/news/thumbnail.jpg`,
});
</script>

<template>
  <div class="page experimental-styles-within py-6">
    <div class="flex flex-wrap items-center justify-between gap-4 px-6">
      <div>
        <h1 class="m-0 text-3xl font-extrabold">News</h1>
      </div>
      <div class="flex gap-2">
        <NewsletterButton />
        <ButtonStyled circular>
          <a v-tooltip="`RSS feed`" aria-label="RSS feed" href="/news/feed/rss.xml" target="_blank">
            <RssIcon />
          </a>
        </ButtonStyled>
        <ButtonStyled circular icon-only>
          <a v-tooltip="`Changelog`" href="/news/changelog" aria-label="Changelog">
            <GitGraphIcon />
          </a>
        </ButtonStyled>
      </div>
    </div>

    <template v-if="articles && articles.length">
      <div
        v-if="featuredArticle"
        class="full-width-bg brand-gradient-bg mt-6 border-0 border-y-[1px] border-solid py-4"
      >
        <nuxt-link
          :to="`${featuredArticle.path}/`"
          class="active:scale-[0.99]! group flex transition-all ease-in-out hover:brightness-125"
        >
          <article class="featured-article px-6">
            <div class="featured-image-container">
              <img
                :src="featuredArticle.thumbnail"
                class="aspect-video w-full rounded-2xl border-[1px] border-solid border-button-border object-cover"
              />
            </div>
            <div class="featured-content">
              <p class="m-0 font-bold">Featured article</p>
              <h3 class="m-0 text-3xl leading-tight group-hover:underline">
                {{ featuredArticle?.title }}
              </h3>
              <p class="m-0 text-lg leading-tight">{{ featuredArticle?.summary }}</p>
              <div class="mt-auto text-secondary">
                {{ dayjs(featuredArticle?.date).format("MMMM D, YYYY") }}
              </div>
            </div>
          </article>
        </nuxt-link>
      </div>

      <div class="mt-6 px-6">
        <div class="group flex w-fit items-center gap-1">
          <h2 class="m-0 text-xl font-extrabold">More articles</h2>
          <ChevronRightIcon
            v-if="false"
            class="ml-0 h-6 w-6 transition-all group-hover:ml-1 group-hover:text-brand"
          />
        </div>

        <div class="mt-4 grid grid-cols-[repeat(auto-fill,minmax(250px,1fr))] gap-4">
          <NewsArticleCard
            v-for="article in articles.slice(1)"
            :key="article.path"
            :article="article"
          />
        </div>
      </div>
    </template>

    <div v-else class="pt-4">Error: Articles could not be loaded.</div>
  </div>
</template>

<style lang="scss" scoped>
.page {
  > *:not(.full-width-bg),
  > .full-width-bg > * {
    max-width: 56rem;
    margin-inline: auto;
  }
}

.brand-gradient-bg {
  background: var(--brand-gradient-bg);
  border-color: var(--brand-gradient-border);
}

.featured-article {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  width: 100%;
}

.featured-image-container {
  flex: 1;
  min-width: 0;
}

.featured-content {
  flex: 1;
  min-width: 16rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

@media (max-width: 640px) {
  .featured-article {
    flex-direction: column;
  }

  .featured-image-container {
    order: 1;
  }

  .featured-content {
    order: 2;
    min-width: 0;
  }
}
</style>
