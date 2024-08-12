<template>
  <UiServersPyroLoading class="h-screen" v-if="status === 'pending'" />
  <div
    v-if="data && status === 'success'"
    data-pyro-server-manager-root
    class="mx-auto flex min-h-screen w-full max-w-[1280px] flex-col gap-6 px-4 sm:px-6"
  >
    <div class="flex flex-row items-center gap-6 pt-4">
      <UiAvatar
        v-if="data && data.project"
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
          {{ data?.name }}
        </h1>
        <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
          <UiServersServerGameLabel
            v-if="showGameLabel"
            :game="data.game!"
            :mcVersion="data.mc_version ?? ''"
          />
          <UiServersServerLoaderLabel
            v-if="showLoaderLabel"
            :loader="data.loader!"
            :loaderVersion="data.loader_version ?? ''"
          />
          <UiServersServerModLabel v-if="showModLabel" :mods="data.mods" />
        </div>
      </div>
    </div>

    <div class="flex flex-row items-center justify-between">
      <UiNavTabs :links="navLinks" />

      <div class="flex flex-row gap-2">
        <UiServersPanelCopyIP :ip="data.net.ip" :port="data.net.port" />
        <UiServersPanelPlay :serverId="data.server_id" :ip="data.net.ip" :port="data.net.port" />
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

  <PyroError v-else-if="status === 'error'" :title="errorTitle" :message="errorMessage" />
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { HomeIcon, CubeIcon, CloudIcon, CogIcon, LeftArrowIcon } from "@modrinth/assets";
import { useServerStore } from "~/stores/servers";
import PyroError from "~/components/ui/servers/PyroError.vue";
import { PyroFetchError } from "~/composables/pyroFetch";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const errorTitle = ref("Error");
const errorMessage = ref("An unexpected error occurred.");

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

const { data, status } = await useLazyAsyncData(
  "ServerPage",
  async () => {
    try {
      await serverStore.fetchServerData(serverId);
      return serverStore.getServerData(serverId);
    } catch (error) {
      if (error instanceof PyroFetchError) {
        switch (error.statusCode) {
          case 401:
            errorTitle.value = "Server Not Found";
            errorMessage.value = "The server you are looking for does not exist.";
            break;
          case 404:
            errorTitle.value = "Server Not Found";
            errorMessage.value = "The server you are looking for does not exist.";
            break;
          default:
            errorTitle.value = "Error";
            errorMessage.value = `An error occurred: ${error.message}`;
        }
      } else {
        errorTitle.value = "Unexpected Error";
        errorMessage.value = "An unexpected error occurred while fetching server data.";
      }
      throw error;
    }
  },
  {
    server: false,
  },
);
</script>
