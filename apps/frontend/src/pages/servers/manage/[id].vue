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
            <LoaderIcon v-if="data.loader" :loader="data.loader" />
            <span class="text-sm font-semibold">
              {{
                data && data.loader && data.loader.charAt(0).toUpperCase() + data.loader.slice(1)
              }}
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
        <Button transparent @click="copyText(data.net.ip)">
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

  <div
    v-else-if="status === 'error'"
    class="relative -mt-12 flex h-screen min-h-[400px] w-full flex-1 items-center justify-center"
  >
    <div
      class="bg-loading-animation absolute inset-0 -mt-8"
      style="
        background: linear-gradient(0deg, rgba(22, 24, 28, 0.64), rgba(22, 24, 28, 0.64)),
          linear-gradient(180deg, rgba(131, 66, 66, 0.275) 0%, rgba(202, 14, 14, 0.9) 97.29%);
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
    <div
      class="pyro-logo-animation relative flex flex-col items-center gap-4 rounded-2xl border-2 border-[#FF496E] bg-[#270B11] p-8"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="size-8 text-[#FF496E]"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
        />
      </svg>
      <h1
        class="m-0 inline-flex items-center gap-2 text-4xl font-bold text-[var(--color-contrast)]"
      >
        Server not found
      </h1>
      <p class="max-w-sm text-center leading-relaxed text-secondary">
        The server you are looking for does not exist or you do not have permission to view it.
      </p>
      <Button color="primary" @click="$router.push('/servers/manage')">
        <LeftArrowIcon />
        Back to Servers
      </Button>
    </div>
  </div>
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
import LoaderIcon from "~/components/ui/servers/LoaderIcon.vue";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

definePageMeta({
  middleware: "auth",
});

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData("specificServer", async () =>
  serverStore.getServerData(serverId),
);

const copyText = (ip: string) => {
  navigator.clipboard.writeText(ip);
};
</script>
