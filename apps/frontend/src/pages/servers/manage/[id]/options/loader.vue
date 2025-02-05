<template>
  <LazyUiServersPlatformVersionSelectModal
    ref="versionSelectModal"
    :server="props.server"
    :current-loader="data?.loader as Loaders"
    @reinstall="emit('reinstall', $event)"
  />

  <LazyUiServersPlatformMrpackModal
    ref="mrpackModal"
    :server="props.server"
    @reinstall="emit('reinstall', $event)"
  />

  <LazyUiServersPlatformChangeModpackVersionModal
    ref="modpackVersionModal"
    :server="props.server"
    :project="data?.project"
    :versions="Array.isArray(versions) ? versions : []"
    :current-version="currentVersion"
    :current-version-id="data?.upstream?.version_id"
    :server-status="data?.status"
    @reinstall="emit('reinstall')"
  />

  <div class="flex h-full w-full flex-col">
    <div v-if="data && versions" class="flex w-full flex-col">
      <div class="card flex flex-col gap-4">
        <div class="flex flex-row items-center justify-between gap-2">
          <h2 class="m-0 text-lg font-bold text-contrast">Modpack</h2>
          <div v-if="data.upstream" class="flex gap-4">
            <ButtonStyled>
              <button
                class="!w-full sm:!w-auto"
                :disabled="isInstalling"
                @click="mrpackModal.show()"
              >
                <UploadIcon class="size-4" /> Import .mrpack
              </button>
            </ButtonStyled>
            <!-- dumb hack to make a button link not a link -->
            <ButtonStyled :disabled="isInstalling">
              <template v-if="isInstalling">
                <button class="cursor-not-allowed opacity-50">
                  <TransferIcon class="size-4" />
                  Switch modpack
                </button>
              </template>
              <nuxt-link v-else :to="`/modpacks?sid=${props.server.serverId}`">
                <TransferIcon class="size-4" />
                Switch modpack
              </nuxt-link>
            </ButtonStyled>
          </div>
        </div>
        <div v-if="data.upstream" class="flex flex-col gap-2">
          <!-- <div v-if="hasNewerVersion" class="text-brand">
            {{ data.project?.title }} has a newer version available! from
            {{ currentVersion?.version_number }} to {{ latestVersion?.version_number }}
          </div> -->
          <NewProjectCard
            class="!bg-bg"
            :project="{
              icon_url: data.project?.icon_url,
              title: data.project?.title,
              description: data.project?.description,
              downloads: data.project?.downloads,
              follows: data.project?.followers,
              date_modified: data.project?.updated,
            }"
            :categories="data.project?.categories || []"
          >
            <template #actions>
              <ButtonStyled color="brand">
                <button :disabled="isInstalling" @click="modpackVersionModal.show()">
                  <SettingsIcon class="size-4" />
                  Change version
                </button>
              </ButtonStyled>
            </template>
          </NewProjectCard>
        </div>
        <div v-else class="flex w-full flex-col items-center gap-2 sm:w-fit sm:flex-row">
          <ButtonStyled>
            <nuxt-link class="!w-full sm:!w-auto" :to="`/modpacks?sid=${props.server.serverId}`">
              <CompassIcon class="size-4" /> Find a modpack
            </nuxt-link>
          </ButtonStyled>
          <span class="hidden sm:block">or</span>
          <ButtonStyled>
            <button class="!w-full sm:!w-auto" @click="mrpackModal.show()">
              <UploadIcon class="size-4" /> Upload .mrpack file
            </button>
          </ButtonStyled>
        </div>
      </div>

      <div class="card flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <h2 class="m-0 text-lg font-bold text-contrast">Platform</h2>
          <p class="m-0">Your server's platform is the software that runs your mods and plugins.</p>
          <div v-if="data.upstream" class="mt-2 flex items-center gap-2">
            <InfoIcon class="hidden sm:block" />
            <span class="text-sm text-secondary">
              The current platform was automatically selected based on your modpack.
            </span>
          </div>
        </div>
        <div
          class="flex w-full flex-col gap-1 rounded-2xl"
          :class="{
            'pointer-events-none cursor-not-allowed select-none opacity-50':
              props.server.general?.status === 'installing',
          }"
          :tabindex="props.server.general?.status === 'installing' ? -1 : 0"
        >
          <UiServersLoaderSelector
            :data="data"
            :is-installing="isInstalling"
            @select-loader="selectLoader"
          />
        </div>
      </div>
    </div>

    <div v-else />
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled, NewProjectCard } from "@modrinth/ui";
import { TransferIcon, UploadIcon, InfoIcon, CompassIcon, SettingsIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";
import type { Loaders } from "~/types/servers";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();

const isInstalling = computed(() => props.server.general?.status === "installing");

const versionSelectModal = ref();
const mrpackModal = ref();
const modpackVersionModal = ref();

const data = computed(() => props.server.general);

const { data: versions } = await useLazyAsyncData(
  `content-loader-versions-${data.value?.upstream?.project_id}`,
  async () => {
    if (!data.value?.upstream?.project_id) return [];
    const result = await useBaseFetch(`project/${data.value.upstream.project_id}/version`);
    return (result || []) as any[];
  },
);

const currentVersion = ref<any | null>(null);

const updateData = async () => {
  if (!data.value?.upstream?.version_id) {
    currentVersion.value = null;
    return;
  }
  const result = await useBaseFetch(`version/${data.value.upstream.version_id}`);
  currentVersion.value = result as any;
};

updateData();

watch(() => props.server.general?.upstream?.version_id, updateData);

// const latestVersion = computed(() => {
//   if (!Array.isArray(versions?.value) || versions.value.length === 0) return null;
//   return versions.value.reduce((latest: any, current: any) => {
//     if (!latest) return current;
//     return latest.version_number > current.version_number ? latest : current;
//   }, null);
// });

// const hasNewerVersion = computed(() => {
//   if (!currentVersion.value?.version_number || !latestVersion.value?.version_number) return false;
//   return latestVersion.value.version_number > currentVersion.value.version_number;
// });

// const handleUpdateToLatest = () => {
//   modpackVersionModal.value?.show();
// };

const selectLoader = (loader: string) => {
  versionSelectModal.value?.show(loader as Loaders);
};
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}

.button-base:active {
  scale: none !important;
}
</style>
