<template>
  <div
    data-pyro-server-list-root
    class="experimental-styles-within relative mx-auto flex min-h-screen w-full max-w-[1280px] flex-col px-4 sm:px-6"
  >
    <div class="relative flex h-fit w-full flex-col items-center justify-between md:flex-row">
      <h1 class="text-contrast text-4xl font-bold">Servers</h1>
      <div class="flex w-full flex-row items-center justify-end gap-4">
        <button
          v-if="status === 'error'"
          type="button"
          alt="Try to load servers again"
          class="flex items-center gap-2 text-sm font-bold"
          @click="() => refreshNuxtData('serversList')"
        >
          <UpdatedIcon />
          Retry
        </button>
        <div class="relative mb-4 w-full text-sm md:mb-0 md:w-72">
          <label class="sr-only" for="search">Search</label>
          <SearchIcon
            class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2"
            aria-hidden="true"
          />
          <input
            id="search"
            v-model="searchInput"
            class="w-full border-[1px] border-solid border-[var(--color-button-border)] pl-9"
            type="search"
            name="search"
            autocomplete="off"
            placeholder="Search servers..."
          />
        </div>
      </div>
    </div>

    <div v-if="status !== 'success'" data-pyro-status>
      <div class="relative flex max-h-[128px] w-full flex-row rounded-3xl bg-bg-raised p-4">
        <Transition name="fade">
          <div v-if="status === 'error'" class="flex h-full w-full items-center gap-6">
            <img
              alt=""
              class="h-full w-full max-w-24 rounded-2xl object-cover"
              height="256"
              src="https://media.tenor.com/scX-kVPwUn8AAAAC/this-is-fine.gif"
              width="256"
              loading="lazy"
              decoding="async"
            />
            <div class="leading-[165%]">
              <h1 class="m-0 mb-2 text-2xl font-semibold">Unable to load servers</h1>
              <p class="max-w-2xl">
                Your servers are safe, but could not be loaded due to a technical issue on our end.
                Please try again later. If this issue persists, please contact
                <a
                  class="text-[var(--color-link)]"
                  href="https://discord.modrinth.com/"
                  rel="noopener noreferrer"
                  target="_blank"
                >
                  support on Discord.
                </a>
              </p>
            </div>
          </div>

          <div v-else class="min-h-[128px]"></div>
        </Transition>
      </div>
    </div>

    <div v-else-if="!data || data.length === 0" class="flex h-full items-center justify-center">
      <p class="text-contrast">No servers found</p>
    </div>

    <ul v-else>
      <div
        v-if="filteredData && filteredData.length === 0"
        class="flex h-full items-center justify-center"
      >
        <p class="text-contrast">No servers found</p>
      </div>
      <div class="flex flex-col gap-4" v-else>
        <ServerListing
          v-for="server in filteredData"
          :key="server.server_id"
          :server_id="server.server_id"
          :name="server.name"
          :state="server.state"
          :net="server.net"
          :modpack="server.modpack"
          :game="server.game"
          :loader="server.loader"
          :loader_version="server.loader_version"
          :mc_version="server.mc_version"
          :mods="server.mods"
          :modpack_id="server.modpack_id"
          :project="server.project"
        />
      </div>
    </ul>

    <UiServersPoweredByPyro />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import Fuse from "fuse.js";
import { SearchIcon, UpdatedIcon } from "@modrinth/assets";
import ServerListing from "~/components/ui/servers/ServerListing.vue";

import type { Server } from "~/types/servers";

definePageMeta({
  middleware: "auth",
});

useHead({
  title: "Servers - Modrinth",
});

const auth = await useAuth();

const { data, status } = await useLazyAsyncData("serversList", async () => {
  const response = await usePyroFetch<{
    servers: Server[];
  }>(auth.value.token, "servers");
  return response.servers;
});

const searchInput = ref("");

const fuse = computed(
  () =>
    new Fuse(data.value || [], {
      keys: ["loader"],
      includeScore: true,
      threshold: 0.4,
    }),
);

const filteredData = computed(() => {
  if (!searchInput.value.trim()) {
    return data.value;
  }

  return fuse.value.search(searchInput.value).map((result) => result.item);
});
</script>

<style>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.1s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
