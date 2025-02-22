<template>
  <div class="relative flex select-none flex-col" data-pyro-servers-page="backups">
    <ErrorBoundary
      v-if="server.backups?.error"
      title="Failed to load backups"
      message="We couldn't load your server's backups. Here's what went wrong:"
      :error="server.backups.error"
      @retry="() => server.refresh(['backups'])"
    />

    <div v-else-if="data" class="contents">
      <ModalCreateBackup
        ref="createBackupModal"
        :server="server"
        @backup-created="handleBackupOperation"
      />
      <ModalRenameBackup
        ref="renameBackupModal"
        :server="server"
        :current-backup-id="currentBackup"
        :backup-name="renameBackupName"
        @backup-renamed="handleBackupOperation"
      />
      <ModalRestoreBackup
        ref="restoreBackupModal"
        :server="server"
        :backup-id="currentBackup"
        :backup-name="currentBackupDetails?.name ?? ''"
        :backup-created-at="currentBackupDetails?.created_at ?? ''"
        @backup-restored="handleBackupOperation"
      />
      <ModalDeleteBackup
        ref="deleteBackupModal"
        :server="server"
        :backup-id="currentBackup"
        :backup-name="currentBackupDetails?.name ?? ''"
        :backup-created-at="currentBackupDetails?.created_at ?? ''"
        @backup-deleted="handleBackupOperation"
      />
      <ModalAutoBackupSettings ref="backupSettingsModal" :server="server" />

      <ul class="m-0 flex list-none flex-col gap-4 p-0">
        <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-6 shadow-md">
          <div class="flex flex-col items-center justify-between gap-4 sm:flex-row sm:gap-0">
            <div class="flex flex-col items-baseline gap-2">
              <div class="text-2xl font-bold text-contrast">{{ backupQuotaText }}</div>
              <div>{{ backupQuotaRemainingText }}</div>
            </div>
            <div class="flex w-full flex-col gap-2 sm:w-fit sm:flex-row">
              <ButtonStyled type="standard">
                <button :disabled="isServerInstalling" @click="showBackupSettingsModal">
                  <SettingsIcon class="h-5 w-5" />
                  Auto backups
                </button>
              </ButtonStyled>
              <ButtonStyled type="standard" color="brand">
                <button
                  v-tooltip="createBackupTooltip"
                  class="w-full sm:w-fit"
                  :disabled="isCreateBackupDisabled"
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
          v-if="hasOngoingBackups"
          data-pyro-server-backup-ongoing
          class="flex w-full flex-row items-center gap-4 rounded-2xl bg-bg-orange p-4 text-contrast"
        >
          A backup is currently being created.
        </div>

        <div class="flex w-full flex-col gap-2">
          <li
            v-for="(backup, index) in backups"
            :key="backup.id"
            class="relative m-0 w-full list-none rounded-2xl bg-bg-raised p-2 shadow-md"
          >
            <div class="flex flex-col gap-4">
              <div class="flex items-center justify-between">
                <div class="flex min-w-0 flex-row items-center gap-4">
                  <div
                    class="grid size-14 shrink-0 place-content-center overflow-hidden rounded-xl border-[1px] border-solid border-button-border shadow-sm"
                    :class="backup.ongoing ? 'text-green [&&]:bg-bg-green' : 'bg-button-bg'"
                  >
                    <component
                      :is="getBackupIcon(backup)"
                      class="size-8"
                      :class="{ 'animate-spin': backup.ongoing }"
                    />
                  </div>
                  <div class="flex min-w-0 flex-col gap-2">
                    <div class="flex min-w-0 flex-col gap-2 sm:flex-row sm:items-center">
                      <div class="max-w-full truncate font-bold text-contrast">
                        {{ backup.name }}
                      </div>
                      <div
                        v-if="index === 0"
                        class="hidden items-center gap-1 rounded-full bg-bg-green p-1 px-1.5 text-xs font-semibold text-brand sm:flex"
                      >
                        <CheckIcon class="size-4" /> Latest
                      </div>
                    </div>
                    <div class="flex items-center gap-1 text-xs">
                      <CalendarIcon class="size-4" />
                      {{ formatBackupDate(backup.created_at) }}
                    </div>
                  </div>
                </div>
                <ButtonStyled v-if="!backup.ongoing" circular type="transparent">
                  <UiServersTeleportOverflowMenu
                    direction="left"
                    position="bottom"
                    class="bg-transparent"
                    :disabled="hasOngoingBackups"
                    :options="getBackupOptions(backup)"
                  >
                    <MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
                    <template #rename><EditIcon /> Rename</template>
                    <template #restore><ClipboardCopyIcon /> Restore</template>
                    <template #lock>
                      <component :is="backup.locked ? LockOpenIcon : LockIcon" />
                      {{ backup.locked ? "Unlock" : "Lock" }}
                    </template>
                    <template #download><DownloadIcon /> Download</template>
                    <template #delete><TrashIcon /> Delete</template>
                  </UiServersTeleportOverflowMenu>
                </ButtonStyled>
              </div>
            </div>
          </li>
        </div>
      </ul>

      <div
        class="over-the-top-download-animation"
        :class="{ 'animation-hidden': !overTheTopDownloadAnimation }"
      >
        <div>
          <div
            v-for="i in 3"
            :key="i"
            :class="`animation-ring-${i}`"
            class="flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight"
            :style="{ opacity: i === 3 ? 0.4 : i === 2 ? 0.6 : 1 }"
          >
            <DownloadIcon v-if="i === 1" class="h-20 w-20 text-contrast" />
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
import ErrorBoundary from "~/components/ErrorBoundary.vue";
import type { Server } from "~/composables/pyroServers";
import ModalCreateBackup from "~/components/ui/servers/backups/ModalCreateBackup.vue";
import ModalRenameBackup from "~/components/ui/servers/backups/ModalRenameBackup.vue";
import ModalRestoreBackup from "~/components/ui/servers/backups/ModalRestoreBackup.vue";
import ModalDeleteBackup from "~/components/ui/servers/backups/ModalDeleteBackup.vue";
import ModalAutoBackupSettings from "~/components/ui/servers/backups/ModalAutoBackupSettings.vue";
import LoadingIcon from "~/components/ui/servers/icons/LoadingIcon.vue";
import type { Option } from "~/components/ui/servers/TeleportOverflowMenu.vue";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
  isServerRunning: boolean;
}>();

defineEmits<{
  (e: "onDownload"): void;
}>();

const route = useNativeRoute();
const serverId = route.params.id;

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
  backupWhileRunning: false,
});

const data = computed(() => props.server.general);
useHead({ title: `Backups - ${data.value?.name ?? "Server"} - Modrinth` });

const backups = computed(
  () =>
    props.server.backups?.data
      ?.slice()
      .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()) ?? [],
);
const hasOngoingBackups = computed(() => backups.value.some((b) => b.ongoing));
const isServerInstalling = computed(() => props.server.general?.status === "installing");
const currentBackupDetails = computed(() =>
  backups.value.find((b) => b.id === currentBackup.value),
);
const isCreateBackupDisabled = computed(
  () =>
    (props.isServerRunning && !userPreferences.value.backupWhileRunning) ||
    (data.value?.used_backup_quota ?? 0) >= (data.value?.backup_quota ?? 0) ||
    hasOngoingBackups.value ||
    isServerInstalling.value,
);

const backupQuotaText = computed(() => {
  const count = data.value?.used_backup_quota ?? 0;
  return count === 0 ? "No backups" : `You've created ${count} backup${count === 1 ? "" : "s"}`;
});

const backupQuotaRemainingText = computed(() => {
  const remaining = (data.value?.backup_quota ?? 0) - (data.value?.used_backup_quota ?? 0);
  return remaining === 0
    ? "You have reached your backup limit. Consider removing old backups to create new ones."
    : `You can create ${remaining} more backups for your server.`;
});

const createBackupTooltip = computed(() => {
  if (props.isServerRunning && !userPreferences.value.backupWhileRunning) {
    return "Cannot create backup while server is running. You can disable this from your server Options > Preferences.";
  }
  if (isServerInstalling.value) {
    return "Cannot create backups while server is being installed";
  }
  return "";
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

const showCreateModel = () => {
  createBackupModal.value?.show();
};

const showBackupSettingsModal = () => {
  backupSettingsModal.value?.show();
};

const handleBackupOperation = ({ success, message }: { success: boolean; message: string }) => {
  addNotification({ type: success ? "success" : "error", text: message });
  if (success) {
    props.server.refresh(["backups"]);
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

const formatBackupDate = (date: string) => {
  const utcDate = new Date(date);
  const localDate = new Date(utcDate.getTime() - utcDate.getTimezoneOffset() * 60000);

  return localDate.toLocaleString("en-US", {
    month: "numeric",
    day: "numeric",
    year: "2-digit",
    hour: "numeric",
    minute: "numeric",
    hour12: true,
  });
};

const getBackupIcon = (backup: any) => {
  if (backup.ongoing) return LoadingIcon;
  if (backup.locked) return LockIcon;
  return BoxIcon;
};

interface Backup {
  id: string;
  name: string;
  created_at: string;
  ongoing: boolean;
  locked: boolean;
}

const getBackupOptions = (backup: Backup): Option[] => [
  {
    id: "rename",
    action: () => {
      renameBackupName.value = backup.name;
      currentBackup.value = backup.id;
      renameBackupModal.value?.show();
    },
  },
  {
    id: "restore",
    action: () => {
      currentBackup.value = backup.id;
      restoreBackupModal.value?.show();
    },
  },
  {
    id: "download",
    action: () => initiateDownload(backup.id),
  },
  {
    id: "lock",
    action: () => {
      if (backup.locked) {
        unlockBackup(backup.id);
      } else {
        lockBackup(backup.id);
      }
    },
  },
  {
    id: "delete",
    action: () => {
      currentBackup.value = backup.id;
      deleteBackupModal.value?.show();
    },
    color: "red" as const,
  },
];

onMounted(() => {
  watchEffect(() => {
    if (refreshInterval.value) {
      clearInterval(refreshInterval.value);
      refreshInterval.value = undefined;
    }

    if (hasOngoingBackups.value) {
      refreshInterval.value = setInterval(() => props.server.refresh(["backups"]), 10000);
    }
  });
});

onUnmounted(() => {
  if (refreshInterval.value) clearInterval(refreshInterval.value);
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
