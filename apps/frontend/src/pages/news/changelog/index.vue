<script setup lang="ts">
import { type Product, getChangelog } from "@modrinth/utils";
import { ChangelogEntry } from "@modrinth/ui";
import Timeline from "@modrinth/ui/src/components/base/Timeline.vue";
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
  <Timeline fade-out-end>
    <ChangelogEntry
      v-for="(entry, index) in changelogEntries"
      :key="entry.date"
      :entry="entry"
      :first="index === 0"
      :show-type="filter === undefined"
      has-link
    />
  </Timeline>
</template>
