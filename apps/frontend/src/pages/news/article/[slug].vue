<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import {
  CheckIcon,
  RssIcon,
  NewspaperIcon,
  BlueskyIcon,
  TwitterIcon,
  MastodonIcon,
  LinkIcon,
  MailIcon,
} from "@modrinth/assets";
import dayjs from "dayjs";
import ShareArticleButtons from "~/components/ui/ShareArticleButtons.vue";

const route = useRoute();
const { data: article } = await useAsyncData(route.path, () => {
  return queryCollection("news").path(`/news/article/${route.params.slug}`).first();
});

if (!article) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: "The article requested could not be found",
  });
}

const articleTitle = computed(() => article.value?.title);
const articleUrl = computed(() => `https://modrinth.com/news/article/${route.params.slug}`);

const dayjsDate = computed(() => dayjs(article.value?.date));

const config = useRuntimeConfig();

useSeoMeta({
  title: () => `${articleTitle.value} - Modrinth News`,
  ogTitle: () => articleTitle.value,
  description: () => article.value?.summary,
  ogDescription: () => article.value?.summary,
  ogType: "article",
  ogImage: () => `${config.public.siteUrl}${article.value?.path}/${article.value?.thumbnail}`,
  articlePublishedTime: () => dayjsDate.value.toISOString(),
  twitterCard: "summary_large_image",
  twitterImage: () => `${config.public.siteUrl}/news/thumbnail.jpg`,
});
</script>

<template>
  <div v-if="article" class="page experimental-styles-within">
    <div
      class="flex items-center justify-between gap-6 border-0 border-b-[1px] border-solid border-divider pb-6"
    >
      <nuxt-link :to="`/news`">
        <h1 class="m-0 text-3xl font-extrabold hover:underline">News</h1>
      </nuxt-link>
      <div class="flex gap-2">
        <ButtonStyled color="brand" type="outlined">
          <button><NewspaperIcon /> Sign up for our newsletter</button>
        </ButtonStyled>
        <ButtonStyled circular>
          <button v-tooltip="`RSS feed`" aria-label="RSS feed">
            <RssIcon />
          </button>
        </ButtonStyled>
      </div>
    </div>
    <article class="mt-6 flex flex-col gap-4">
      <h2 class="m-0 text-4xl font-extrabold">{{ article.title }}</h2>
      <p class="m-0 text-lg leading-tight">{{ article.summary }}</p>
      <div class="mt-auto text-secondary">Posted on {{ dayjsDate.format("MMMM D, YYYY") }}</div>
      <ShareArticleButtons :title="article.title" :url="articleUrl" />
      <img
        :src="article.thumbnail"
        class="aspect-video w-full rounded-2xl border-[1px] border-solid border-button-border object-cover"
        :alt="article.title"
      />
      <ContentRenderer class="markdown-body" :value="article" />
      <h3
        class="mb-0 mt-4 border-0 border-t-[1px] border-solid border-divider pt-4 text-lg font-extrabold"
      >
        Share this article
      </h3>
      <ShareArticleButtons :title="article.title" :url="articleUrl" />
    </article>
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

:deep(.markdown-body) {
  h1,
  h2 {
    border-bottom: none;
    padding: 0;
  }

  ul > li:not(:last-child) {
    margin-bottom: 0.5rem;
  }

  ul {
    strong {
      color: var(--color-contrast);
      font-weight: 600;
    }
  }

  h1,
  h2,
  h3 {
    margin-bottom: 0.25rem;
  }

  p {
    margin-bottom: 1.25rem;
  }

  a {
    color: var(--color-brand);
    font-weight: 600;

    &:hover {
      text-decoration: underline;
    }
  }

  h1,
  h2 {
    a {
      font-weight: 800;
    }
  }

  h1,
  h2,
  h3,
  h4,
  h5,
  h6 {
    a {
      color: var(--color-contrast);
    }
  }

  img {
    border: 1px solid var(--color-button-border);
    border-radius: var(--radius-lg);
  }

  > img,
  > :has(img:first-child:last-child) {
    display: flex;
    justify-content: center;
  }
}
</style>
