<template>
  <NuxtLayout>
    <ModrinthLoadingIndicator />
    <Notifications />
    <NuxtPage />
  </NuxtLayout>
</template>
<script setup lang="ts">
import ModrinthLoadingIndicator from "~/components/ui/modrinth-loading-indicator.ts";
import Notifications from "~/components/ui/Notifications.vue";

onMounted(() =>
  new MutationObserver(() =>
    document.querySelectorAll("img").forEach((img) => {
      img.onload = () => {
        if (img.naturalWidth < 32 || img.naturalHeight < 32) img.style.imageRendering = "pixelated";
      };
    }),
  ).observe(document.body, { childList: true, subtree: true }),
);
</script>
