<template>
  <div>
    <div v-if="data && status === 'success'">
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
      <Modal ref="createBackupModal" header="">
        <div class="flex flex-col gap-4 p-6">
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-4">
              <UiAvatar
                v-if="data && data.project"
                :src="data.project.icon_url"
                no-shadow
                size="sm"
                alt="Server Icon"
              />
              <div class="text-2xl font-extrabold text-contrast">Create backup</div>
            </div>
            <button
              class="h-8 w-8 rounded-full bg-[#ffffff10] p-2 text-contrast"
              @click="createBackupModal?.hide()"
            >
              <XIcon class="h-4 w-4" />
            </button>
          </div>
          <div class="mb-3 mt-3">
            Your server will temporarily shutdown while the backup is being created.
          </div>
          <div class="flex flex-col gap-2">
            <div class="font-semibold text-contrast">Name<span class="text-red-500">*</span></div>
            <input
              v-model="c_backupsName"
              type="text"
              class="bg-bg-input w-full rounded-lg p-4"
              placeholder="e.g. Before 1.21"
            />
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="createBackupModal?.hide()"> Cancel </Button>
            <Button color="primary" @click="createBackup"> <PlusIcon /> Create backup </Button>
          </div>
        </div>
      </Modal>
      <Modal ref="renameBackupModal" header="">
        <div class="flex flex-col gap-4 p-6">
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-4">
              <UiAvatar
                v-if="data && data.project"
                :src="data.project.icon_url"
                no-shadow
                size="sm"
                alt="Server Icon"
              />
              <div class="text-2xl font-extrabold text-contrast">Rename backup</div>
            </div>
            <button
              class="h-8 w-8 rounded-full bg-[#ffffff10] p-2 text-contrast"
              @click="renameBackupModal?.hide()"
            >
              <XIcon class="h-4 w-4" />
            </button>
          </div>
          <div class="mt-2 flex flex-col gap-2">
            <div class="font-semibold text-contrast">Name<span class="text-red-500">*</span></div>
            <input
              v-model="r_backupsName"
              type="text"
              class="bg-bg-input w-full rounded-lg p-4"
              placeholder="e.g. Before 1.21"
            />
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="renameBackupModal?.hide()"> Cancel </Button>
            <Button color="primary" @click="renameBackup(currentBackup)"> Rename backup </Button>
          </div>
        </div>
      </Modal>
      <Modal ref="restoreBackupModal" header="">
        <div class="flex flex-col gap-4 p-6">
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-4">
              <UiAvatar
                v-if="data && data.project"
                :src="data.project.icon_url"
                no-shadow
                size="sm"
                alt="Server Icon"
              />
              <div class="text-2xl font-extrabold text-contrast">Restore backup</div>
            </div>
            <button
              class="h-8 w-8 rounded-full bg-[#ffffff10] p-2 text-contrast"
              @click="restoreBackupModal?.hide()"
            >
              <XIcon class="h-4 w-4" />
            </button>
          </div>
          <div class="flex flex-col gap-4">
            <div class="relative flex w-full flex-col gap-2 rounded-2xl bg-bg p-6">
              <div class="text-2xl font-extrabold text-contrast">
                {{ data.backups.find((b) => b.id === currentBackup)?.name }}
              </div>
              <div class="flex gap-2 font-semibold text-contrast">
                <CalendarIcon />
                {{
                  new Date(
                    data.backups.find((b) => b.id === currentBackup)?.created_at || "",
                  ).toLocaleString()
                }}
              </div>
            </div>
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="restoreBackupModal?.hide()"> Cancel </Button>
            <Button color="primary" @click="restoreBackup(currentBackup)"> Restore backup </Button>
          </div>
        </div>
      </Modal>
      <Modal ref="deleteBackupModal" header="">
        <div
          class="flex flex-col gap-4 rounded-2xl border-2 border-solid border-[#FF496E] bg-[#270B11] p-6"
        >
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-4">
              <UiAvatar
                v-if="data && data.project"
                :src="data.project.icon_url"
                no-shadow
                size="sm"
                alt="Server Icon"
              />
              <div class="text-2xl font-extrabold text-contrast">Delete backup</div>
            </div>
            <button
              class="h-8 w-8 rounded-full bg-[#ffffff10] p-2 text-contrast"
              @click="deleteBackupModal?.hide()"
            >
              <XIcon class="h-4 w-4" />
            </button>
          </div>
          <div class="flex flex-col gap-4">
            <div class="relative flex w-full flex-col gap-2 rounded-2xl bg-[#0e0e0ea4] p-6">
              <div class="text-2xl font-extrabold text-contrast">
                {{ data.backups.find((b) => b.id === currentBackup)?.name }}
              </div>
              <div class="flex gap-2 font-semibold text-contrast">
                <CalendarIcon />
                {{
                  new Date(
                    data.backups.find((b) => b.id === currentBackup)?.created_at || "",
                  ).toLocaleString()
                }}
              </div>
            </div>
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="deleteBackupModal?.hide()"> Cancel </Button>
            <Button color="danger" @click="deleteBackupModal?.hide()"> Delete backup </Button>
          </div>
        </div>
      </Modal>

      <div class="flex flex-col gap-6">
        <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-8">
          <div class="flex items-center justify-between">
            <div class="flex flex-col gap-2">
              <div class="text-2xl font-extrabold text-contrast">
                {{ data.used_backup_quota }} Backups
              </div>
              <div class="font-semibold text-contrast">
                {{ data.backup_quota - data.used_backup_quota }} Slots avaliable
              </div>
            </div>
            <Button color="primary" @click="showCreateModel()"> <PlusIcon /> Create backup </Button>
          </div>
        </div>

        <div
          v-for="(backup, index) in data.backups"
          :key="backup.id"
          class="relative w-full rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
              <div class="flex flex-col gap-2">
                <div class="flex items-center gap-2">
                  <div class="text-2xl font-extrabold text-contrast">{{ backup.name }}</div>
                  <div v-if="index == 0" class="flex gap-2 font-bold text-brand">
                    <CheckIcon class="h-5 w-5" /> Latest
                  </div>
                </div>
                <div class="flex gap-2 font-semibold text-contrast">
                  <CalendarIcon /> {{ new Date(backup.created_at).toLocaleString() }}
                </div>
              </div>
              <OverflowMenu
                :options="[
                  {
                    id: 'rename',
                    action: () => {
                      r_backupsName = backup.name;
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
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else-if="status === 'pending'" />
    <UiServersPyroError
      v-else-if="status === 'error'"
      title="Could not load backups"
      message="Your backups could not be displayed due to a technical issue on our end. Please try again later."
    />
  </div>
</template>

<script setup lang="ts">
import { Button, OverflowMenu, Modal } from "@modrinth/ui";
import {
  PlusIcon,
  CheckIcon,
  CalendarIcon,
  MoreHorizontalIcon,
  EditIcon,
  ClipboardCopyIcon,
  DownloadIcon,
  TrashIcon,
  XIcon,
} from "@modrinth/assets";
import { ref, reactive } from "vue";

import type { ServerBackup } from "~/types/servers.ts";

const app = useNuxtApp();
const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();
const auth = await useAuth();
const emits = defineEmits(["onDownload"]);

useHead({
  title: `Backups - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

const backupError = ref<string | null>(null);

const { data, status } = await useLazyAsyncData("backupsData", async () => {
  await serverStore.fetchServerData(serverId);
  return serverStore.getServerData(serverId);
});

const overTheTopDownloadAnimation = ref();

const createBackupModal = ref<typeof Modal>();
const renameBackupModal = ref<typeof Modal>();
const restoreBackupModal = ref<typeof Modal>();
const deleteBackupModal = ref<typeof Modal>();

const c_backupsName = ref("");
const r_backupsName = ref("");

const currentBackup = ref("");

const backupsState = reactive({
  loading: false,
});

const showCreateModel = () => {
  createBackupModal.value?.show();
};

const createBackup = async () => {
  backupsState.loading = true;
  const backupName = c_backupsName.value;
  try {
    serverStore.createBackup(serverId, backupName);

    await refreshNuxtData("backupsData");
    createBackupModal.value?.hide();
    // @ts-ignore
    app.$notify({
      group: "server",
      title: `Backup created`,
      text: "Your backup has been created successfully.",
      type: "success",
    });
  } catch (error) {
    backupError.value = error as string;
  } finally {
    backupsState.loading = false;
  }
};

const renameBackup = async (backupId: string) => {
  const backupName = r_backupsName.value;
  console.log("renaming", backupName);
  try {
    await serverStore.renameBackup(serverId, backupId, backupName);

    await refreshNuxtData("backupsData");
    renameBackupModal.value?.hide();
  } catch (error) {
    backupError.value = error as string;
  }

  await renameBackupModal.value?.hide();
};

const restoreBackup = async (backupId: string) => {
  try {
    await serverStore.restoreBackup(serverId, backupId);

    await restoreBackupModal.value?.hide();
  } catch (error) {
    backupError.value = error as string;
  }

  await restoreBackupModal.value?.hide();
};

const deleteBackup = async (backupId: string) => {
  try {
    await serverStore.deleteBackup(serverId, backupId);

    await refreshNuxtData("backupsData");
    await deleteBackupModal.value?.hide();
  } catch (error) {
    backupError.value = error as string;
  }

  await deleteBackupModal.value?.hide();
};

function triggerDownloadAnimation() {
  overTheTopDownloadAnimation.value = true;
  setTimeout(() => (overTheTopDownloadAnimation.value = false), 500);
}

const initiateDownload = async (backupId: string) => {
  triggerDownloadAnimation();

  const downloadurl: any = await serverStore.downloadBackup(serverId, backupId);
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
