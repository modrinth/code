<template>
  <NuxtLink
    :to="`/servers/manage/${server_id}`"
    :aria-disabled="status.isInstalling || status.isFailed"
    :tabindex="status.isInstalling || status.isFailed ? -1 : 0"
    :class="status.isInstalling || status.isFailed ? 'pointer-events-none cursor-not-allowed' : ''"
    class="flex flex-row items-center overflow-x-hidden rounded-3xl bg-bg-raised p-4"
    data-pyro-server-listing
    :data-pyro-server-listing-id="server_id"
  >
    <!-- how do we want to get this now? -->
    <UiAvatar :src="iconUrl" no-shadow size="md" alt="Server Icon" />
    <div class="ml-8 flex flex-col gap-2.5">
      <div class="flex flex-col gap-2 md:flex-row md:items-center">
        <h2 class="m-0 text-xl font-bold text-[var(--color-contrast)]">{{ name }}</h2>
        <UiServersServerInstallStatusPill v-if="status.state" :state="status.state" />
        <ChevronRightIcon v-if="!status.isInstalling && !status.isFailed" />
      </div>

      <div
        v-if="projectData?.title"
        class="m-0 flex flex-row items-center gap-1 text-sm font-medium text-[var(--color-text-secondary)]"
      >
        <UiAvatar
          :src="iconUrl"
          no-shadow
          style="min-height: 20px; min-width: 20px; height: 20px; width: 20px"
          alt="Server Icon"
        />
        Using {{ projectData?.title || "Unknown" }}
      </div>

      <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
        <UiServersServerGameLabel
          v-if="showGameLabel"
          :game="game!"
          :mc-version="mc_version ?? ''"
        />
        <UiServersServerLoaderLabel
          v-if="showLoaderLabel"
          :loader="loader!"
          :loader-version="loader_version ?? ''"
        />
        <UiServersServerModLabel v-if="showModLabel" :mods="mods || []" />
      </div>
    </div>
  </NuxtLink>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { ChevronRightIcon } from "@modrinth/assets";
import type { StatusState } from "./ServerInstallStatusPill.vue";
import type { Project, Server } from "~/types/servers";

const props = defineProps<Partial<Server>>();

const status = computed(() => ({
  state: props.status as StatusState | undefined,
  isFailed: props.status === "Failed",
  isInstalling: props.status === "Installing",
}));

const showGameLabel = computed(() => !!props.game);
const showLoaderLabel = computed(() => !!props.loader);
const showModLabel = computed(() => (props.mods?.length ?? 0) > 0);

const { data: projectData } = await useLazyAsyncData<Project>(
  `server-project-${props.server_id}`,
  async (): Promise<Project> => {
    const result = await useBaseFetch(`project/${props.upstream?.project_id}`);
    return result as Project;
  },
);

const iconUrl = computed(() => projectData.value?.icon_url || undefined);
</script>
