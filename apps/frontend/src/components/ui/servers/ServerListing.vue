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
    <UiAvatar :src="iconUrl" no-shadow size="md" alt="Server Icon" />
    <div class="ml-8 flex flex-col gap-3">
      <div class="flex flex-col gap-2 md:flex-row md:items-center">
        <h2 class="m-0 text-xl font-bold text-[var(--color-contrast)]">{{ name }}</h2>
        <UiServersServerInstallStatusPill v-if="status.state" :state="status.state" />
        <ChevronRightIcon v-if="!status.isInstalling && !status.isFailed" />
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
import { computed, toRaw } from "vue";
import { ChevronRightIcon } from "@modrinth/assets";
import type { StatusState } from "./ServerInstallStatusPill.vue";
import type { Project, Server } from "~/types/servers";

const props = defineProps<Partial<Server>>();

const status = computed(() => ({
  state: props.state as StatusState | undefined,
  isFailed: props.state === "Failed",
  isInstalling: props.state === "Installing",
}));

const showGameLabel = computed(() => !!props.game);
const showLoaderLabel = computed(() => !!props.loader);
const showModLabel = computed(() => (props.mods?.length ?? 0) > 0);

const { data: iconData } = await useLazyAsyncData(`server-icon-${props.server_id}`, async () => {
  if (props.modpack) {
    const versionData: any = await toRaw(useBaseFetch(`version/${props.modpack}`));
    if (versionData && versionData.project_id) {
      const projectData: Project = (await toRaw(
        useBaseFetch(`project/${versionData.project_id}`),
      )) as Project;
      return projectData.icon_url;
    }
  }
  return null;
});

const iconUrl = computed(() => iconData.value || undefined);
</script>
