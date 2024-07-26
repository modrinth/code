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
              <div class="text-2xl font-extrabold text-white">Create Backup</div>
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
            <Button color="primary" @click="createBackup"> <PlusIcon /> Create Backup </Button>
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
              <div class="text-2xl font-extrabold text-white">Rename Backup</div>
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
            <Button color="primary" @click="renameBackupModal.hide()"> Rename Backup </Button>
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
              <div class="text-2xl font-extrabold text-white">Restore Backup</div>
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
            <Button color="primary" @click="restoreBackupModal.hide()"> Restore Backup </Button>
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
              <div class="text-2xl font-extrabold text-white">Delete Backup</div>
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
            <Button color="danger" @click="deleteBackupModal.hide()"> Delete Backup </Button>
          </div>
        </div>
      </Modal>

      <div class="flex flex-col gap-6">
        <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-8">
          <div class="flex items-center justify-between">
            <div class="flex flex-col gap-2">
              <div class="text-2xl font-extrabold text-white">
                {{ backups.length || 0 }} Backups
              </div>
              <div class="text-contrast font-semibold">
                {{ 15 - backupsData.length }} Slots avaliable
              </div>
            </div>
            <Button color="primary" @click="showModel('createBackupModal')">
              <PlusIcon /> Create Backup
            </Button>
          </div>
        </div>

        <div
          v-for="backup in backupsData"
          :key="backup[0].id"
          class="relative w-full rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
              <div class="flex flex-col gap-2">
                <div class="flex items-center gap-2">
                  <div class="text-2xl font-extrabold text-white">{{ backup[0].name }}</div>
                  <div class="flex gap-2 font-bold text-brand">
                    <CheckIcon class="h-5 w-5" /> Latest
                  </div>
                </div>
                <div class="text-contrast flex gap-2 font-semibold">
                  <CalendarIcon /> {{ backup[0].created_at }}
                </div>
              </div>
              <OverflowMenu
                :options="[
                  { id: 'rename', action: () => renameBackups() },
                  { id: 'restore', action: () => restoreBackups() },
                  { id: 'download', action: () => console.log('download') },
                  { id: 'delete', action: () => deleteBackupModal.show(), color: 'red' },
                ]"
                direction="right"
              >
                <MoreHorizontalIcon class="h-5 w-5" />
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
import { useServerStore } from "~~/stores/servers";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";
import { ref } from "vue";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();
const auth = await useAuth();

import type { Server, ServerBackup } from "~/types/servers";

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData("backupsServerData", async () =>
  serverStore.getServerData(serverId),
);

const { data: backupsData, status: backupsStatus } = await useLazyAsyncData(
  "backupsData",
  async () => usePyroFetch<ServerBackup[]>(auth.value.token, `servers/${serverId}/backups`),
);

const createBackupModal = ref<Modal | null>(null);
const renameBackupModal = ref<Modal | null>(null);
const restoreBackupModal = ref<Modal | null>(null);
const deleteBackupModal = ref<Modal | null>(null);

const c_backupsName = ref();
const r_backupsName = ref();

const showModel = (modal: string) => {
  if (modal === "createBackupModal") {
    createBackupModal.value.show();
  } else if (modal === "renameBackupModal") {
    renameBackupModal.value.show();
  } else if (modal === "restoreBackupModal") {
    restoreBackupModal.value.show();
  } else if (modal === "deleteBackupModal") {
    deleteBackupModal.value.show();
  }
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

const renameBackups = async (backupId: string) => {
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

const backups = [
  {
    name: "Backup #4",
    daytime: "Today, 7:27 PM",
    latest: true,
  },
  {
    name: "Backup #3",
    daytime: "Yesterday, 4:21 PM",
    latest: false,
  },
  {
    name: "Backup #2",
    daytime: "Yesterday, 4:20 PM",
    latest: false,
  },
  {
    name: "Backup #1",
    daytime: "2 weeks ago",
    latest: false,
  },
];
</script>
