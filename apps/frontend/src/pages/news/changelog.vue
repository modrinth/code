<script setup lang="ts">
import { type Product, getChangelog } from "@modrinth/utils";
import { ChangelogEntry } from "@modrinth/ui";
import NavTabs from "~/components/ui/NavTabs.vue";

const route = useRoute();

const filter = ref<Product | undefined>(undefined);

if (route.query.filter) {
  filter.value = route.query.filter as Product;
}

watch(
  () => route.query,
  () => {
    if (route.query.filter) {
      filter.value = route.query.filter as Product;
    } else {
      filter.value = undefined;
    }
  },
);

const changelogEntries = computed(() =>
  getChangelog().filter((x) => !filter.value || x.product === filter.value),
);
</script>

<template>
  <div class="page experimental-styles-within">
    <h1 class="m-0 text-3xl font-extrabold">Changelog</h1>
    <p class="my-3">Keep up-to-date on what's new with Modrinth.</p>
    <NavTabs
      :links="[
        {
          label: 'All',
          href: '',
        },
        {
          label: 'Website',
          href: 'web',
        },
        {
          label: 'Servers',
          href: 'servers',
        },
        {
          label: 'App',
          href: 'app',
        },
      ]"
      query="filter"
      class="mb-4"
    />
    <div class="relative flex flex-col gap-4 pb-6">
      <div class="absolute flex h-full w-4 justify-center">
        <div class="timeline-indicator" />
      </div>
      <ChangelogEntry
        v-for="(entry, index) in changelogEntries"
        :key="entry.date"
        :entry="entry"
        :first="index === 0"
        :show-type="filter === undefined"
        class="relative z-10"
      />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.page {
  padding: 0.5rem;
  margin-left: auto;
  margin-right: auto;
  max-width: 56rem;
}

.timeline-indicator {
  background-image: linear-gradient(
    to bottom,
    var(--color-raised-bg) 66%,
    rgba(255, 255, 255, 0) 0%
  );
  background-size: 100% 30px;
  background-repeat: repeat-y;
  margin-top: 1rem;

  height: calc(100% - 1rem);
  width: 4px;

  mask-image: linear-gradient(to bottom, black calc(100% - 15rem), transparent 100%);
}
</style>
