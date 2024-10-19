<template>
  <div aria-hidden="true" class="pointer-events-none h-full w-full select-none">
    <div class="flex flex-col gap-6">
      <div class="flex flex-row items-center gap-6">
        <div
          class="relative max-h-[150px] min-h-[150px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-row items-center gap-2">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">0.00%</h2>
          </div>
          <h3 class="relative z-10 text-base font-normal text-secondary">CPU usage</h3>
          <CPUIcon class="absolute right-8 top-8" />
        </div>
        <div
          class="relative max-h-[150px] min-h-[150px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-row items-center gap-2">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">0.00%</h2>
          </div>
          <h3 class="relative z-10 text-base font-normal text-secondary">Memory usage</h3>
          <DBIcon class="absolute right-8 top-8" />
        </div>
        <div
          class="relative max-h-[150px] min-h-[150px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-row items-center gap-2">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">0 MB</h2>
          </div>
          <h3 class="relative z-10 text-base font-normal text-secondary">Storage usage</h3>
          <FolderOpenIcon class="absolute right-8 top-8 size-8" />
        </div>
      </div>

      <div
        class="relative flex h-full w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised p-8"
      >
        <div class="experimental-styles-within flex flex-row items-center">
          <div class="flex flex-row items-center gap-4">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
          </div>
        </div>
        <div
          class="console relative h-full min-h-[488px] w-full overflow-hidden rounded-xl bg-bg text-sm"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CPUIcon, DBIcon, FolderOpenIcon } from "@modrinth/assets";
</script>
