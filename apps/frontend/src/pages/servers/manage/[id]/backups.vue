<template>
  <div
    v-if="server.backups?.error"
    class="flex w-full flex-col items-center justify-center gap-4 p-4"
  >
    <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
      <div class="flex flex-col items-center text-center">
        <div class="flex flex-col items-center gap-4">
          <div class="grid place-content-center rounded-full bg-bg-orange p-4">
            <IssuesIcon class="size-12 text-orange" />
          </div>
          <h1 class="m-0 mb-2 w-fit text-4xl font-bold">Failed to load backups</h1>
        </div>
        <p class="text-lg text-secondary">
          We couldn't load your server's backups. Here's what went wrong:
        </p>
        <p>
          <span class="break-all font-mono">{{ JSON.stringify(server.backups.error) }}</span>
        </p>
        <ButtonStyled size="large" color="brand" @click="() => server.refresh(['backups'])">
          <button class="mt-6 !w-full">Retry</button>
        </ButtonStyled>
      </div>
    </div>
  </div>
  <div v-else-if="data" class="contents">
    <BackupCreateModal ref="createBackupModal" :server="server" />
    <BackupRenameModal ref="renameBackupModal" :server="server" />
    <BackupRestoreModal ref="restoreBackupModal" :server="server" />
    <BackupDeleteModal ref="deleteBackupModal" :server="server" @delete="deleteBackup" />
    <BackupSettingsModal ref="backupSettingsModal" :server="server" />

    <div class="mb-6 flex flex-col items-center justify-between gap-4 sm:flex-row">
      <div class="flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <h1 class="m-0 text-2xl font-extrabold text-contrast">Backups</h1>
          <TagItem
            v-tooltip="`${data.backup_quota - data.used_backup_quota} backup slots remaining`"
            class="cursor-help"
            :style="{
              '--_color':
                data.backup_quota <= data.used_backup_quota
                  ? 'var(--color-red)'
                  : data.backup_quota - data.used_backup_quota <= 3
                    ? 'var(--color-orange)'
                    : undefined,
              '--_bg-color':
                data.backup_quota <= data.used_backup_quota
                  ? 'var(--color-red-bg)'
                  : data.backup_quota - data.used_backup_quota <= 3
                    ? 'var(--color-orange-bg)'
                    : undefined,
            }"
          >
            {{ data.used_backup_quota }} / {{ data.backup_quota }}
          </TagItem>
        </div>
        <p class="m-0">
          You can have up to {{ data.backup_quota }} backups at once, stored securely off-site.
        </p>
      </div>
      <div
        class="grid w-full grid-cols-[repeat(auto-fit,_minmax(180px,1fr))] gap-2 sm:flex sm:w-fit sm:flex-row"
      >
        <ButtonStyled type="standard">
          <button
            v-tooltip="
              'Auto backups are currently unavailable; we apologize for the inconvenience.'
            "
            :disabled="true || server.general?.status === 'installing'"
            @click="showbackupSettingsModal"
          >
            <SettingsIcon class="h-5 w-5" />
            Auto backups
          </button>
        </ButtonStyled>
        <ButtonStyled type="standard" color="brand">
          <button
            v-tooltip="backupCreationDisabled"
            class="w-full sm:w-fit"
            :disabled="!!backupCreationDisabled"
            @click="showCreateModel"
          >
            <PlusIcon class="h-5 w-5" />
            Create backup
          </button>
        </ButtonStyled>
      </div>
    </div>

    <div class="flex w-full flex-col gap-2">
      <div
        v-if="backups.length === 0"
        class="mt-6 flex items-center justify-center gap-2 text-center text-secondary"
      >
        <template v-if="data.used_backup_quota">
          <SpinnerIcon class="animate-spin" />
          Loading backups...
        </template>
        <template v-else> You don't have any backups yet. </template>
      </div>
      <BackupItem
        v-for="backup in backups"
        :key="`backup-${backup.id}`"
        :backup="backup"
        :kyros-url="props.server.general?.node.instance"
        :jwt="props.server.general?.node.token"
        @prepare="() => prepareDownload(backup.id)"
        @download="() => triggerDownloadAnimation()"
        @rename="() => renameBackupModal?.show(backup)"
        @restore="() => restoreBackupModal?.show(backup)"
        @lock="
          () => {
            if (backup.locked) {
              unlockBackup(backup.id);
            } else {
              lockBackup(backup.id);
            }
          }
        "
        @delete="
          (skipConfirmation?: boolean) =>
            !skipConfirmation ? deleteBackup(backup) : deleteBackupModal?.show(backup)
        "
        @retry="() => retryBackup(backup.id)"
      />
    </div>

    <div
      class="over-the-top-download-animation"
      :class="{ 'animation-hidden': !overTheTopDownloadAnimation }"
    >
      <div>
        <div
          class="animation-ring-3 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-40"
        ></div>
        <div
          class="animation-ring-2 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-60"
        ></div>
        <div
          class="animation-ring-1 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight"
        >
          <DownloadIcon class="h-20 w-20 text-contrast" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled, TagItem } from "@modrinth/ui";
import { useStorage } from "@vueuse/core";
import { SpinnerIcon, PlusIcon, DownloadIcon, SettingsIcon, IssuesIcon } from "@modrinth/assets";
import { ref, computed } from "vue";
import type { Server } from "~/composables/pyroServers";
import BackupItem from "~/components/ui/servers/BackupItem.vue";
import BackupRenameModal from "~/components/ui/servers/BackupRenameModal.vue";
import BackupCreateModal from "~/components/ui/servers/BackupCreateModal.vue";
import BackupRestoreModal from "~/components/ui/servers/BackupRestoreModal.vue";
import BackupDeleteModal from "~/components/ui/servers/BackupDeleteModal.vue";
import BackupSettingsModal from "~/components/ui/servers/BackupSettingsModal.vue";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
  isServerRunning: boolean;
}>();

const route = useNativeRoute();
const serverId = route.params.id;

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
  backupWhileRunning: false,
});

defineEmits(["onDownload"]);

const data = computed(() => props.server.general);
const backups = computed(() => {
  if (!props.server.backups?.data) return [];

  return [...props.server.backups.data].sort((a, b) => {
    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
  });
});

useHead({
  title: `Backups - ${data.value?.name ?? "Server"} - Modrinth`,
});

const overTheTopDownloadAnimation = ref();

const createBackupModal = ref<InstanceType<typeof BackupCreateModal>>();
const renameBackupModal = ref<InstanceType<typeof BackupRenameModal>>();
const restoreBackupModal = ref<InstanceType<typeof BackupRestoreModal>>();
const deleteBackupModal = ref<InstanceType<typeof BackupDeleteModal>>();
const backupSettingsModal = ref<InstanceType<typeof BackupSettingsModal>>();

const backupCreationDisabled = computed(() => {
  if (props.isServerRunning && !userPreferences.value.backupWhileRunning) {
    return "Cannot create backup while server is running";
  }
  if (
    data.value?.used_backup_quota !== undefined &&
    data.value?.backup_quota !== undefined &&
    data.value?.used_backup_quota >= data.value?.backup_quota
  ) {
    return `All ${data.value.backup_quota} of your backup slots are in use`;
  }
  if (backups.value.some((backup) => backup.task?.create?.state === "ongoing")) {
    return "A backup is already in progress";
  }
  if (props.server.general?.status === "installing") {
    return "Cannot create backup while server is installing";
  }
  return undefined;
});

const showCreateModel = () => {
  createBackupModal.value?.show();
};

const showbackupSettingsModal = () => {
  backupSettingsModal.value?.show();
};

function triggerDownloadAnimation() {
  overTheTopDownloadAnimation.value = true;
  setTimeout(() => (overTheTopDownloadAnimation.value = false), 500);
}

const prepareDownload = async (backupId: string) => {
  try {
    await props.server.backups?.prepare(backupId);
  } catch (error) {
    console.error("Failed to prepare download:", error);
    addNotification({ type: "error", title: "Failed to prepare backup for download", text: error });
  }
};

const lockBackup = async (backupId: string) => {
  try {
    await props.server.backups?.lock(backupId);
    await props.server.refresh(["backups"]);
  } catch (error) {
    console.error("Failed to toggle lock:", error);
  }
};

const unlockBackup = async (backupId: string) => {
  try {
    await props.server.backups?.unlock(backupId);
    await props.server.refresh(["backups"]);
  } catch (error) {
    console.error("Failed to toggle lock:", error);
  }
};

const retryBackup = async (backupId: string) => {
  try {
    await props.server.backups?.retry(backupId);
    await props.server.refresh(["backups"]);
  } catch (error) {
    console.error("Failed to retry backup:", error);
  }
};

async function deleteBackup(backup?: Backup) {
  if (!backup) {
    addNotification({
      type: "error",
      title: "Error deleting backup",
      text: "Backup is null",
    });
    return;
  }

  try {
    await props.server.backups?.delete(backup.id);
    await props.server.refresh();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    addNotification({
      type: "error",
      title: "Error deleting backup",
      text: message,
    });
  }
}
</script>

<style scoped>
.over-the-top-download-animation {
  position: fixed;
  z-index: 100;
  inset: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  pointer-events: none;
  scale: 0.5;
  transition: all 0.5s ease-out;
  opacity: 1;

  &.animation-hidden {
    scale: 0.8;
    opacity: 0;

    .animation-ring-1 {
      width: 25rem;
      height: 25rem;
    }
    .animation-ring-2 {
      width: 50rem;
      height: 50rem;
    }
    .animation-ring-3 {
      width: 100rem;
      height: 100rem;
    }
  }

  > div {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    width: fit-content;
    height: fit-content;

    > * {
      position: absolute;
      scale: 1;
      transition: all 0.2s ease-out;
      width: 20rem;
      height: 20rem;
    }
  }
}
</style>
