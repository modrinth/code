<template>
  <div class="mb-3 flex rounded-2xl">
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
  </div>
</template>
<script setup lang="ts">
import { ChevronRightIcon } from "@modrinth/assets";
</script>
