<template>
  <div class="ad-parent relative mb-3 flex w-full justify-center rounded-2xl bg-bg-raised">
    <div class="flex max-h-[250px] min-h-[250px] min-w-[300px] max-w-[300px] flex-col gap-4 p-6">
      <p class="m-0 text-2xl font-bold text-contrast">90% of ad revenue goes to creators</p>
      <nuxt-link to="/plus" class="mt-auto items-center gap-1 text-purple hover:underline">
        <span>
          Support creators and Modrinth ad-free with
          <span class="font-bold">Modrinth+</span>
        </span>
        <ChevronRightIcon class="relative top-[3px] h-5 w-5" />
      </nuxt-link>
    </div>
    <div
      class="absolute top-0 flex items-center justify-center overflow-hidden rounded-2xl bg-bg-raised"
    >
      <div data-ad="project-rail" />
    </div>
  </div>
</template>
<script setup>
import { ChevronRightIcon } from "@modrinth/assets";

useHead({
  script: [
    {
      type: "module",
      src: "//js.rev.iq",
      "data-domain": "modrinth.com",
      async: true,
    },
  ],
});
</script>
<style>
iframe[id^="google_ads_iframe"] {
  color-scheme: normal;
  background: transparent;
}

@media (max-width: 1024px) {
  .ad-parent {
    display: none;
  }
}
</style>
