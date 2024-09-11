<template>
  <div
    class="grid h-[50vh] w-full grid-cols-1 rounded-xl border border-solid border-bg-raised md:flex"
  >
    <div class="col-start-1 row-start-1 flex h-full flex-col md:relative">
      <div
        class="z-10 flex w-10 items-center gap-2 rounded-tl-xl text-lg font-bold text-contrast md:hidden"
        :class="{
          'w-[16rem] border-0 border-r-[1px] border-solid border-bg-raised bg-bg-raised':
            sidebarVisible,
        }"
      >
        <Button @click="toggleSidebar" class="h-10 w-10" transparent icon-only>
          <HamburgerIcon class="h-6 w-6 text-brand" />
        </Button>
      </div>

      <div
        class="z-10 flex h-[90%] w-[16rem] flex-col gap-2 rounded-t-none border-0 border-r border-solid border-bg-raised bg-[#1A1B1F] p-4 md:mt-0 md:h-full md:rounded-l-xl"
        :class="{
          'hidden md:flex': !sidebarVisible,
        }"
      >
        <div v-for="link in navLinks" :key="link.label">
          <nuxt-link
            :to="link.href"
            class="flex items-center gap-2 rounded-xl p-2 hover:bg-brand-highlight hover:text-brand"
            :class="{ 'bg-brand-highlight text-brand': route.path === link.href }"
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
      class="col-start-1 row-start-1 h-full w-full overflow-y-auto"
      :class="{ 'rounded-xl bg-black opacity-30 bg-blend-overlay': sidebarVisible && $mq == 'sm' }"
    >
      <NuxtPage :route="props.route" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { RouteLocationNormalized } from "vue-router";
import { Button } from "@modrinth/ui";
const props = defineProps<{
  navLinks: { label: string; href: string; icon: Component }[];
  route: RouteLocationNormalized;
}>();

import { HamburgerIcon } from "@modrinth/assets";

const sidebarVisible = ref(false);
const toggleSidebar = () => {
  sidebarVisible.value = !sidebarVisible.value;
};
</script>
