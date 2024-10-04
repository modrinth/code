<template>
  <div class="contents">
    <UiServersPanelLoading v-if="isLoading" class="h-screen" />
    <div
      v-else-if="serverData"
      data-pyro-server-manager-root
      class="mx-auto flex min-h-screen w-full max-w-[1280px] flex-col gap-6 px-3"
    >
      <div class="flex flex-row items-center gap-6 pt-4">
        <img
          v-if="serverData?.image"
          no-shadow
          size="lg"
          alt="Server Icon"
          class="h-[9rem] w-[9rem] rounded-xl bg-bg-raised"
          :src="serverData.image"
        />
        <img
          v-else
          no-shadow
          size="lg"
          alt="Server Icon"
          class="h-[9rem] w-[9rem] rounded-xl bg-bg-raised"
          src="~/assets/images/servers/minecraft_server_icon.png"
        />
        <div class="flex flex-col gap-4">
          <div class="-mb-2 flex shrink-0 flex-row items-center gap-1">
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
          </div>
        </div>
      </div>

      <div class="flex w-full flex-col justify-between gap-4 md:flex-row md:items-center">
        <UiNavTabs :links="navLinks" />
      </div>

      <div data-pyro-mount class="h-full w-full">
        <NuxtPage :route="route" />
      </div>

      <UiServersPoweredByPyro />
    </div>

    <UiServersPyroError v-else-if="error" :title="errorTitle" :message="errorMessage" />
  </div>
</template>

<script setup lang="ts">
import { LeftArrowIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const errorTitle = ref("Error");
const errorMessage = ref("An unexpected error occurred.");
const isLoading = ref(true);
const error = ref<Error | null>(null);

const serverData = computed(() => serverStore.serverData[serverId]);

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

definePageMeta({
  middleware: "auth",
});

onMounted(async () => {
  try {
    isLoading.value = true;
    await serverStore.fetchServerData(serverId);
  } catch (err) {
    error.value = err as Error;
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
  } finally {
    isLoading.value = false;
  }
});
</script>
