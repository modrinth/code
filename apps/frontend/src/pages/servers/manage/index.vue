<template>
  <div
    data-pyro-server-list-root
    class="experimental-styles-within relative mx-auto mb-6 flex min-h-screen w-full max-w-[1280px] flex-col px-6"
  >
    <div
      v-if="hasError || fetchError"
      class="mx-auto flex h-full min-h-[calc(100vh-4rem)] flex-col items-center justify-center gap-4 text-left"
    >
      <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
        <div class="flex flex-col items-center text-center">
          <div class="flex flex-col items-center gap-4">
            <div class="grid place-content-center rounded-full bg-bg-blue p-4">
              <HammerIcon class="size-12 text-blue" />
            </div>
            <h1 class="m-0 w-fit text-3xl font-bold">Servers could not be loaded</h1>
          </div>
          <p class="text-lg text-secondary">We may have temporary issues with our servers.</p>
          <ul class="m-0 list-disc space-y-4 p-0 pl-4 text-left text-sm leading-[170%]">
            <li>
              Our systems automatically alert our team when there's an issue. We are already working
              on getting them back online.
            </li>
            <li>
              If you recently purchased your Modrinth Server, it is currently in a queue and will
              appear here as soon as it's ready. <br />
              <span class="font-medium text-contrast"
                >Do not attempt to purchase a new server.</span
              >
            </li>
            <li>
              If you require personalized support regarding the status of your server, please
              contact Modrinth Support.
            </li>

            <li v-if="fetchError" class="text-red">
              <p>Error details:</p>
              <CopyCode
                :text="(fetchError as ModrinthServersFetchError).message || 'Unknown error'"
                :copyable="false"
                :selectable="false"
                :language="'json'"
              />
            </li>
          </ul>
        </div>
        <ButtonStyled size="large" type="standard" color="brand">
          <a class="mt-6 !w-full" href="https://support.modrinth.com">Contact Modrinth Support</a>
        </ButtonStyled>
        <ButtonStyled size="large" @click="() => reloadNuxtApp()">
          <button class="mt-3 !w-full">Reload</button>
        </ButtonStyled>
      </div>
    </div>

    <LazyUiServersServerManageEmptyState
      v-else-if="serverList.length === 0 && !isPollingForNewServers && !hasError"
    />

    <template v-else>
      <div class="relative flex h-fit w-full flex-col items-center justify-between md:flex-row">
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

      <ul
        v-if="filteredData.length > 0 || isPollingForNewServers"
        class="m-0 flex flex-col gap-4 p-0"
      >
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
          :flows="server.flows"
        />
        <LazyUiServersServerListingSkeleton v-if="isPollingForNewServers" />
      </ul>
      <div v-else class="flex h-full items-center justify-center">
        <p class="text-contrast">No servers found.</p>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import Fuse from "fuse.js";
import { HammerIcon, PlusIcon, SearchIcon } from "@modrinth/assets";
import { ButtonStyled, CopyCode } from "@modrinth/ui";
import type { Server, ModrinthServersFetchError } from "@modrinth/utils";
import { reloadNuxtApp } from "#app";
import { useServersFetch } from "~/composables/servers/servers-fetch.ts";

definePageMeta({
  middleware: "auth",
});

useHead({
  title: "Servers - Modrinth",
});

interface ServerResponse {
  servers: Server[];
}

const router = useRouter();
const route = useRoute();
const hasError = ref(false);
const isPollingForNewServers = ref(false);

const {
  data: serverResponse,
  error: fetchError,
  refresh,
} = await useAsyncData<ServerResponse>("ServerList", () =>
  useServersFetch<ServerResponse>("servers"),
);

watch([fetchError, serverResponse], ([error, response]) => {
  hasError.value = !!error || !response;
});

const serverList = computed(() => {
  if (!serverResponse.value) return [];
  return serverResponse.value.servers;
});

const searchInput = ref("");

const fuse = computed(() => {
  if (serverList.value.length === 0) return null;
  return new Fuse(serverList.value, {
    keys: ["name", "loader", "mc_version", "game", "state"],
    includeScore: true,
    threshold: 0.4,
  });
});

function introToTop(array: Server[]): Server[] {
  return array.slice().sort((a, b) => {
    return Number(b.flows?.intro) - Number(a.flows?.intro);
  });
}

const filteredData = computed(() => {
  if (!searchInput.value.trim()) {
    return introToTop(serverList.value);
  }
  return fuse.value
    ? introToTop(fuse.value.search(searchInput.value).map((result) => result.item))
    : [];
});

const previousServerList = ref<Server[]>([]);
const refreshCount = ref(0);

const checkForNewServers = async () => {
  await refresh();
  refreshCount.value += 1;
  if (JSON.stringify(previousServerList.value) !== JSON.stringify(serverList.value)) {
    isPollingForNewServers.value = false;
    clearInterval(intervalId);
    router.replace({ query: {} });
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
