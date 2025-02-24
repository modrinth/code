<script setup lang="ts">
import { getChangelog } from "@modrinth/utils";
import { ChangelogEntry } from "@modrinth/ui";
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
    <div class="relative flex flex-col gap-4 pb-6">
      <div class="absolute flex h-full w-4 justify-center">
        <div class="timeline-indicator" :class="{ first: isFirst }" />
      </div>
      <ChangelogEntry :entry="changelogEntry" :first="isFirst" show-type class="relative z-[1]" />
    </div>
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

  height: calc(100% + 2rem);
  width: 4px;
  margin-top: -2rem;

  mask-image: linear-gradient(
    to bottom,
    transparent 0%,
    black 8rem,
    black calc(100% - 8rem),
    transparent 100%
  );

  &.first {
    margin-top: 1rem;

    mask-image: linear-gradient(black calc(100% - 15rem), transparent 100%);
  }
}
</style>
