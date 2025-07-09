<script setup lang="ts">
import { getChangelog } from "@modrinth/utils";
import { ChangelogEntry, Timeline } from "@modrinth/ui";
import { ChevronLeftIcon } from "@modrinth/assets";

const route = useRoute();

const changelogEntry = computed(() =>
  route.params.date
    ? getChangelog().find((x) => {
        if (x.product === route.params.product) {
          console.log("Found matching product!");

          if (x.version && x.version === route.params.date) {
            console.log("Found matching version!");
            return x;
          } else if (x.date.unix() === Number(route.params.date as string)) {
            console.log("Found matching date!");
            return x;
          }
        }
        return undefined;
      })
    : undefined,
);

const isFirst = computed(() => changelogEntry.value?.date === getChangelog()[0].date);

if (!changelogEntry.value) {
  createError({ statusCode: 404, statusMessage: "Version not found" });
}
</script>

<template>
  <div v-if="changelogEntry">
    <nuxt-link
      :to="`/news/changelog?filter=${changelogEntry.product}`"
      class="mb-4 mt-4 flex w-fit items-center gap-2 text-link"
    >
      <ChevronLeftIcon /> View full changelog
    </nuxt-link>
    <Timeline fade-out-end :fade-out-start="!isFirst">
      <ChangelogEntry :entry="changelogEntry" :first="isFirst" show-type />
    </Timeline>
  </div>
</template>
