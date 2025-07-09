<template>
  <div class="page experimental-styles-within">
    <h1 class="m-0 text-3xl font-extrabold">Changelog</h1>
    <p class="my-3">Keep up-to-date on what's new with Modrinth.</p>
    <NuxtPage />
  </div>
</template>

<script lang="ts" setup>
const config = useRuntimeConfig();

useSeoMeta({
  title: "Modrinth Changelog",
  ogTitle: "Modrinth Changelog",
  description: "Keep up-to-date on what's new with Modrinth.",
  ogDescription: "Keep up-to-date on what's new with Modrinth.",
  ogType: "website",
  ogImage: () => `${config.public.siteUrl}/news/changelog.webp`,
  twitterCard: "summary_large_image",
  twitterImage: () => `${config.public.siteUrl}/news/changelog.webp`,
});
</script>

<style lang="scss" scoped>
.page {
  padding: 1rem;
  margin-left: auto;
  margin-right: auto;
  max-width: 56rem;
}
</style>
