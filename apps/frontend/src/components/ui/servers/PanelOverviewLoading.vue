<template>
  <div class="h-full w-full">
    <div class="flex flex-col gap-6">
      <div class="flex flex-row items-center gap-6">
        <div class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-bg-raised"></div>
        <div class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-bg-raised"></div>
        <div class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-bg-raised"></div>
      </div>

      <div
        class="relative flex h-full w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised p-8"
      >
        <div class="experimental-styles-within flex flex-row items-center justify-between">
          <div class="flex flex-row items-center gap-4">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Console</h2>
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
          class="monocraft-font console relative h-full min-h-[288px] w-full overflow-hidden rounded-xl bg-black text-sm"
        ></div>
      </div>
    </div>
  </div>
</template>
