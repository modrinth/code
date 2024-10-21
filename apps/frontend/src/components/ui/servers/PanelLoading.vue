<template>
  <div
    aria-hidden="true"
    class="experimental-styles-within pointer-events-none mx-auto flex min-h-screen w-full max-w-[1280px] select-none flex-col gap-6 px-3"
  >
    <div class="flex flex-row items-center gap-6 pt-4">
      <div class="h-[6rem] w-[6rem] animate-pulse rounded-2xl bg-bg-raised"></div>
      <div class="flex flex-col gap-2">
        <div class="flex shrink-0 flex-row items-center gap-1">
          <div class="breadcrumb goto-link flex w-fit items-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14.414"
              height="12.162"
              viewBox="0 0 14.414 12.162"
            >
              <path
                fill="none"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M6.081 10.747 1.414 6.081m0 0 4.667-4.667M1.414 6.081h12"
              ></path>
            </svg>
            All servers
          </div>
        </div>
        <span class="h-[2.6rem] w-[200px] animate-pulse rounded-full bg-bg-raised"></span>
        <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
          <div class="flex flex-row items-center gap-2">
            <GameIcon aria-hidden="true" class="size-5 text-secondary" />
            <span class="text-sm font-semibold">Minecraft</span>
            <span class="h-4 w-[40px] rounded-full bg-bg-raised"></span>
          </div>
          <div class="flex flex-row items-center gap-4">
            <div class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
            <div class="flex animate-pulse flex-row items-center gap-2">
              <div class="h-4 w-4 rounded-full bg-bg-raised"></div>
              <span class="h-4 w-[100px] rounded-full bg-bg-raised"></span>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="flex flex-row items-center justify-between">
      <nav
        data-v-cc204322=""
        class="experimental-styles-within relative flex w-fit overflow-clip rounded-full bg-bg-raised p-1 text-sm font-bold"
      >
        <a
          data-v-cc204322=""
          aria-current="page"
          href="/servers/manage/d9cb74f4-931a-4b9b-bb31-3f02baa2d591"
          class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
          ><span data-v-cc204322="" class="text-nowrap">Overview</span></a
        ><a
          data-v-cc204322=""
          href="/servers/manage/d9cb74f4-931a-4b9b-bb31-3f02baa2d591/content"
          class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
          ><span data-v-cc204322="" class="text-nowrap">Content</span></a
        ><a
          data-v-cc204322=""
          href="/servers/manage/d9cb74f4-931a-4b9b-bb31-3f02baa2d591/files"
          class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
          ><span data-v-cc204322="" class="text-nowrap">Files</span></a
        ><a
          data-v-cc204322=""
          href="/servers/manage/d9cb74f4-931a-4b9b-bb31-3f02baa2d591/backups"
          class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
          ><span data-v-cc204322="" class="text-nowrap">Backups</span></a
        ><a
          data-v-cc204322=""
          href="/servers/manage/d9cb74f4-931a-4b9b-bb31-3f02baa2d591/options"
          class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
          ><span data-v-cc204322="" class="text-nowrap">Options</span></a
        >
      </nav>
    </div>
  </div>
</template>

<script setup lang="ts">
import { GameIcon } from "@modrinth/assets";
</script>
