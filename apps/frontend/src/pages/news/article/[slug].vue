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
import { ref } from "vue";

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

const articleUrl = computed(() => `https://modrinth.com/news/article/${route.params.slug}`);

const dayjsDate = computed(() => dayjs(article.date));

const copied = ref(false);

async function copyToClipboard(text: string) {
  await navigator.clipboard.writeText(text);
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 3000);
}
</script>

<template>
  <div class="page experimental-styles-within">
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
      <div class="flex gap-2">
        <ButtonStyled circular>
          <a
            v-tooltip="`Share on Bluesky`"
            :href="`https://bsky.app/intent/compose?text=${encodeURIComponent(articleUrl)}`"
            target="_blank"
          >
            <BlueskyIcon />
          </a>
        </ButtonStyled>
        <ButtonStyled circular>
          <a
            v-tooltip="`Share on Mastodon`"
            :href="`https://tootpick.org/#text=${encodeURIComponent(articleUrl)}`"
            target="_blank"
          >
            <MastodonIcon />
          </a>
        </ButtonStyled>
        <ButtonStyled circular>
          <a
            v-tooltip="`Share on X`"
            :href="`https://www.x.com/intent/post?url=${encodeURIComponent(articleUrl)}`"
            target="_blank"
          >
            <TwitterIcon />
          </a>
        </ButtonStyled>
        <ButtonStyled circular>
          <a
            v-tooltip="`Share via email`"
            :href="`mailto:?subject=${encodeURIComponent(article.title)}&body=${encodeURIComponent(articleUrl)}`"
            target="_blank"
          >
            <MailIcon />
          </a>
        </ButtonStyled>
        <ButtonStyled circular>
          <button
            v-tooltip="copied ? `Copied to clipboard` : `Copy link`"
            :disabled="copied"
            class="relative grid place-items-center overflow-hidden"
            @click="copyToClipboard(articleUrl)"
          >
            <CheckIcon
              class="absolute transition-all ease-in-out"
              :class="copied ? 'translate-y-0' : 'translate-y-7'"
            />
            <LinkIcon
              class="absolute transition-all ease-in-out"
              :class="copied ? '-translate-y-7' : 'translate-y-0'"
            />
          </button>
        </ButtonStyled>
      </div>
      <img
        :src="article.thumbnail"
        class="aspect-video w-full rounded-2xl border-[1px] border-solid border-button-border object-cover"
        :alt="article.title"
      />
      <!--      <div v-html="renderHighlightedString(article.body)" />-->

      <ContentRenderer v-if="article" class="markdown-body" :value="article" />
      <div v-else>Article body could not be found.</div>
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
  > *:not(img, :has(img:first-child:last-child)) {
    margin-inline: 2rem;
  }

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
  h2,
  h3,
  h4,
  h5,
  h6 {
    a {
      font-weight: 800;
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
