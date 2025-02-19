<template>
  <div
    aria-hidden="true"
    style="font-variant-numeric: tabular-nums"
    class="pointer-events-none h-full w-full select-none"
  >
    <div class="flex flex-col gap-6">
      <div class="flex flex-row items-center gap-6">
        <div
          class="relative max-h-[156px] min-h-[156px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="relative z-10 -ml-3 w-fit rounded-xl px-3 py-1">
            <div class="-mb-0.5 mt-0.5 flex flex-row items-center gap-2">
              <h2 class="m-0 -ml-0.5 text-3xl font-extrabold text-contrast">0.00%</h2>
              <h3 class="relative z-10 text-sm font-normal text-secondary">/ 100%</h3>
            </div>
            <h3 class="relative z-10 text-base font-normal text-secondary">CPU usage</h3>
          </div>
          <CPUIcon class="absolute right-10 top-10" />
        </div>
        <div
          class="relative max-h-[156px] min-h-[156px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="relative z-10 -ml-3 w-fit rounded-xl px-3 py-1">
            <div class="-mb-0.5 mt-0.5 flex flex-row items-center gap-2">
              <h2 class="m-0 -ml-0.5 text-3xl font-extrabold text-contrast">0.00%</h2>
              <h3 class="relative z-10 text-sm font-normal text-secondary">/ 100%</h3>
            </div>
            <h3 class="relative z-10 text-base font-normal text-secondary">Memory usage</h3>
          </div>
          <DBIcon class="absolute right-10 top-10" />
        </div>
        <div
          class="relative isolate min-h-[156px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8 transition-transform duration-100 hover:scale-105 active:scale-100"
        >
          <div class="flex flex-row items-center gap-2">
            <h2 class="m-0 -ml-0.5 mt-1 text-3xl font-extrabold text-contrast">0 B</h2>
          </div>
          <h3 class="relative z-10 text-base font-normal text-secondary">Storage usage</h3>
          <FolderOpenIcon class="absolute right-10 top-10 size-8" />
        </div>
      </div>

      <div
        class="relative flex h-full w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised p-8"
      >
        <div class="experimental-styles-within flex flex-row items-center">
          <div class="flex flex-row items-center gap-4">
            <h2 class="m-0 text-3xl font-extrabold text-contrast">Console</h2>
          </div>
        </div>
        <div class="relative w-full">
          <input type="text" placeholder="Search logs" class="h-12 !w-full !pl-10 !pr-48" />
          <SearchIcon class="absolute left-4 top-1/2 -translate-y-1/2" />
        </div>
        <div
          class="console relative h-full min-h-[516px] w-full overflow-hidden rounded-xl bg-bg text-sm"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CPUIcon, DBIcon, FolderOpenIcon, SearchIcon } from "@modrinth/assets";
</script>

<style scoped>
html.light-mode .console {
  background: var(--color-bg);
}

html.dark-mode .console {
  background: black;
}

html.oled-mode .console {
  background: black;
}
</style>
