<template>
  <div class="ad-parent relative mb-3 flex w-full justify-center rounded-2xl bg-bg-raised">
    <div class="flex max-h-[250px] min-h-[250px] min-w-[300px] max-w-[300px] flex-col gap-4 p-6">
      <p class="m-0 text-2xl font-bold text-contrast">75% of ad revenue goes to creators</p>
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
      src: "https://bservr.com/o.js?uid=8118d1fdb2e0d6f32180bd27",
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
  color: var(--color-base);
}

#qc-cmp2-ui::before {
  background: var(--color-raised-bg);
}

#qc-cmp2-ui::after {
  background: var(--color-raised-bg);
}

#qc-cmp2-ui button[mode="primary"] {
  background: var(--color-brand);
  color: var(--color-accent-contrast);
  border-radius: var(--radius-lg);
  border: none;
}

#qc-cmp2-ui button[mode="secondary"] {
  background: var(--color-button-bg);
  color: var(--color-base);
  border-radius: var(--radius-lg);
  border: none;
}

#qc-cmp2-ui button[mode="link"] {
  color: var(--color-link);
}

#qc-cmp2-ui h2 {
  color: var(--color-contrast);
  font-size: 1.5rem;
}

#qc-cmp2-ui div,
#qc-cmp2-ui li,
#qc-cmp2-ui strong,
#qc-cmp2-ui p,
#qc-cmp2-ui .qc-cmp2-list-item-title,
#qc-cmp2-ui .qc-cmp2-expandable-info {
  color: var(--color-base);
  font-family: var(--font-standard);
}

#qc-cmp2-ui .qc-cmp2-toggle[aria-checked="true"] {
  background-color: var(--color-brand);
  border: 1px solid var(--color-brand);
}

@media (max-width: 1024px) {
  .ad-parent {
    display: none;
  }
}
</style>
