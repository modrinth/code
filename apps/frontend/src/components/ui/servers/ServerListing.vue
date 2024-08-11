<template>
  <NuxtLink
    :to="`/servers/manage/${server_id}`"
    :aria-disabled="state === 'Installing'"
    :tabindex="state === 'Installing' ? -1 : 0"
    :class="state === 'Installing' ? 'pointer-events-none cursor-not-allowed' : ''"
    class="flex flex-row items-center overflow-x-hidden rounded-3xl bg-bg-raised p-4"
    data-pyro-server-listing
    :data-pyro-server-listing-id="server_id"
  >
    <UiAvatar no-shadow size="md" :src="icon_url" alt="Server Icon" />
    <div class="ml-8 flex flex-col gap-3">
      <div class="flex flex-col gap-2 md:flex-row md:items-center">
        <h2 class="m-0 text-xl font-bold">{{ name }}</h2>
        <UiServersServerInstallStatusPill v-if="state" :state="state as StatusState" />
        <ChevronRightIcon v-if="state !== 'Installing'" />
      </div>

      <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
        <UiServersServerGameLabel v-if="game" :game="game" :mcVersion="mc_version ?? ''" />
        <UiServersServerLoaderLabel
          v-if="loader"
          :loader="loader"
          :loaderVersion="loader_version ?? ''"
        />
        <UiServersServerModLabel v-if="mods?.length ?? 0 > 0" :mods="mods" />
      </div>
    </div>
  </NuxtLink>
</template>

<script setup lang="ts">
import type { Server } from "~/types/servers";
import type { StatusState } from "./ServerInstallStatusPill.vue";

import { ChevronRightIcon } from "@modrinth/assets";

const props = defineProps<Partial<Server>>();

let icon_url: string | undefined = undefined;

if (props.modpack) {
  const pid: any = await toRaw(useBaseFetch(`version/${await props.modpack}`));
  const project: any = await toRaw(useBaseFetch(`project/${pid.project_id}`));
  icon_url = project.icon_url;
}
</script>
