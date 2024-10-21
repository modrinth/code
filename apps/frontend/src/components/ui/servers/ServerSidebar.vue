<template>
  <div class="static w-full grid-cols-1 md:relative md:flex">
    <div class="static h-full flex-col pb-4 md:flex md:pb-0 md:pr-4">
      <div class="z-10 flex select-none flex-col gap-2 rounded-2xl bg-bg-raised p-4 md:w-[16rem]">
        <div v-for="link in navLinks" :key="link.label">
          <nuxt-link
            :to="link.href"
            class="flex items-center gap-2 rounded-xl p-2 hover:bg-button-bg"
            :class="{ 'bg-button-bg text-contrast': route.path === link.href }"
          >
            <div class="flex items-center gap-2 font-bold">
              <component :is="link.icon" class="h-6 w-6" />
              {{ link.label }}
            </div>
          </nuxt-link>
        </div>
      </div>
    </div>

    <div class="h-full w-full">
      <NuxtPage :route="props.route" :server="props.server" @reinstall="onReinstall" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { RouteLocationNormalized } from "vue-router";
import type { Server } from "~/composables/pyroServers";

const emit = defineEmits(["reinstall"]);

const props = defineProps<{
  navLinks: { label: string; href: string; icon: Component }[];
  route: RouteLocationNormalized;
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const onReinstall = (...args: any[]) => {
  emit("reinstall", ...args);
};
</script>
