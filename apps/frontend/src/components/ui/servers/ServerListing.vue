<template>
  <NuxtLink
    :href="`/servers/manage/${serverId}`"
    class="flex flex-row items-center rounded-3xl bg-bg-raised p-4"
    data-pyro-server-listing
  >
    <UiAvatar no-shadow size="md" :src="''" alt="Server Icon" />
    <div class="ml-8 flex flex-col gap-3">
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-xl font-bold">{{ name }}</h2>
        <ChevronRightIcon />
      </div>

      <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
        <div v-if="game" class="flex flex-row items-center gap-2">
          <img src="~/assets/images/games/minecraft.png" :alt="`${game} Logo`" class="size-5" />
          <span class="text-sm font-semibold">{{ game[0].toUpperCase() + game.slice(1) }}</span>
        </div>

        <div v-if="loader && loader_version" class="h-6 w-0.5 bg-button-border"></div>
        <div v-if="loader && loader_version" class="flex flex-row items-center gap-2">
          <LoaderIcon />
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
import type { Server as ServerType } from "~/types/servers";

import { ChevronRightIcon, BoxIcon, LoaderIcon } from "@modrinth/assets";

type ServerProps = Omit<ServerType, "server_id"> & { serverId: string };

defineProps<ServerProps>();
</script>
