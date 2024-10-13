<template>
  <div class="relative w-full grid-cols-1 md:flex">
    <div class="absolute flex h-full flex-col md:relative">
      <div
        class="z-10 flex w-10 items-center gap-2 rounded-tl-xl text-lg font-bold text-contrast md:hidden"
        :class="{
          'w-[16rem] border-0 border-r-[1px] border-solid border-bg-raised bg-bg-raised':
            sidebarVisible,
        }"
      >
        <Button class="h-[5%] w-10" transparent icon-only @click="toggleSidebar">
          <HamburgerIcon class="h-6 w-6 text-brand" />
        </Button>
      </div>

      <div
        class="z-10 flex w-[16rem] select-none flex-col gap-2 rounded-xl bg-bg-raised p-4 md:mt-0 md:h-full md:rounded-l-xl"
        :class="{
          'hidden md:flex': !sidebarVisible,
        }"
      >
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

    <div
      class="h-full w-full"
      :class="{ 'rounded-xl bg-black opacity-30 bg-blend-overlay': sidebarVisible && $mq == 'sm' }"
    >
      <NuxtPage :route="props.route" :server="props.server" @reinstall="onReinstall" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { RouteLocationNormalized } from "vue-router";
import { Button } from "@modrinth/ui";
import { HamburgerIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const emit = defineEmits(["reinstall"]);

const props = defineProps<{
  navLinks: { label: string; href: string; icon: Component }[];
  route: RouteLocationNormalized;
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const onReinstall = () => {
  emit("reinstall");
};

const sidebarVisible = ref(false);
const toggleSidebar = () => {
  sidebarVisible.value = !sidebarVisible.value;
};
</script>
