<template>
  <div class="contents">
    <div v-if="data">
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

      <NewModal ref="createBackupModal" header="Creating backup" @show="focusCreateInput">
        <div class="flex flex-col gap-2 md:w-[600px]">
          <div class="font-semibold text-contrast">Name</div>
          <input
            ref="createBackupInput"
            v-model="createBackupName"
            type="text"
            class="bg-bg-input w-full rounded-lg p-4"
            placeholder="e.g. Before 1.21"
          />
        </div>
        <div class="mb-1 mt-4 flex justify-start gap-4">
          <ButtonStyled color="brand">
            <button :disabled="isCreatingBackup" @click="createBackup">
              <PlusIcon />
              Create backup
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button @click="createBackupModal?.hide()">
              <XIcon />
              Cancel
            </button>
          </ButtonStyled>
        </div>
      </NewModal>

      <NewModal ref="renameBackupModal" header="Renaming backup" @show="focusRenameInput">
        <div class="flex flex-col gap-2 md:w-[600px]">
          <div class="font-semibold text-contrast">Name</div>
          <input
            ref="renameBackupInput"
            v-model="renameBackupName"
            type="text"
            class="bg-bg-input w-full rounded-lg p-4"
            placeholder="e.g. Before 1.21"
          />
        </div>
        <div class="mb-1 mt-4 flex justify-start gap-4">
          <ButtonStyled color="brand">
            <button :disabled="isRenamingBackup" @click="renameBackup">
              <SaveIcon />
              Rename backup
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button @click="renameBackupModal?.hide()">
              <XIcon />
              Cancel
            </button>
          </ButtonStyled>
        </div>
      </NewModal>

      <NewModal ref="restoreBackupModal" header="Restoring backup">
        <div class="flex flex-col gap-4">
          <div class="relative flex w-full flex-col gap-2 rounded-2xl bg-bg p-6">
            <div class="text-2xl font-extrabold text-contrast">
              {{ backups?.find((b) => b.id === currentBackup)?.name }}
            </div>
            <div class="flex gap-2 font-semibold text-contrast">
              <CalendarIcon />
              {{
                new Date(
                  backups?.find((b: any) => b.id === currentBackup)?.created_at || "",
                ).toLocaleString()
              }}
            </div>
          </div>
        </div>
        <div class="mb-1 mt-4 flex justify-end gap-4">
          <ButtonStyled color="brand">
            <button :disabled="isRestoringBackup" @click="restoreBackup">Restore backup</button>
          </ButtonStyled>
          <ButtonStyled type="transparent">
            <button @click="restoreBackupModal?.hide()">Cancel</button>
          </ButtonStyled>
        </div>
      </NewModal>

      <NewModal ref="deleteBackupModal" danger header="Deleting backup">
        <div class="flex flex-col gap-4">
          <div class="relative flex w-full flex-col gap-2 rounded-2xl bg-[#0e0e0ea4] p-6">
            <div class="text-2xl font-extrabold text-contrast">
              {{ backups?.find((b) => b.id === currentBackup)?.name }}
            </div>
            <div class="flex gap-2 font-semibold text-contrast">
              <CalendarIcon />
              {{
                new Date(
                  backups?.find((b: any) => b.id === currentBackup)?.created_at || "",
                ).toLocaleString()
              }}
            </div>
          </div>
        </div>
        <div class="mb-1 mt-4 flex justify-end gap-4">
          <ButtonStyled color="red">
            <button :disabled="isDeletingBackup" @click="deleteBackup">
              <TrashIcon />
              Delete backup
            </button>
          </ButtonStyled>
          <ButtonStyled type="transparent">
            <button @click="deleteBackupModal?.hide()">Cancel</button>
          </ButtonStyled>
        </div>
      </NewModal>

      <UiServersBackupSettingsModal ref="backupSettingsModal" :server="server" />

      <ul class="m-0 flex list-none flex-col gap-6 p-0">
        <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-6">
          <div class="flex flex-col items-center justify-between gap-4 sm:flex-row sm:gap-0">
            <div class="flex items-baseline gap-2 sm:flex-col">
              <div class="text-2xl font-extrabold text-contrast">
                {{ data.used_backup_quota }} Backup{{ data.used_backup_quota > 1 ? "s" : "" }}
              </div>
              <div class="">({{ data.backup_quota - data.used_backup_quota }} slots available)</div>
            </div>
            <div class="flex w-full gap-2 sm:w-fit">
              <ButtonStyled type="standard" color="brand">
                <button
                  v-tooltip="
                    isServerRunning && !userPreferences.backupWhileRunning
                      ? 'Cannot create backup while server is running. You can disable this from your server Options > Preferences.'
                      : ''
                  "
                  class="w-full sm:w-fit"
                  :disabled="isServerRunning && !userPreferences.backupWhileRunning"
                  @click="showCreateModel"
                >
                  <PlusIcon class="h-5 w-5" />
                  Create backup
                </button>
              </ButtonStyled>
              <ButtonStyled type="standard">
                <button @click="showbackupSettingsModal">
                  <SettingsIcon class="h-5 w-5" />
                </button>
              </ButtonStyled>
            </div>
          </div>
        </div>

        <li
          v-for="(backup, index) in backups"
          :key="backup.id"
          class="relative m-0 w-full list-none rounded-2xl bg-bg-raised px-6 py-4"
        >
          <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
              <div class="flex flex-col gap-2">
                <div class="flex items-center gap-2">
                  <div class="text-xl font-bold text-contrast">{{ backup.name }}</div>
                  <div v-if="index == 0" class="flex gap-2 font-bold text-brand">
                    <CheckIcon class="h-5 w-5" /> Latest
                  </div>
                </div>
                <div class="flex items-center gap-2 text-sm font-semibold text-contrast">
                  <CalendarIcon /> {{ new Date(backup.created_at).toLocaleString() }}
                </div>
              </div>
              <OverflowMenu
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
                    id: 'delete',
                    action: () => {
                      currentBackup = backup.id;
                      deleteBackupModal?.show();
                    },
                    color: 'red',
                  },
                ]"
                direction="right"
                class="bg-transparent"
              >
                <MoreHorizontalIcon class="h-5 w-5 bg-transparent text-contrast" />

                <template #rename> <EditIcon /> Rename </template>
                <template #restore> <ClipboardCopyIcon /> Restore </template>
                <template #download> <DownloadIcon /> Download </template>
                <template #delete> <TrashIcon /> Delete </template>
              </OverflowMenu>
            </div>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled, OverflowMenu, NewModal } from "@modrinth/ui";
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
  XIcon,
  SaveIcon,
} from "@modrinth/assets";
import { ref, computed } from "vue";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
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

const backupError = ref<string | null>(null);

const overTheTopDownloadAnimation = ref();

const createBackupModal = ref<typeof NewModal>();
const renameBackupModal = ref<typeof NewModal>();
const restoreBackupModal = ref<typeof NewModal>();
const deleteBackupModal = ref<typeof NewModal>();
const backupSettingsModal = ref<typeof NewModal>();

const createBackupInput = ref<HTMLInputElement>();
const renameBackupInput = ref<HTMLInputElement>();

const createBackupName = ref("");
const renameBackupName = ref("");
const currentBackup = ref("");

const isCreatingBackup = ref(false);
const isRenamingBackup = ref(false);
const isRestoringBackup = ref(false);
const isDeletingBackup = ref(false);

const focusCreateInput = () => {
  nextTick(() => {
    setTimeout(() => {
      createBackupInput.value?.focus();
    }, 100);
  });
};

const focusRenameInput = () => {
  nextTick(() => {
    setTimeout(() => {
      renameBackupInput.value?.focus();
    }, 100);
  });
};

const showCreateModel = () => {
  createBackupModal.value?.show();
};

const showbackupSettingsModal = () => {
  backupSettingsModal.value?.show();
};

const createBackup = async () => {
  if (!createBackupName.value.trim()) {
    addNotification({
      group: "server",
      title: "Error",
      text: "Backup name cannot be empty",
      type: "error",
    });
    return;
  }

  isCreatingBackup.value = true;
  try {
    await props.server.backups?.create(createBackupName.value);
    await props.server.refresh();
    createBackupModal.value?.hide();
    addNotification({
      group: "server",
      title: "Backup created",
      text: "Your backup has been created successfully.",
      type: "success",
    });
  } catch (error) {
    if (error instanceof PyroFetchError && error.statusCode === 429) {
      addNotification({
        group: "server",
        title: "Error creating backup",
        text: "Please wait a few moments before creating another backup.",
        type: "error",
      });
    } else {
      backupError.value = error instanceof Error ? error.message : String(error);
      addNotification({
        group: "server",
        title: "Error creating backup",
        text: backupError.value,
        type: "error",
      });
    }
  } finally {
    isCreatingBackup.value = false;
  }
};

const renameBackup = async () => {
  if (!renameBackupName.value.trim() || !currentBackup.value) {
    addNotification({
      group: "server",
      title: "Error",
      text: "Backup name cannot be empty",
      type: "error",
    });
    return;
  }

  isRenamingBackup.value = true;
  try {
    await props.server.backups?.rename(currentBackup.value, renameBackupName.value);
    await props.server.refresh();
    renameBackupModal.value?.hide();
    addNotification({
      group: "server",
      title: "Backup renamed",
      text: "Your backup has been renamed successfully.",
      type: "success",
    });
  } catch (error) {
    backupError.value = error instanceof Error ? error.message : String(error);
    addNotification({
      group: "server",
      title: "Error renaming backup",
      text: backupError.value,
      type: "error",
    });
  } finally {
    isRenamingBackup.value = false;
  }
};

const restoreBackup = async () => {
  if (!currentBackup.value) {
    addNotification({
      group: "server",
      title: "Error",
      text: "No backup selected",
      type: "error",
    });
    return;
  }

  isRestoringBackup.value = true;
  try {
    await props.server.backups?.restore(currentBackup.value);
    restoreBackupModal.value?.hide();
    addNotification({
      group: "server",
      title: "Backup restored",
      text: "Your backup has been restored successfully.",
      type: "success",
    });
  } catch (error) {
    backupError.value = error instanceof Error ? error.message : String(error);
    addNotification({
      group: "server",
      title: "Error restoring backup",
      text: backupError.value,
      type: "error",
    });
  } finally {
    isRestoringBackup.value = false;
  }
};

const deleteBackup = async () => {
  if (!currentBackup.value) {
    addNotification({
      group: "server",
      title: "Error",
      text: "No backup selected",
      type: "error",
    });
    return;
  }

  isDeletingBackup.value = true;
  try {
    await props.server.backups?.delete(currentBackup.value);
    await props.server.refresh();
    deleteBackupModal.value?.hide();
    addNotification({
      group: "server",
      title: "Backup deleted",
      text: "Your backup has been deleted successfully.",
      type: "success",
    });
  } catch (error) {
    backupError.value = error instanceof Error ? error.message : String(error);
    addNotification({
      group: "server",
      title: "Error deleting backup",
      text: backupError.value,
      type: "error",
    });
  } finally {
    isDeletingBackup.value = false;
  }
};

function triggerDownloadAnimation() {
  overTheTopDownloadAnimation.value = true;
  setTimeout(() => (overTheTopDownloadAnimation.value = false), 500);
}

const initiateDownload = async (backupId: string) => {
  triggerDownloadAnimation();

  const downloadurl: any = await props.server.backups?.download(backupId);
  const a = document.createElement("a");
  a.href = downloadurl.download_url;
  a.click();
  a.remove();
};
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
