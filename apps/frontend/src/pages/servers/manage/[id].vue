<template>
  <!--  && data.state === 'running' -->
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
          <div class="flex flex-row items-center gap-2">
            <img
              src="~/assets/images/games/minecraft.png"
              alt="Minecraft Java Edition"
              class="size-5"
            />
            <span class="text-sm font-semibold">
              {{ data && data.game.charAt(0).toUpperCase() + data.game.slice(1) }}
              {{ data?.mc_version }}
            </span>
          </div>
          <div class="h-6 w-0.5 bg-[#26252b]"></div>
          <div class="flex flex-row items-center gap-2">
            <LoaderIcon
              class="grid place-content-center"
              v-if="data.loader"
              :loader="data.loader"
            />
            <span class="text-sm font-semibold">
              {{ data && data.loader }}
            </span>
          </div>
          <div class="h-6 w-0.5 bg-[#26252b]"></div>
          <div class="flex flex-row items-center gap-2">
            <BoxIcon />
            <span class="text-sm font-semibold">{{ data?.mods.length }} mods</span>
          </div>
        </div>
      </div>
    </div>

    <div class="flex flex-row items-center justify-between">
      <UiNavTabs :links="navLinks" />

      <div class="flex flex-row gap-2">
        <Button transparent @click="copyText(data.net.ip + ':' + data.net.port)">
          <CopyIcon />
          Copy IP
        </Button>
        <a
          class="btn btn-primary"
          :href="'modrinth://servers/' + data.server_id + '/' + data.net.ip + ':' + data.net.port"
        >
          <PlayIcon />
          Play
        </a>
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

  <PyroError
    v-else-if="status === 'error'"
    title="Server not found"
    message="The server you are looking for does not exist or you do not have permission to view it."
  />
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";
import {
  HomeIcon,
  CubeIcon,
  CloudIcon,
  CogIcon,
  CopyIcon,
  PlayIcon,
  BoxIcon,
  LeftArrowIcon,
} from "@modrinth/assets";
import { useServerStore } from "~/stores/servers";
import PyroError from "~/components/ui/servers/PyroError.vue";
import LoaderIcon from "~/components/ui/servers/LoaderIcon.vue";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const navLinks = [
  { icon: HomeIcon, label: "Overview", href: `/servers/manage/${serverId}` },
  { icon: CubeIcon, label: "Content", href: `/servers/manage/${serverId}/content` },
  { icon: CloudIcon, label: "Backups", href: `/servers/manage/${serverId}/backups` },
  { icon: CogIcon, label: "Options", href: `/servers/manage/${serverId}/options` },
];

definePageMeta({
  middleware: "auth",
});

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData("ServerPage", async () =>
  serverStore.getServerData(serverId),
);

const copyText = (ip: string) => {
  navigator.clipboard.writeText(ip);
};
</script>
