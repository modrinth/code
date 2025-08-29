<template>
  <div class="static w-full grid-cols-1 md:relative md:flex">
    <div class="static h-full flex-col pb-4 md:flex md:pr-4 md:pb-0">
      <div class="bg-bg-raised z-10 flex flex-col gap-2 rounded-2xl p-4 select-none md:w-[16rem]">
        <div v-for="link in navLinks" :key="link.label">
          <NuxtLink
            :to="link.href"
            class="hover:bg-button-bg flex items-center gap-2 rounded-xl p-2"
            :class="{ 'bg-button-bg text-contrast': route.path === link.href }"
          >
            <div class="flex items-center gap-2 font-bold">
              <component :is="link.icon" class="size-6" />
              {{ link.label }}
            </div>

            <div class="grow" />
            <RightArrowIcon v-if="link.external" class="size-4" />
          </NuxtLink>
        </div>
      </div>
    </div>

    <div class="h-full w-full">
      <NuxtPage
        :route="route"
        :server="server"
        :backup-in-progress="backupInProgress"
        @reinstall="onReinstall"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { RightArrowIcon } from "@modrinth/assets";
import type { RouteLocationNormalized } from "vue-router";
import type { BackupInProgressReason } from "~/pages/servers/manage/[id].vue";
import { ModrinthServer } from "~/composables/servers/modrinth-servers.ts";

const emit = defineEmits(["reinstall"]);

defineProps<{
  navLinks: { label: string; href: string; icon: Component; external?: boolean }[];
  route: RouteLocationNormalized;
  server: ModrinthServer;
  backupInProgress?: BackupInProgressReason;
}>();

const onReinstall = (...args: any[]) => {
  emit("reinstall", ...args);
};
</script>
