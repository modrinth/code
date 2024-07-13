<template>
  <div
    v-if="status === 'success'"
    data-pyro-server-manager-root
    class="mx-auto flex min-h-screen w-full max-w-[1280px] flex-col gap-6 px-4 sm:px-6"
  >
    <div class="flex flex-row items-center gap-6 pt-4">
      <UiAvatar
        no-shadow
        size="lg"
        src="https://cdn.modrinth.com/data/23niDfW7/20644fac7c3890555049874f2cfb1040d10a5126.jpeg"
        alt="Server Icon"
      />
      <div class="flex flex-col gap-4">
        <div class="-mb-2 flex shrink-0 flex-row items-center gap-1">
          <NuxtLink to="/servers/manage" class="breadcrumb goto-link flex w-fit items-center">
            <LeftArrowIcon />
            All Servers
          </NuxtLink>
        </div>
        <h1 class="m-0 text-4xl font-bold text-[var(--color-contrast)]">
          {{ data?.name }}
        </h1>
        <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
          <div class="flex flex-row items-center gap-2">
            <img
              src="~/assets/images/games/minecraft-java-edition.png"
              alt="Minecraft Java Edition"
              class="size-5"
            />
            <span class="text-sm font-semibold">{{
              data && data.game.charAt(0).toUpperCase() + data.game.slice(1)
            }}</span>
          </div>
          <div class="h-6 w-0.5 bg-[var(--color-button-border)]"></div>
          <div class="flex flex-row items-center gap-2">
            <LoaderIcon />
            <span class="text-sm font-semibold"
              >{{ data && data.loader.charAt(0).toUpperCase() + data.loader.slice(1) }}
              {{ data?.version }}</span
            >
          </div>
          <div class="h-6 w-0.5 bg-[var(--color-button-border)]"></div>
          <div class="flex flex-row items-center gap-2">
            <BoxIcon />
            <span class="text-sm font-semibold">{{ data?.mods.length }} mods</span>
          </div>
        </div>
      </div>
    </div>

    <div class="flex flex-row items-center justify-between">
      <UiNavTabs
        :links="[
          {
            icon: HomeIcon,
            label: 'Overview',
            href: `/servers/manage/${serverId}`,
          },
          {
            icon: CubeIcon,
            label: 'Content',
            href: `/servers/manage/${serverId}/content`,
          },
          {
            icon: CloudIcon,
            label: 'Backups',
            href: `/servers/manage/${serverId}/backups`,
          },
          {
            icon: CogIcon,
            label: 'Options',
            href: `/servers/manage/${serverId}/options`,
          },
        ]"
      />

      <div class="flex flex-row gap-2">
        <Button transparent onclick="navigator.clipboard.writeText('{{ data?.net.ip }}')">
          <CopyIcon />

          Copy IP
        </Button>
        <Button color="primary">
          <PlayIcon />
          Join
        </Button>
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
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";
import { HomeIcon, CubeIcon, CloudIcon, CogIcon, CopyIcon, PlayIcon, BoxIcon, LoaderIcon, LeftArrowIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id;

definePageMeta({
  middleware: "auth",
});

type ServerDetails = {
  name: string;
  server_id: string;
  net: {
    ip: string;
    port: number;
    domain: string;
  };
  modpack: string;
  game: string;
  loader: string;
  version: string;
  mods: {
    id: number;
    filename: string;
    modrinth_ids: {
      project_id: string;
      version_id: string;
    };
  }[];
};

const { data, status } = await useLazyAsyncData("serversList", async () => {
  return await usePyroFetch<ServerDetails>(0, `servers/${serverId}`);
});
</script>

<style>
.page-enter-active,
.page-leave-active {
  transition: all 0.1s;
}
.page-enter-from,
.page-leave-to {
  opacity: 0;
}
</style>
