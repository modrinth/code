<template>
  <div class="static w-full grid-cols-1 md:relative md:flex">
    <div class="static h-full flex-col pb-4 md:flex md:pb-0 md:pr-4">
      <div
        class="z-10 select-none rounded-2xl bg-bg-raised p-4 shadow-lg transition-all duration-300 ease-in-out hover:shadow-xl md:w-[16rem]"
      >
        <transition-group name="nav-item" tag="div" class="flex flex-col gap-2">
          <div v-for="link in navLinks" :key="link.label">
            <NuxtLink
              :to="link.href"
              class="flex items-center gap-2 rounded-xl p-2 transition-all duration-300 ease-in-out hover:bg-button-bg"
              :class="{ 'bg-button-bg text-contrast': route.path === link.href }"
            >
              <div class="flex items-center gap-2 font-bold">
                <component
                  :is="link.icon"
                  class="h-6 w-6 transition-transform duration-300 ease-in-out"
                  :class="{ 'rotate-6': route.path === link.href }"
                />
                <span
                  class="transition-all duration-300 ease-in-out"
                  :class="{ 'translate-x-1': route.path === link.href }"
                >
                  {{ link.label }}
                </span>
              </div>
            </NuxtLink>
          </div>
        </transition-group>
      </div>
    </div>

    <div class="h-full w-full">
      <Suspense>
        <NuxtPage
          :key="route.path"
          :route="props.route"
          :server="props.server"
          @reinstall="onReinstall"
        />
      </Suspense>
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

<style scoped>
.nav-item-enter-active,
.nav-item-leave-active {
  transition: all 0.5s ease;
}
.nav-item-enter-from,
.nav-item-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

@keyframes pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(255, 255, 255, 0.4);
  }
  70% {
    box-shadow: 0 0 0 10px rgba(255, 255, 255, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(255, 255, 255, 0);
  }
}

.animate-pulse {
  animation: pulse 2s infinite;
}
</style>
