<template>
  <div class="flex h-[50vh] w-full rounded-xl border border-solid border-bg-raised">
    <div
      class="flex h-full w-[15rem] flex-col gap-2 rounded-l-xl border-0 border-r border-solid border-bg-raised bg-[#1A1B1F] p-4"
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
    <div class="h-full w-full overflow-y-auto">
      <NuxtPage :route="route" />
    </div>
  </div>
</template>

<script lang="ts">
import type { RouteLocationNormalized } from "vue-router";

export default defineComponent({
  name: "ServerSidebar",
  props: {
    navLinks: {
      type: Array as PropType<{ label: string; href: string; icon: Component }[]>,
      required: true,
    },
    route: {
      type: Object as PropType<RouteLocationNormalized>,
      required: true,
    },
  },
});
</script>
