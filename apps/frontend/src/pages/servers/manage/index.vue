<template>
  <div
    data-pyro-server-list-root
    class="experimental-styles-within relative mx-auto flex min-h-screen w-full max-w-[1280px] flex-col px-6"
  >
    <div
      v-if="serverList.length > 0 || isPollingForNewServers"
      class="relative flex h-fit w-full flex-col items-center justify-between md:flex-row"
    >
      <h1 class="w-full text-4xl font-bold text-contrast">Servers</h1>
      <div class="mb-4 flex w-full flex-row items-center justify-end gap-2 md:mb-0 md:gap-4">
        <div class="relative w-full text-sm md:w-72">
          <label class="sr-only" for="search">Search</label>
          <SearchIcon
            class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2"
            aria-hidden="true"
          />
          <input
            id="search"
            v-model="searchInput"
            class="w-full border-[1px] border-solid border-button-border pl-9"
            type="search"
            name="search"
            autocomplete="off"
            placeholder="Search servers..."
          />
        </div>
        <ButtonStyled type="standard">
          <NuxtLink
            class="!h-10 whitespace-pre !border-[1px] !border-solid !border-button-border text-sm !font-medium"
            :to="{ path: '/servers', hash: '#plan' }"
          >
            <PlusIcon class="size-4" />
            New server
          </NuxtLink>
        </ButtonStyled>
      </div>
    </div>

    <LazyUiServersServerManageEmptyState
      v-if="serverList.length === 0 && !isPollingForNewServers"
    />

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
          :net="server.net"
        />
        <LazyUiServersServerListingSkeleton v-if="isPollingForNewServers" />
      </ul>
      <div v-else class="flex h-full items-center justify-center">
        <p class="text-contrast">No servers found.</p>
      </div>
    </template>

    <UiServersPoweredByPyro />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import Fuse from "fuse.js";
import { PlusIcon, SearchIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import type { Server } from "~/types/servers";

definePageMeta({
  middleware: "auth",
});

useHead({
  title: "Servers - Modrinth",
});

interface ServerResponse {
  servers: Server[];
}

const route = useRoute();
const isPollingForNewServers = ref(false);

const { data: serverResponse, refresh } = await useAsyncData<ServerResponse>(
  "ServerList",
  async () => {
    try {
      const response = await usePyroFetch<{ servers: Server[] }>("servers");
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

const previousServerList = ref<Server[]>([]);
const refreshCount = ref(0);

const checkForNewServers = async () => {
  await refresh();
  refreshCount.value += 1;
  if (JSON.stringify(previousServerList.value) !== JSON.stringify(serverList.value)) {
    isPollingForNewServers.value = false;
    clearInterval(intervalId);
  } else if (refreshCount.value >= 5) {
    isPollingForNewServers.value = false;
    clearInterval(intervalId);
  }
};

let intervalId: ReturnType<typeof setInterval> | undefined;

onMounted(() => {
  if (route.query.redirect_status === "succeeded") {
    isPollingForNewServers.value = true;
    previousServerList.value = [...serverList.value];
    intervalId = setInterval(checkForNewServers, 5000);
  }
});

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId);
  }
});
</script>
