<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import { ChevronRightIcon, RssIcon, NewspaperIcon } from "@modrinth/assets";
import dayjs from "dayjs";

const articles = [
  {
    title: "Host your own server with Modrinth Servers — now in beta",
    short_title: "Introducing Modrinth Servers",
    summary: "Fast, simple, reliable servers directly integrated into Modrinth.",
    short_summary: "Host your next Minecraft server with Modrinth.",
    thumbnail:
      "https://media.beehiiv.com/cdn-cgi/image/format=auto,width=800,height=421,fit=scale-down,onerror=redirect/uploads/asset/file/eefddc59-b4c4-4e7d-92e8-c26bdef42984/Modrinth-Servers-Thumb.png",
    date: dayjs("2024-11-02T22:00:00-08:00"),
    slug: "modrinth-servers-beta",
  },
  {
    title: "Quintupling Creator Revenue and Becoming Sustainable",
    short_title: "TODO",
    summary: "Announcing an update to our monetization program, creator split, and more!",
    short_summary: "TODO",
    thumbnail:
      "https://media.beehiiv.com/cdn-cgi/image/fit=scale-down,format=auto,onerror=redirect,quality=80/uploads/asset/file/c99b9885-8248-4d7a-b19a-3ae2c902fdd5/revenue.png",
    date: dayjs("2024-09-13T12:00:00-08:00"),
    slug: "creator-revenue-update",
  },
  {
    title: "Introducing Modrinth+, a new look, and a new ad system",
    short_title: "TODO",
    summary: "Learn about this major update to Modrinth.",
    short_summary: "TODO",
    thumbnail:
      "https://media.beehiiv.com/cdn-cgi/image/fit=scale-down,format=auto,onerror=redirect,quality=80/uploads/asset/file/38ce85e4-5d93-43eb-b61b-b6296f6b9e66/things.png",
    date: dayjs("2024-08-21T12:00:00-08:00"),
    slug: "introducing-modrinth-refreshed-site-look-new-advertising-system",
  },
  {
    title: `Malware Discovery Disclosure: "Windows Borderless" mod`,
    short_title: "TODO",
    summary: "Threat Analysis and Plan of Action",
    short_summary: "TODO",
    thumbnail:
      "https://media.beehiiv.com/cdn-cgi/image/fit=scale-down,format=auto,onerror=redirect,quality=80/uploads/asset/file/5417dc7d-a542-4e43-bac7-c6b2ef9638ab/windows-borderless.png",
    date: dayjs("2024-05-07T12:00:00-08:00"),
    slug: "windows-borderless-malware-disclosure",
  },
];

const featuredArticle = articles[0];
</script>

<template>
  <div class="page experimental-styles-within">
    <div class="flex items-center justify-between gap-6">
      <div>
        <h1 class="m-0 text-3xl font-extrabold">News</h1>
      </div>
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
    <div class="full-width-bg brand-gradient-bg mt-6 border-0 border-y-[1px] border-solid py-4">
      <nuxt-link
        :to="`/news/article/${featuredArticle.slug}`"
        class="active:scale-[0.99]! group flex transition-all ease-in-out hover:brightness-125"
      >
        <article class="grid grid-cols-[4fr_3fr] gap-4">
          <img
            :src="featuredArticle.thumbnail"
            class="aspect-video w-full rounded-2xl border-[1px] border-solid border-button-border object-cover"
          />
          <div class="flex flex-col gap-2">
            <p class="m-0 font-bold">Featured article</p>
            <h3 class="m-0 text-3xl leading-tight group-hover:underline">
              {{ featuredArticle.title }}
            </h3>
            <p class="m-0 text-lg leading-tight">{{ featuredArticle.summary }}</p>
            <div class="mt-auto text-secondary">
              {{ featuredArticle.date.format("MMMM D, YYYY") }}
            </div>
          </div>
        </article>
      </nuxt-link>
    </div>
    <div class="mt-6">
      <nuxt-link class="flex w-fit items-center gap-1" to="/news/articles">
        <h2 class="m-0 text-xl font-extrabold">More articles</h2>
        <ChevronRightIcon />
      </nuxt-link>
      <div class="mt-4 grid grid-cols-3 gap-4">
        <nuxt-link
          v-for="article in articles.slice(1)"
          :key="`post-${article.slug}`"
          :to="`/news/article/${article.slug}`"
          class="active:scale-[0.99]! group flex flex-col gap-2 transition-all ease-in-out hover:brightness-125"
        >
          <article class="flex grow flex-col gap-4">
            <img
              :src="article.thumbnail"
              class="aspect-video w-full rounded-xl border-[1px] border-solid border-button-border object-cover"
            />
            <div class="flex grow flex-col gap-2">
              <h3 class="m-0 text-base leading-tight group-hover:underline">{{ article.title }}</h3>
              <p v-if="article.summary" class="m-0 text-sm leading-tight">{{ article.summary }}</p>
              <div class="mt-auto text-sm text-secondary">
                {{ article.date.format("MMMM D, YYYY") }}
              </div>
            </div>
          </article>
        </nuxt-link>
      </div>
    </div>
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
</style>
