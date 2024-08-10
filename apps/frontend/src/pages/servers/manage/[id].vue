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
        <Button transparent @click="copyText(data.net.ip + ':' + data.net.port)">
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

  <PyroLoading v-else-if="status === 'pending'" />

  <div
    v-else-if="data && data.state === 'installing'"
    class="relative -mt-12 flex h-screen min-h-[400px] w-full flex-1 items-center justify-center"
  >
    <div
      class="bg-loading-animation absolute inset-0 -mt-8"
      style="
        background: linear-gradient(0deg, rgba(22, 24, 28, 0.64), rgba(22, 24, 28, 0.64)),
          linear-gradient(180deg, rgba(66, 131, 92, 0.275) 0%, rgba(17, 35, 43, 0.5) 97.29%);
      "
    ></div>
    <div
      class="bg-loading-animation pointer-events-none absolute inset-0 mx-auto flex h-full w-full max-w-7xl select-none items-center justify-center"
    >
      <img
        src="~/assets/images/games/bg-mock.png"
        alt="Background"
        class="absolute inset-0 mt-12 h-full w-full object-fill"
      />
    </div>
    <div class="pyro-logo-animation relative flex flex-col items-center gap-4 p-8">
      <svg
        class="h-5 w-5 animate-spin text-white"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
      >
        <circle
          class="opacity-25"
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          stroke-width="4"
        ></circle>
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        ></path>
      </svg>
      <h1
        class="m-0 inline-flex items-center gap-2 text-4xl font-bold text-[var(--color-contrast)]"
      >
        Your server is installing...
      </h1>
      <p class="max-w-sm text-center leading-relaxed text-secondary">
        Sit tight, your server is being set up. This should only take a minute.
      </p>
      <Button color="primary" @click="refreshNuxtData('specificServer')"> Refresh </Button>
    </div>
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
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";
import PyroError from "~/components/ui/servers/PyroError.vue";
import LoaderIcon from "~/components/ui/servers/LoaderIcon.vue";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

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
