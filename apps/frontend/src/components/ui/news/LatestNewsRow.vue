<template>
  <div class="mx-2 p-4 !py-8 shadow-md sm:mx-8 sm:p-32">
    <div class="my-8 flex items-center justify-between">
      <h2 class="m-0 mx-auto text-3xl font-extrabold sm:text-4xl">Latest news from Modrinth</h2>
    </div>

    <div v-if="latestArticles" class="grid grid-cols-[repeat(auto-fit,minmax(250px,1fr))] gap-4">
      <div
        v-for="(article, index) in latestArticles"
        :key="article.slug"
        :class="{ 'max-xl:hidden': index === 2 }"
      >
        <NewsArticleCard :article="article" />
      </div>
    </div>
    <div class="mx-2 my-8 flex w-full items-center justify-center">
      <ButtonStyled color="brand" size="large">
        <nuxt-link to="/news">
          <NewspaperIcon />
          View all news
        </nuxt-link>
      </ButtonStyled>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NewspaperIcon } from "@modrinth/assets";
import { articles as rawArticles } from "@modrinth/blog";
import { ButtonStyled } from "@modrinth/ui";
import { ref, computed } from "vue";
import NewsArticleCard from "./NewsArticleCard.vue";

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

const latestArticles = computed(() => articles.value.slice(0, 3));
</script>
