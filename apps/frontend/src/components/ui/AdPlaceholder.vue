<template>
  <div
    v-if="!auth.user || isPermission(auth.user.badges, 1 << 0)"
    class="relative mb-3 flex rounded-2xl"
  >
    <div class="flex min-h-[250px] min-w-[300px] flex-col gap-4 rounded-2xl bg-bg-raised p-6">
      <p class="m-0 text-xl font-bold text-contrast">90% of ad revenue goes to creators</p>
      <nuxt-link to="/plus" class="mt-auto flex items-center gap-1 text-purple hover:underline">
        <span>
          Go ad-free with
          <span class="font-bold"><span class="text-contrast">Modrinth</span>+</span>
        </span>
        <ChevronRightIcon class="h-5 w-5" />
      </nuxt-link>
    </div>
    <div
      class="absolute top-0 flex items-center justify-center overflow-hidden rounded-2xl bg-bg-raised"
    >
      <div id="modrinth-rail-1" />
    </div>
  </div>
</template>
<script setup>
import { ChevronRightIcon } from "@modrinth/assets";

useHead({
  script: [
    {
      src: "https://dn0qt3r0xannq.cloudfront.net/modrinth-7JfmkEIXEp/modrinth-longform/prebid-load.js",
      async: true,
    },
    {
      src: "/inmobi.js",
      async: true,
    },
  ],
  link: [
    {
      rel: "preload",
      as: "script",
      href: "https://www.googletagservices.com/tag/js/gpt.js",
    },
  ],
});

const auth = await useAuth();

onMounted(() => {
  window.tude = window.tude || { cmd: [] };
  tude.cmd.push(function () {
    tude.refreshAdsViaDivMappings([
      {
        divId: "modrinth-rail-1",
        baseDivId: "pb-slot-square-2",
      },
    ]);
  });
});
</script>
<style>
iframe[id^="google_ads_iframe"] {
  color-scheme: normal;
  background: transparent;
}

#qc-cmp2-ui {
  background: var(--color-raised-bg);
  border-radius: var(--radius-lg);
}

#qc-cmp2-ui button[mode="primary"] {
  background: var(--color-brand);
  color: var(--color-accent-contrast);
  border-radius: var(--radius-lg);
}

#qc-cmp2-ui button[mode="secondary"] {
  background: var(--color-button-bg);
  color: var(--color-base);
  border-radius: var(--radius-lg);
}

#qc-cmp2-ui h2 {
  color: var(--color-contrast);
  font-size: 1.5rem;
}

#qc-cmp2-ui p {
  font-family: var(--font-standard);
  font-size: 1rem;
}
</style>
