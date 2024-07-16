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
          <div class="h-6 w-0.5 bg-button-border"></div>
          <div class="flex flex-row items-center gap-2">
            <LoaderIcon />
            <span class="text-sm font-semibold"
              >{{ data && data.loader.charAt(0).toUpperCase() + data.loader.slice(1) }}
              {{ data?.version }}</span
            >
          </div>
          <div class="h-6 w-0.5 bg-button-border"></div>
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

  <div v-else-if="status === 'pending'" class="flex h-screen w-full items-center justify-center">
    <PyroIcon class="pyro-logo-animation size-32 opacity-10" />
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
  LoaderIcon,
  LeftArrowIcon,
  PyroIcon,
} from "@modrinth/assets";

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

@keyframes zoom-in {
  0% {
    transform: scale(0.5);
  }
  100% {
    transform: scale(1);
  }
}

.pyro-logo-animation {
  animation: zoom-in 0.8s
    linear(
      0 0%,
      0.01 0.8%,
      0.04 1.6%,
      0.161 3.3%,
      0.816 9.4%,
      1.046 11.9%,
      1.189 14.4%,
      1.231 15.7%,
      1.254 17%,
      1.259 17.8%,
      1.257 18.6%,
      1.236 20.45%,
      1.194 22.3%,
      1.057 27%,
      0.999 29.4%,
      0.955 32.1%,
      0.942 33.5%,
      0.935 34.9%,
      0.933 36.65%,
      0.939 38.4%,
      1 47.3%,
      1.011 49.95%,
      1.017 52.6%,
      1.016 56.4%,
      1 65.2%,
      0.996 70.2%,
      1.001 87.2%,
      1 100%
    );
}

@keyframes fade-bg-in {
  0% {
    opacity: 0;
  }
  100% {
    opacity: 0.6;
  }
}

.bg-loading-animation {
  animation: fade-bg-in 0.12s linear forwards;
}
</style>
