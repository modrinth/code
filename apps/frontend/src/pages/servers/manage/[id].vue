<template>
  <ClientOnly>
    <div class="contents">
      <UiServersPanelLoading v-if="isLoading" class="h-screen" />
      <div
        v-else-if="data"
        data-pyro-server-manager-root
        class="mx-auto flex min-h-screen w-full max-w-[1280px] flex-col gap-6 px-4 sm:px-6"
      >
        <div class="flex flex-row items-center gap-6 pt-4">
          <UiAvatar
            v-if="data.project"
            no-shadow
            size="lg"
            alt="Server Icon"
            :src="data.project.icon_url"
          />
          <div class="flex flex-col gap-4">
            <div class="-mb-2 flex shrink-0 flex-row items-center gap-1">
              <NuxtLink to="/servers/manage" class="breadcrumb goto-link flex w-fit items-center">
                <LeftArrowIcon />
                All servers
              </NuxtLink>
            </div>
            <h1 class="m-0 text-4xl font-bold text-[var(--color-contrast)]">
              {{ data.name }}
            </h1>
            <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
              <UiServersServerGameLabel
                v-if="showGameLabel"
                :game="data.game!"
                :mc-version="data.mc_version ?? ''"
              />
              <UiServersServerLoaderLabel
                v-if="showLoaderLabel"
                :loader="data.loader!"
                :loader-version="data.loader_version ?? ''"
              />
              <UiServersServerModLabel v-if="showModLabel" :mods="data.mods" />
            </div>
          </div>
        </div>

        <div class="flex flex-row items-center justify-between">
          <UiNavTabs :links="navLinks" />

          <div class="flex flex-row gap-2">
            <UiServersPanelCopyIP :ip="data.net.ip" :port="data.net.port" />
            <UiServersPanelPlay
              :server-id="data.server_id"
              :ip="data.net.ip"
              :port="data.net.port"
            />
          </div>
        </div>

        <div data-pyro-mount class="h-full w-full">
          <NuxtPage
            :route="route"
            :transition="{
              name: 'page',
              mode: 'out-in',
            }"
          />
        </div>

        <UiServersPoweredByPyro />
      </div>

      <PyroError v-else-if="error" :title="errorTitle" :message="errorMessage" />
    </div>
  </ClientOnly>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { HomeIcon, CubeIcon, CloudIcon, CogIcon, LeftArrowIcon } from "@modrinth/assets";
import { useServerStore } from "~/stores/servers.ts";
import PyroError from "~/components/ui/servers/PyroError.vue";
import { PyroFetchError } from "~/composables/pyroFetch.ts";
import type { Server } from "~/types/servers";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const errorTitle = ref("Error");
const errorMessage = ref("An unexpected error occurred.");
const isLoading = ref(true);
const error = ref<Error | null>(null);
const data = ref<Server | null>(null);

const showGameLabel = computed(() => !!data.value?.game);
const showLoaderLabel = computed(() => !!data.value?.loader);
const showModLabel = computed(() => (data.value?.mods?.length ?? 0) > 0);

const navLinks = [
  { icon: HomeIcon, label: "Overview", href: `/servers/manage/${serverId}` },
  { icon: CubeIcon, label: "Content", href: `/servers/manage/${serverId}/content` },
  { icon: CloudIcon, label: "Backups", href: `/servers/manage/${serverId}/backups` },
  { icon: CogIcon, label: "Options", href: `/servers/manage/${serverId}/options` },
];

definePageMeta({
  middleware: "auth",
});

onMounted(async () => {
  try {
    isLoading.value = true;
    await serverStore.fetchServerData(serverId);
    data.value = serverStore.getServerData(serverId) || null;
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
