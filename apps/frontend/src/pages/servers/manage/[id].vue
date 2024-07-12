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
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
              class="size-5"
            >
              <path
                fill-rule="evenodd"
                d="M17 10a.75.75 0 0 1-.75.75H5.612l4.158 3.96a.75.75 0 1 1-1.04 1.08l-5.5-5.25a.75.75 0 0 1 0-1.08l5.5-5.25a.75.75 0 1 1 1.04 1.08L5.612 9.25H16.25A.75.75 0 0 1 17 10Z"
                clip-rule="evenodd"
              />
            </svg>
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
            <svg
              xmlns="http://www.w3.org/2000/svg"
              xml:space="preserve"
              fill-rule="evenodd"
              stroke-linecap="round"
              stroke-linejoin="round"
              clip-rule="evenodd"
              viewBox="0 0 24 24"
              class="size-5"
            >
              <path fill="none" d="M0 0h24v24H0z"></path>
              <path
                fill="none"
                stroke="currentColor"
                stroke-width="23"
                d="m820 761-85.6-87.6c-4.6-4.7-10.4-9.6-25.9 1-19.9 13.6-8.4 21.9-5.2 25.4 8.2 9 84.1 89 97.2 104 2.5 2.8-20.3-22.5-6.5-39.7 5.4-7 18-12 26-3 6.5 7.3 10.7 18-3.4 29.7-24.7 20.4-102 82.4-127 103-12.5 10.3-28.5 2.3-35.8-6-7.5-8.9-30.6-34.6-51.3-58.2-5.5-6.3-4.1-19.6 2.3-25 35-30.3 91.9-73.8 111.9-90.8"
                transform="matrix(.08671 0 0 .0867 -49.8 -56)"
              ></path>
            </svg>
            <span class="text-sm font-semibold"
              >{{ data && data.loader.charAt(0).toUpperCase() + data.loader.slice(1) }}
              {{ data?.version }}</span
            >
          </div>
          <div class="h-6 w-0.5 bg-[var(--color-button-border)]"></div>
          <div class="flex flex-row items-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke-width="1.5"
              stroke="currentColor"
              class="size-5"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="m21 7.5-9-5.25L3 7.5m18 0-9 5.25m9-5.25v9l-9 5.25M3 7.5l9 5.25M3 7.5v9l9 5.25m0-9v9"
              ></path>
            </svg>
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
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            class="size-5"
          >
            <path
              fill-rule="evenodd"
              d="M15.988 3.012A2.25 2.25 0 0 1 18 5.25v6.5A2.25 2.25 0 0 1 15.75 14H13.5V7A2.5 2.5 0 0 0 11 4.5H8.128a2.252 2.252 0 0 1 1.884-1.488A2.25 2.25 0 0 1 12.25 1h1.5a2.25 2.25 0 0 1 2.238 2.012ZM11.5 3.25a.75.75 0 0 1 .75-.75h1.5a.75.75 0 0 1 .75.75v.25h-3v-.25Z"
              clip-rule="evenodd"
            />
            <path
              fill-rule="evenodd"
              d="M2 7a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V7Zm2 3.25a.75.75 0 0 1 .75-.75h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1-.75-.75Zm0 3.5a.75.75 0 0 1 .75-.75h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1-.75-.75Z"
              clip-rule="evenodd"
            />
          </svg>

          Copy IP
        </Button>
        <Button color="primary">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            class="size-5"
          >
            <path
              d="M6.3 2.84A1.5 1.5 0 0 0 4 4.11v11.78a1.5 1.5 0 0 0 2.3 1.27l9.344-5.891a1.5 1.5 0 0 0 0-2.538L6.3 2.841Z"
            />
          </svg>
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
import HomeIcon from "~/assets/images/utils/navtabs/home.svg?component";
import CubeIcon from "~/assets/images/utils/navtabs/cube.svg?component";
import CloudIcon from "~/assets/images/utils/navtabs/cloud.svg?component";
import CogIcon from "~/assets/images/utils/navtabs/cog.svg?component";

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
