<template>
  <div
    data-pyro-server-list-root
    class="experimental-styles-within relative mx-auto flex min-h-screen w-full max-w-[1280px] flex-col px-3"
  >
    <div class="relative flex h-fit w-full flex-col items-center justify-between md:flex-row">
      <h1 class="text-4xl font-bold text-[--color-contrast]">Servers</h1>
      <div class="flex w-full flex-row items-center justify-end gap-4">
        <button
          v-if="status === 'error'"
          type="button"
          alt="Reload servers"
          class="flex items-center gap-2 bg-transparent text-sm font-bold"
          @click="() => refreshNuxtData('ServerList')"
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
              class="h-full w-full max-w-24 rounded-2xl object-cover align-middle"
              height="256"
              src="~/assets/images/servers/this-is-fine.gif"
              width="256"
              loading="lazy"
              decoding="async"
            />
            <div class="leading-[165%]">
              <h1 class="m-0 mb-2 text-2xl font-semibold">Unable to load servers</h1>
              <p class="m-0 max-w-2xl">
                Your servers are safe, but could not be loaded due to a technical issue on our end.
                Please try again later. If this issue persists, please contact
                <a
                  class="text-[var(--color-link)]"
                  href="https://support.modrinth.com/"
                  rel="noopener noreferrer"
                  target="_blank"
                >
                  Modrinth support.
                </a>
              </p>
            </div>
          </div>

          <div v-else-if="status === 'pending'" class="min-h-[128px]"></div>
        </Transition>
      </div>
    </div>

    <div
      v-else-if="serverList.length === 0"
      class="flex h-full min-h-[128px] items-center justify-center"
    >
      <p class="text-contrast">You don't have any servers yet.</p>
    </div>

    <template v-else>
      <ul v-if="filteredData.length > 0" class="m-0 flex flex-col gap-4 p-0">
        <UiServersServerListing
          v-for="server in filteredData"
          :key="server.server_id"
          :server_id="server.server_id"
          :name="server.name"
          :status="server.status"
          :game="server.game"
          :loader="server.loader"
          :loader_version="server.loader_version"
          :mc_version="server.mc_version"
          :upstream="server.upstream"
        />
      </ul>
      <div v-else class="flex h-full items-center justify-center">
        <p class="text-contrast">No servers found.</p>
      </div>
    </template>

    <UiServersPoweredByPyro />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import Fuse from "fuse.js";
import { SearchIcon, UpdatedIcon } from "@modrinth/assets";
import type { Server } from "~/types/servers";

definePageMeta({
  middleware: "auth",
});

useHead({
  title: "Servers - Modrinth",
});

const serverStore = useServerStore();

interface ServerResponse {
  servers: Server[];
}

const { data: serverResponse, status } = await useAsyncData<ServerResponse>(
  "ServerList",
  async () => {
    try {
      const response = await serverStore.listServers();
      return response;
    } catch {
      throw new PyroFetchError("Unable to load servers");
    }
  },
);

const serverList = computed(() => serverResponse.value?.servers || []);

const searchInput = ref("");

const fuse = computed(() => {
  if (serverList.value.length === 0) return null;
  return new Fuse(serverList.value, {
    keys: ["name", "loader", "mc_version", "game", "state"],
    includeScore: true,
    threshold: 0.4,
  });
});

const filteredData = computed(() => {
  if (!searchInput.value.trim()) {
    return serverList.value;
  }
  return fuse.value ? fuse.value.search(searchInput.value).map((result) => result.item) : [];
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
