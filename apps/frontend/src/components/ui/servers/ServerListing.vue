<template>
  <NuxtLink
    :href="`/servers/manage/${serverId}`"
    class="flex flex-row items-center rounded-3xl bg-[var(--color-raised-bg)] p-4"
    data-pyro-server-listing
  >
    <UiAvatar no-shadow size="md" :src="''" alt="Server Icon" />
    <div class="ml-8 flex flex-col gap-3">
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-xl font-bold">test</h2>
        <ChevronRightIcon />
      </div>

      <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
        <div class="flex flex-row items-center gap-2">
          <img
            src="~assets/images/games/minecraft-java-edition.png"
            alt="Minecraft Java Edition"
            class="size-5"
          />
          <span class="text-sm font-semibold">Minecraft</span>
        </div>

        <div class="h-6 w-0.5 bg-[var(--color-button-border)]"></div>
        <div class="flex flex-row items-center gap-2">
          <LoaderIcon />
          <span class="text-sm font-semibold capitalize"> {{ loader }} {{ version }} </span>
        </div>

        <div class="h-6 w-0.5 bg-[var(--color-button-border)]"></div>
        <div class="flex flex-row items-center gap-2">
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
