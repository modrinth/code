<template>
  <div>
    <div v-if="data && backupsData && status === 'success' && backupsStatus === 'success'">
      <Modal header="" ref="createBackupModal">
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
              <div class="text-2xl font-extrabold text-white">Create backup</div>
            </div>
            <button
              @click="createBackupModal.hide()"
              class="text-contrast rounded-full bg-[#ffffff10] p-2"
            >
              <XIcon class="h-5 w-5" />
            </button>
          </div>
          <div class="mb-3 mt-3">
            Your server will temporarily shutdown while the backup is being created.
          </div>
          <div class="flex flex-col gap-2">
            <div class="font-semibold text-white">Name<span class="text-red-500">*</span></div>
            <input
              ref="c_backupsName"
              type="text"
              class="bg-bg-input w-full rounded-lg p-4"
              placeholder="e.g. Before 1.21"
            />
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="createBackupModal.hide()"> Cancel </Button>
            <Button color="primary" @click="createBackup"> <PlusIcon /> Create backup </Button>
          </div>
        </div>
      </Modal>
      <Modal header="" ref="renameBackupModal">
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
              <div class="text-2xl font-extrabold text-white">Rename backup</div>
            </div>
            <button
              @click="renameBackupModal.hide()"
              class="text-contrast rounded-full bg-[#ffffff10] p-2"
            >
              <XIcon class="h-5 w-5" />
            </button>
          </div>
          <div class="mt-2 flex flex-col gap-2">
            <div class="font-semibold text-white">Name<span class="text-red-500">*</span></div>
            <input
              ref="r_backupsName"
              type="text"
              class="bg-bg-input w-full rounded-lg p-4"
              placeholder="e.g. Before 1.21"
            />
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="renameBackupModal.hide()"> Cancel </Button>
            <Button color="primary" @click="renameBackupModal.hide()"> Rename backup </Button>
          </div>
        </div>
      </Modal>
      <Modal header="" ref="restoreBackupModal">
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
              <div class="text-2xl font-extrabold text-white">Restore backup</div>
            </div>
            <button
              @click="restoreBackupModal.hide()"
              class="text-contrast rounded-full bg-[#ffffff10] p-2"
            >
              <XIcon class="h-5 w-5" />
            </button>
          </div>
          <div class="flex flex-col gap-4">
            <div class="relative flex w-full flex-col gap-2 rounded-2xl bg-bg p-6">
              <div class="text-2xl font-extrabold text-white">Backup #4</div>
              <div class="text-contrast flex gap-2 font-semibold">
                <CalendarIcon /> Today, 7:27 PM
              </div>
            </div>
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="restoreBackupModal.hide()"> Cancel </Button>
            <Button color="primary" @click="restoreBackupModal.hide()"> Restore backup </Button>
          </div>
        </div>
      </Modal>
      <Modal header="" ref="deleteBackupModal">
        <div class="flex flex-col gap-4 rounded-2xl border-2 border-[#FF496E] bg-[#270B11] p-6">
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-4">
              <UiAvatar
                v-if="data && data.project"
                :src="data.project.icon_url"
                no-shadow
                size="sm"
                alt="Server Icon"
              />
              <div class="text-2xl font-extrabold text-white">Delete backup</div>
            </div>
            <button
              @click="deleteBackupModal.hide()"
              class="text-contrast rounded-full bg-[#ffffff10] p-2"
            >
              <XIcon class="h-5 w-5" />
            </button>
          </div>
          <div class="flex flex-col gap-4">
            <div class="relative flex w-full flex-col gap-2 rounded-2xl bg-[#0e0e0ea4] p-6">
              <div class="text-2xl font-extrabold text-white">Backup #4</div>
              <div class="text-contrast flex gap-2 font-semibold">
                <CalendarIcon /> Today, 7:27 PM
              </div>
            </div>
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="deleteBackupModal.hide()"> Cancel </Button>
            <Button color="danger" @click="deleteBackupModal.hide()"> Delete backup </Button>
          </div>
        </div>
      </Modal>

      <div class="flex flex-col gap-6">
        <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-8">
          <div class="flex items-center justify-between">
            <div class="flex flex-col gap-2">
              <div class="text-2xl font-extrabold text-white">
                {{ data.used_backup_quota }} Backups
              </div>
              <div class="text-contrast font-semibold">
                {{ data.backup_quota - data.used_backup_quota }} Slots avaliable
              </div>
            </div>
            <Button color="primary" @click="showCreateModel()"> <PlusIcon /> Create backup </Button>
          </div>
        </div>

        <div
          v-for="backup in backupsData"
          :key="backup.id"
          class="relative w-full rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
              <div class="flex flex-col gap-2">
                <div class="flex items-center gap-2">
                  <div class="text-2xl font-extrabold text-white">{{ backup.name }}</div>
                  <div
                    class="flex gap-2 font-bold text-brand"
                    v-if="latestBackup?.id === backup.id"
                  >
                    <CheckIcon class="h-5 w-5" /> Latest
                  </div>
                </div>
                <div class="text-contrast flex gap-2 font-semibold">
                  <CalendarIcon /> {{ new Date(backup.created_at).toLocaleString() }}
                </div>
              </div>
              <OverflowMenu
                :options="[
                  { id: 'rename', action: () => renameBackupModal.show() },
                  { id: 'restore', action: () => restoreBackupModal.show() },
                  { id: 'download', action: () => initiateDownload(backup.id) },
                  { id: 'delete', action: () => deleteBackupModal.show(), color: 'red' },
                ]"
                direction="right"
                class="bg-transparent"
              >
                <MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
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
    <PyroLoading v-else-if="status === 'pending' || backupsStatus === 'pending'" />
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
import { useServerStore } from "~/stores/servers";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";
import { ref } from "vue";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();
const auth = await useAuth();

import type { Server, ServerBackup } from "~/types/servers";

const { data, status } = await useLazyAsyncData("backupsServerData", async () => {
  await serverStore.fetchServerData(serverId);
  return serverStore.getServerData(serverId);
});

// timestamp format 2024-07-26T05:49:19.121845
const { data: backupsData, status: backupsStatus } = await useLazyAsyncData(
  "backupsData",
  async () => usePyroFetch<ServerBackup[]>(auth.value.token, `servers/${serverId}/backups`),
);
const latestBackup = backupsData.value?.reduce((a, b) => (a.created_at > b.created_at ? a : b));
backupsData.value?.sort((a, b) => (a.created_at > b.created_at ? -1 : 1));

const createBackupModal = ref<Modal | null>(null);
const renameBackupModal = ref<Modal | null>(null);
const restoreBackupModal = ref<Modal | null>(null);
const deleteBackupModal = ref<Modal | null>(null);

const c_backupsName = ref();
const r_backupsName = ref();

const showCreateModel = () => {
  createBackupModal.value.show();
};

const createBackup = async () => {
  const backupName = c_backupsName.value.value;
  await usePyroFetch(
    auth.value.token,
    `servers/${serverId}/backups`,
    0,
    "POST",
    "application/json",
    {
      name: backupName,
    },
  );
};

const renameBackup = async (backupId: string) => {
  const backupName = r_backupsName.value.value;
  await usePyroFetch(
    auth.value.token,
    `servers/${serverId}/backups/${backupId}`,
    0,
    "PUT",
    "application/json",
    {
      name: backupName,
    },
  );

  await renameBackupModal.value?.hide();
};

const restoreBackup = async (backupId: string) => {
  await usePyroFetch(
    auth.value.token,
    `servers/${serverId}/backups/${backupId}/restore`,
    0,
    "POST",
    "application/json",
  );

  await restoreBackupModal.value?.hide();
};

interface downloadUrl {
  download_url: string;
  experation_seconds: number;
}

const initiateDownload = async (backupId: string) => {
  const downloadurl: downloadUrl = await usePyroFetch(
    auth.value.token,
    `servers/${serverId}/backups/${backupId}`,
    0,
    "GET",
    "application/json",
  );
  const a = document.createElement("a");
  a.href = downloadurl.download_url;
  a.download = "backup.zip";
  a.click();
};
</script>
