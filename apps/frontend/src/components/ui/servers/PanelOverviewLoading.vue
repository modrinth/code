<template>
  <div aria-hidden="true" class="pointer-events-none h-full w-full select-none">
    <div class="flex flex-col gap-8">
      <div class="flex flex-row items-center gap-6">
        <div
          class="fade-in-up relative max-h-[150px] min-h-[150px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-row items-center gap-2">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">0.00%</h2>
          </div>
          <h3>CPU usage</h3>
          <CPUIcon class="absolute right-8 top-8" />
        </div>
        <div
          class="fade-in-up relative max-h-[150px] min-h-[150px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-row items-center gap-2">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">0.00%</h2>
          </div>
          <h3>Memory usage</h3>
          <DBIcon class="absolute right-8 top-8" />
        </div>
        <div
          class="fade-in-up relative max-h-[150px] min-h-[150px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-row items-center gap-2">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">0 MB</h2>
          </div>
          <h3>Storage usage</h3>
          <FolderOpenIcon class="absolute right-8 top-8 size-8" />
        </div>
      </div>

      <div
        class="fade-in-up relative flex h-full w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised p-8"
      >
        <div class="experimental-styles-within flex flex-row items-center justify-between">
          <div class="flex flex-row items-center gap-4">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-text-secondary)]">Console</h2>
            <div class="flex items-center gap-2 rounded-full bg-button-border px-2 py-1 opacity-0">
              <span class="text-sm font-semibold">Offline</span>
            </div>
          </div>
          <div class="btn btn-secondary color-accent-contrast opacity-0">
            <div class="contents">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                fill="none"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                viewBox="0 0 24 24"
              >
                <path d="m5 3 14 9-14 9z"></path>
              </svg>
              Start
            </div>
          </div>
        </div>
        <div
          class="console relative h-full min-h-[488px] w-full overflow-hidden rounded-xl bg-black text-sm"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CPUIcon, DBIcon, FolderOpenIcon } from "@modrinth/assets";
</script>

<style scoped>
@keyframes fade-in-up {
  from {
    opacity: 0;
    transform: translateY(40px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

.fade-in-up {
  animation: fade-in-up 400ms
    linear(
      0,
      0.006,
      0.025 2.8%,
      0.101 6.1%,
      0.539 18.9%,
      0.721 25.3%,
      0.849 31.5%,
      0.937 38.1%,
      0.968 41.8%,
      0.991 45.7%,
      1.006 50.1%,
      1.015 55%,
      1.017 63.9%,
      1.001
    );
}
</style>
