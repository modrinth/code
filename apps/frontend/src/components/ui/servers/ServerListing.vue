<template>
  <NuxtLink
    :href="`/servers/manage/${server_id}`"
    class="flex flex-row items-center overflow-x-hidden rounded-3xl bg-bg-raised p-4"
    data-pyro-server-listing
  >
    <UiAvatar no-shadow size="md" :src="icon_url" alt="Server Icon" />
    <div class="ml-8 flex flex-col gap-3">
      <div class="flex flex-col gap-2 md:flex-row md:items-center">
        <h2 class="m-0 text-xl font-bold">{{ name }}</h2>
        <div
          v-if="state === 'installing'"
          class="flex w-fit flex-row items-center rounded-full bg-red-500 px-2 py-0.5 text-sm font-bold text-white"
        >
          <svg
            class="mr-2 h-3 w-3 animate-spin text-white"
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
          Installing
        </div>
        <ChevronRightIcon />
      </div>

      <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
        <div v-if="game" class="flex flex-row items-center gap-2">
          <img src="~/assets/images/games/minecraft.png" :alt="`${game} Logo`" class="size-5" />
          <span class="text-sm font-semibold"
            >{{ game[0].toUpperCase() + game.slice(1) }} {{ mc_version }}</span
          >
        </div>

        <div v-if="loader && loader_version" class="h-6 w-0.5 bg-button-border"></div>
        <div v-if="loader && loader_version" class="flex flex-row items-center gap-2">
          <LoaderIcon v-if="loader" :loader="loader" />
          <span class="text-sm font-semibold capitalize"> {{ loader }} {{ loader_version }} </span>
        </div>

        <div v-if="mods?.length > 0" class="h-6 w-0.5 bg-button-border"></div>
        <div v-if="mods?.length > 0" class="flex flex-row items-center gap-2">
          <BoxIcon />
          <span class="text-sm font-semibold">
            {{ mods?.length }}
            <span v-if="mods?.length === 1">Mod</span>
            <span v-else>Mods</span>
          </span>
        </div>
      </div>
    </div>
  </NuxtLink>
</template>

<script setup lang="ts">
import type { Server } from "~/types/servers";

import { ChevronRightIcon, BoxIcon } from "@modrinth/assets";
import LoaderIcon from "./LoaderIcon.vue";

const props = defineProps<Partial<Server>>();

let icon_url: string | undefined = undefined;

if (props.modpack) {
  const pid: any = await toRaw(useBaseFetch(`version/${await props.modpack}`));
  const project: any = await toRaw(useBaseFetch(`project/${pid.project_id}`));
  icon_url = project.icon_url;
}
</script>
