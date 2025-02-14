<script setup lang="ts">
import { type Product, getChangelog } from "@modrinth/utils";
import { ChangelogEntry } from "@modrinth/ui";
import NavTabs from "~/components/ui/NavTabs.vue";

const route = useRoute();

const filter = ref<Product | undefined>(undefined);
const allChangelogEntries = ref(getChangelog());

function updateFilter() {
  if (route.query.filter) {
    filter.value = route.query.filter as Product;
  } else {
    filter.value = undefined;
  }
}

updateFilter();

watch(
  () => route.query,
  () => updateFilter(),
);

const changelogEntries = computed(() =>
  allChangelogEntries.value.filter((x) => !filter.value || x.product === filter.value),
);
</script>

<template>
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
      has-link
      class="relative z-[1]"
    />
  </div>
</template>

<style lang="scss" scoped>
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
