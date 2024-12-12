<template>
  <div class="contents">
    <div v-if="data" class="contents">
      <LazyUiServersBackupCreateModal
        ref="createBackupModal"
        :server="server"
        @backup-created="handleBackupCreated"
      />
      <LazyUiServersBackupRenameModal
        ref="renameBackupModal"
        :server="server"
        :current-backup-id="currentBackup"
        :backup-name="renameBackupName"
        @backup-renamed="handleBackupRenamed"
      />
      <LazyUiServersBackupRestoreModal
        ref="restoreBackupModal"
        :server="server"
        :backup-id="currentBackup"
        :backup-name="currentBackupDetails?.name ?? ''"
        :backup-created-at="currentBackupDetails?.created_at ?? ''"
        @backup-restored="handleBackupRestored"
      />
      <LazyUiServersBackupDeleteModal
        ref="deleteBackupModal"
        :server="server"
        :backup-id="currentBackup"
        :backup-name="currentBackupDetails?.name ?? ''"
        :backup-created-at="currentBackupDetails?.created_at ?? ''"
        @backup-deleted="handleBackupDeleted"
      />

      <LazyUiServersBackupSettingsModal ref="backupSettingsModal" :server="server" />

      <ul class="m-0 flex list-none flex-col gap-4 p-0">
        <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-6 shadow-md">
          <div class="flex flex-col items-center justify-between gap-4 sm:flex-row sm:gap-0">
            <div class="flex flex-col items-baseline gap-2">
              <div class="text-2xl font-bold text-contrast">
                {{
                  data.used_backup_quota === 0
                    ? "No backups"
                    : `You've created ${data.used_backup_quota} backup${data.used_backup_quota === 1 ? "" : "s"}`
                }}
              </div>
              <div>
                {{
                  data.backup_quota - data.used_backup_quota === 0
                    ? "You have reached your backup limit. Consider removing old backups to create new ones."
                    : `You can create ${data.backup_quota - data.used_backup_quota} more backups for your server.`
                }}
              </div>
            </div>
            <div class="flex w-full flex-col gap-2 sm:w-fit sm:flex-row">
              <ButtonStyled type="standard">
                <button @click="showbackupSettingsModal">
                  <SettingsIcon class="h-5 w-5" />
                  Auto backups
                </button>
              </ButtonStyled>
              <ButtonStyled type="standard" color="brand">
                <button
                  v-tooltip="
                    isServerRunning && !userPreferences.backupWhileRunning
                      ? 'Cannot create backup while server is running. You can disable this from your server Options > Preferences.'
                      : ''
                  "
                  class="w-full sm:w-fit"
                  :disabled="
                    (isServerRunning && !userPreferences.backupWhileRunning) ||
                    data.used_backup_quota >= data.backup_quota ||
                    backups.some((backup) => backup.ongoing)
                  "
                  @click="showCreateModel"
                >
                  <PlusIcon class="h-5 w-5" />
                  Create backup
                </button>
              </ButtonStyled>
            </div>
          </div>
        </div>

        <div
          v-if="backups.some((backup) => backup.ongoing)"
          data-pyro-server-backup-ongoing
          class="flex w-full flex-row items-center gap-4 rounded-2xl bg-bg-orange p-4 text-contrast"
        >
          A backup is currently being created. This may take a few minutes. This page will
          automatically refresh when the backup is complete.
        </div>

        <li
          v-for="(backup, index) in backups"
          :key="backup.id"
          class="relative m-0 w-full list-none rounded-2xl bg-bg-raised p-4 shadow-md"
        >
          <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
              <div class="flex min-w-0 flex-row items-center gap-4">
                <div
                  class="grid size-14 shrink-0 place-content-center overflow-hidden rounded-xl border-[1px] border-solid border-button-border shadow-sm"
                  :class="backup.ongoing ? 'text-green [&&]:bg-bg-green' : 'bg-button-bg'"
                >
                  <UiServersIconsLoadingIcon
                    v-if="backup.ongoing"
                    v-tooltip="'Backup in progress'"
                    class="size-6 animate-spin"
                  />
                  <LockIcon v-else-if="backup.locked" class="size-8" />
                  <BoxIcon v-else class="size-8" />
                </div>
                <div class="flex min-w-0 flex-col gap-2">
                  <div class="flex min-w-0 flex-col gap-2 sm:flex-row sm:items-center">
                    <div class="max-w-full truncate text-xl font-bold text-contrast">
                      {{ backup.name }}
                    </div>

                    <div
                      v-if="index == 0"
                      class="hidden items-center gap-1 rounded-full bg-bg-green p-1 px-1.5 text-xs font-semibold text-brand sm:flex"
                    >
                      <CheckIcon class="size-4" /> Latest
                    </div>
                  </div>
                  <div class="flex items-center gap-2 text-sm font-semibold">
                    <CalendarIcon class="size-4" />
                    {{
                      new Date(backup.created_at).toLocaleString("en-US", {
                        month: "numeric",
                        day: "numeric",
                        year: "2-digit",
                        hour: "numeric",
                        minute: "numeric",
                        hour12: true,
                      })
                    }}
                  </div>
                </div>
              </div>
              <ButtonStyled v-if="!backup.ongoing" circular type="transparent">
                <UiServersTeleportOverflowMenu
                  direction="left"
                  position="bottom"
                  class="bg-transparent"
                  :options="[
                    {
                      id: 'rename',
                      action: () => {
                        renameBackupName = backup.name;
                        currentBackup = backup.id;
                        renameBackupModal?.show();
                      },
                    },
                    {
                      id: 'restore',
                      action: () => {
                        currentBackup = backup.id;
                        restoreBackupModal?.show();
                      },
                    },
                    { id: 'download', action: () => initiateDownload(backup.id) },
                    {
                      id: 'lock',
                      action: () => {
                        if (backup.locked) {
                          unlockBackup(backup.id);
                        } else {
                          lockBackup(backup.id);
                        }
                      },
                    },
                    {
                      id: 'delete',
                      action: () => {
                        currentBackup = backup.id;
                        deleteBackupModal?.show();
                      },
                      color: 'red',
                    },
                  ]"
                >
                  <MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
                  <template #rename> <EditIcon /> Rename </template>
                  <template #restore> <ClipboardCopyIcon /> Restore </template>
                  <template v-if="backup.locked" #lock> <LockOpenIcon /> Unlock </template>
                  <template v-else #lock> <LockIcon /> Lock </template>
                  <template #download> <DownloadIcon /> Download </template>
                  <template #delete> <TrashIcon /> Delete </template>
                </UiServersTeleportOverflowMenu>
              </ButtonStyled>
            </div>
          </div>
        </li>
      </ul>

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
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { useStorage } from "@vueuse/core";
import {
  PlusIcon,
  CheckIcon,
  CalendarIcon,
  MoreHorizontalIcon,
  EditIcon,
  ClipboardCopyIcon,
  DownloadIcon,
  TrashIcon,
  SettingsIcon,
  BoxIcon,
  LockIcon,
  LockOpenIcon,
} from "@modrinth/assets";
import { ref, computed } from "vue";
import type { Server } from "~/composables/pyroServers";

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

const createBackupModal = ref<typeof NewModal>();
const renameBackupModal = ref<typeof NewModal>();
const restoreBackupModal = ref<typeof NewModal>();
const deleteBackupModal = ref<typeof NewModal>();
const backupSettingsModal = ref<typeof NewModal>();

const renameBackupName = ref("");
const currentBackup = ref("");

const refreshInterval = ref<ReturnType<typeof setInterval>>();

const currentBackupDetails = computed(() => {
  return backups.value.find((backup) => backup.id === currentBackup.value);
});

const showCreateModel = () => {
  createBackupModal.value?.show();
};

const showbackupSettingsModal = () => {
  backupSettingsModal.value?.show();
};

const handleBackupCreated = (payload: { success: boolean; message: string }) => {
  if (payload.success) {
    addNotification({ type: "success", text: payload.message });
  } else {
    addNotification({ type: "error", text: payload.message });
  }
};

const handleBackupRenamed = (payload: { success: boolean; message: string }) => {
  if (payload.success) {
    addNotification({ type: "success", text: payload.message });
  } else {
    addNotification({ type: "error", text: payload.message });
  }
};

const handleBackupRestored = (payload: { success: boolean; message: string }) => {
  if (payload.success) {
    addNotification({ type: "success", text: payload.message });
  } else {
    addNotification({ type: "error", text: payload.message });
  }
};

const handleBackupDeleted = (payload: { success: boolean; message: string }) => {
  if (payload.success) {
    addNotification({ type: "success", text: payload.message });
  } else {
    addNotification({ type: "error", text: payload.message });
  }
};

function triggerDownloadAnimation() {
  overTheTopDownloadAnimation.value = true;
  setTimeout(() => (overTheTopDownloadAnimation.value = false), 500);
}

const initiateDownload = async (backupId: string) => {
  triggerDownloadAnimation();

  try {
    const downloadurl: any = await props.server.backups?.download(backupId);
    if (!downloadurl || !downloadurl.download_url) {
      throw new Error("Invalid download URL.");
    }

    let finalDownloadUrl: string = downloadurl.download_url;

    if (!/^https?:\/\//i.test(finalDownloadUrl)) {
      finalDownloadUrl = `https://${finalDownloadUrl.startsWith("/") ? finalDownloadUrl.substring(1) : finalDownloadUrl}`;
    }

    const a = document.createElement("a");
    a.href = finalDownloadUrl;
    a.setAttribute("download", "");
    a.click();
    a.remove();
  } catch (error) {
    console.error("Download failed:", error);
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

onMounted(() => {
  watchEffect(() => {
    const hasOngoingBackups = backups.value.some((backup) => backup.ongoing);

    if (refreshInterval.value) {
      clearInterval(refreshInterval.value);
      refreshInterval.value = undefined;
    }

    if (hasOngoingBackups) {
      refreshInterval.value = setInterval(() => {
        props.server.refresh(["backups"]);
      }, 10000);
    }
  });
});

onUnmounted(() => {
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value);
  }
});
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
