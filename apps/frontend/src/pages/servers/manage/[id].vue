<template>
  <div class="contents">
    <UiServersPanelLoading v-if="initialLoading" class="h-screen" />
    <div
      v-else-if="serverData && serverData.status !== 'installing'"
      data-pyro-server-manager-root
      class="mx-auto flex min-h-screen w-full max-w-[1280px] flex-col gap-6 px-3"
    >
      <div class="flex flex-row items-center gap-6 pt-4">
        <img
          v-if="serverData?.image"
          no-shadow
          size="lg"
          alt="Server Icon"
          class="h-[6rem] w-[6rem] rounded-xl bg-bg-raised"
          :src="serverData.image"
        />
        <img
          v-else
          no-shadow
          size="lg"
          alt="Server Icon"
          class="h-[6rem] w-[6rem] rounded-xl bg-bg-raised"
          src="~/assets/images/servers/minecraft_server_icon.png"
        />
        <div class="flex flex-col gap-2">
          <div class="flex shrink-0 flex-row items-center gap-1">
            <NuxtLink to="/servers/manage" class="breadcrumb goto-link flex w-fit items-center">
              <LeftArrowIcon />
              All servers
            </NuxtLink>
          </div>
          <div class="flex flex-row items-center gap-4">
            <h1 class="m-0 text-4xl font-bold text-[var(--color-contrast)]">
              {{ serverData.name }}
            </h1>
          </div>

          <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
            <UiServersServerGameLabel
              v-if="showGameLabel"
              :game="serverData.game!"
              :mc-version="serverData.mc_version ?? ''"
            />
            <UiServersServerLoaderLabel
              v-if="showLoaderLabel"
              :loader="serverData.loader!"
              :loader-version="serverData.loader_version ?? ''"
            />
            <UiServersServerModLabel v-if="showModLabel" :mods="serverData.mods" />
            <UiServersServerSubdomainLabel
              v-if="serverData.net.domain"
              :subdomain="serverData.net.domain"
            />
          </div>
        </div>
      </div>

      <div class="flex w-full flex-col justify-between gap-4 md:flex-row md:items-center">
        <UiNavTabs :links="navLinks" />
      </div>

      <div data-pyro-mount class="h-full w-full flex-1">
        <NuxtPage :route="route" />
      </div>

      <UiServersPoweredByPyro />
    </div>

    <UiServersPyroError v-else-if="error" :title="errorTitle" :message="errorMessage" />
    <div v-else class="flex h-screen flex-col items-center justify-center">
      <p class="text-lg font-bold">Get ready! We're preparing your server for you.</p>
      <div class="h-1.5 w-full max-w-lg overflow-hidden rounded-xl bg-brand-highlight">
        <div class="progress left-right h-full w-full bg-brand"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { LeftArrowIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const errorTitle = ref("Error");
const errorMessage = ref("An unexpected error occurred.");
const initialLoading = ref(true);

const {
  data: serverData,
  error,
  refresh,
} = useAsyncData(async () => {
  try {
    await serverStore.fetchServerData(serverId);
    initialLoading.value = false;
    return serverStore.serverData[serverId];
  } catch (err) {
    initialLoading.value = false;
    if (err instanceof PyroFetchError) {
      switch (err.statusCode) {
        case 400:
          errorTitle.value = "Oh no, a pop-up";
          errorMessage.value = "Request was malformed.";
          break;
        case 401:
        case 404:
          errorTitle.value = "Server Not Found";
          errorMessage.value = "The server you are looking for does not exist.";
          break;
        default:
          errorTitle.value = "Error";
          errorMessage.value = `An error occurred: ${err.message}`;
      }
    } else {
      errorTitle.value = "Unexpected Error";
      errorMessage.value = "An unexpected error occurred while fetching server data.";
    }
    throw err;
  }
});

const showGameLabel = computed(() => !!serverData.value?.game);
const showLoaderLabel = computed(() => !!serverData.value?.loader);
const showModLabel = computed(() => (serverData.value?.mods?.length ?? 0) > 0);

const navLinks = [
  { label: "Overview", href: `/servers/manage/${serverId}`, subpages: [] },
  {
    label: "Content",
    href: `/servers/manage/${serverId}/content`,
    subpages: ["mods", "datapacks"],
  },
  { label: "Files", href: `/servers/manage/${serverId}/files`, subpages: [] },
  { label: "Backups", href: `/servers/manage/${serverId}/backups`, subpages: [] },
  {
    label: "Options",
    href: `/servers/manage/${serverId}/options`,
    subpages: ["startup", "network", "properties", "info"],
  },
];

let intervalId: ReturnType<typeof setInterval> | null = null;

const startPolling = () => {
  intervalId = setInterval(async () => {
    await refresh();
  }, 10000);
};

const stopPolling = () => {
  if (intervalId) {
    clearInterval(intervalId);
    intervalId = null;
  }
};

onMounted(() => {
  if (serverData.value?.status === "installing") {
    startPolling();
  }
});

onUnmounted(() => {
  stopPolling();
});

watch(
  () => serverData.value?.status,
  (newStatus) => {
    if (newStatus === "installing") {
      startPolling();
    } else {
      stopPolling();
    }
  },
);

definePageMeta({
  middleware: "auth",
});
</script>

<style scoped>
.progress {
  animation: progress 1s infinite linear;
}

.left-right {
  transform-origin: 0% 50%;
}

@keyframes progress {
  0% {
    transform: translateX(0) scaleX(0);
  }
  40% {
    transform: translateX(0) scaleX(0.4);
  }
  100% {
    transform: translateX(100%) scaleX(0.5);
  }
}
</style>
