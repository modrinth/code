<template>
  <LazyUiServersPlatformVersionSelectModal
    ref="versionSelectModal"
    :server="props.server"
    :current-loader="data?.loader as Loaders"
    :backup-in-progress="backupInProgress"
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
        <div class="flex select-none flex-col items-center justify-between gap-2 lg:flex-row">
          <div class="flex flex-row items-center gap-2">
            <h2 class="m-0 text-lg font-bold text-contrast">Modpack</h2>
            <div
              v-if="updateAvailable"
              class="rounded-full bg-bg-orange px-2 py-1 text-xs font-medium text-orange"
            >
              <span>Update available</span>
            </div>
          </div>
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
            <ButtonStyled>
              <template v-if="isInstalling">
                <button :disabled="isInstalling">
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
          <div
            v-if="versionsError || currentVersionError"
            class="rounded-2xl border border-solid border-red p-4 text-contrast"
          >
            <p class="m-0 font-bold">Something went wrong while loading your modpack.</p>
            <p class="m-0 mb-2 mt-1 text-sm">
              {{ versionsError || currentVersionError }}
            </p>
            <ButtonStyled>
              <button :disabled="isInstalling" @click="refreshData">Retry</button>
            </ButtonStyled>
          </div>

          <NewProjectCard
            v-if="!versionsError && !currentVersionError"
            class="!cursor-default !bg-bg !filter-none"
            :project="projectCardData"
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
            <nuxt-link
              v-tooltip="backupInProgress ? formatMessage(backupInProgress.tooltip) : undefined"
              :class="{ disabled: backupInProgress }"
              class="!w-full sm:!w-auto"
              :to="`/modpacks?sid=${props.server.serverId}`"
            >
              <CompassIcon class="size-4" /> Find a modpack
            </nuxt-link>
          </ButtonStyled>
          <span class="hidden sm:block">or</span>
          <ButtonStyled>
            <button
              v-tooltip="backupInProgress ? formatMessage(backupInProgress.tooltip) : undefined"
              :disabled="!!backupInProgress"
              class="!w-full sm:!w-auto"
              @click="mrpackModal.show()"
            >
              <UploadIcon class="size-4" /> Upload .mrpack file
            </button>
          </ButtonStyled>
        </div>
      </div>

      <div class="card flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <h2 class="m-0 text-lg font-bold text-contrast">Platform</h2>
          <p class="m-0">Your server's platform is the software that runs mods and plugins.</p>
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
import type { BackupInProgressReason } from "~/pages/servers/manage/[id].vue";

const { formatMessage } = useVIntl();

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
  backupInProgress?: BackupInProgressReason;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();

const isInstalling = computed(() => props.server.general?.status === "installing");

const versionSelectModal = ref();
const mrpackModal = ref();
const modpackVersionModal = ref();

const data = computed(() => props.server.general);

const {
  data: versions,
  error: versionsError,
  refresh: refreshVersions,
} = await useAsyncData(
  `content-loader-versions-${data.value?.upstream?.project_id}`,
  async () => {
    if (!data.value?.upstream?.project_id) return [];
    try {
      const result = await useBaseFetch(`project/${data.value.upstream.project_id}/version`);
      return result || [];
    } catch (e) {
      console.error("couldnt fetch all versions:", e);
      throw new Error("Failed to load modpack versions.");
    }
  },
  { default: () => [] },
);

const {
  data: currentVersion,
  error: currentVersionError,
  refresh: refreshCurrentVersion,
} = await useAsyncData(
  `content-loader-version-${data.value?.upstream?.version_id}`,
  async () => {
    if (!data.value?.upstream?.version_id) return null;
    try {
      const result = await useBaseFetch(`version/${data.value.upstream.version_id}`);
      return result || null;
    } catch (e) {
      console.error("couldnt fetch version:", e);
      throw new Error("Failed to load modpack version.");
    }
  },
  { default: () => null },
);

const projectCardData = computed(() => ({
  icon_url: data.value?.project?.icon_url,
  title: data.value?.project?.title,
  description: data.value?.project?.description,
  downloads: data.value?.project?.downloads,
  follows: data.value?.project?.followers,
  // @ts-ignore
  date_modified: currentVersion.value?.date_published || data.value?.project?.updated,
}));

const selectLoader = (loader: string) => {
  versionSelectModal.value?.show(loader as Loaders);
};

const refreshData = async () => {
  await Promise.all([refreshVersions(), refreshCurrentVersion()]);
};

const updateAvailable = computed(() => {
  // so sorry
  // @ts-ignore
  if (!data.value?.upstream || !versions.value?.length || !currentVersion.value) {
    return false;
  }

  // @ts-ignore
  const latestVersion = versions.value[0];
  // @ts-ignore
  return latestVersion.id !== currentVersion.value.id;
});

watch(
  () => props.server.general?.status,
  async (newStatus, oldStatus) => {
    if (oldStatus === "installing" && newStatus === "available") {
      await Promise.all([
        refreshVersions(),
        refreshCurrentVersion(),
        props.server.refresh(["general"]),
      ]);
    }
  },
);
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}

.button-base:active {
  scale: none !important;
}
</style>
