<template>
  <div>
    <div v-if="data && status === 'success'">
      <Modal v-if="createBackupModal" header="">
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
              type="text"
              class="bg-bg-input w-full rounded-lg p-4"
              placeholder="e.g. Before 1.21"
            />
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="createBackupModal.hide()"> Cancel </Button>
            <Button color="primary" @click="createBackupModal.hide()">
              <PlusIcon /> Create Backup
            </Button>
          </div>
        </div>
      </Modal>
      <Modal v-if="renameBackupModal" header="">
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
      <Modal v-if="restoreBackupModal" header="">
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
      <Modal v-if="deleteBackupModal" header="">
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
              <div class="text-2xl font-extrabold text-white">{{ backups.length }} Backups</div>
              <div class="text-contrast font-semibold">11 Slots avaliable</div>
            </div>
            <Button color="primary" @click="createBackupModal.show()">
              <PlusIcon /> Create Backup
            </Button>
          </div>
        </div>

        <div
          v-for="backup in backups"
          :key="backup.name"
          class="relative w-full rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-col gap-4">
            <div class="flex items-center justify-between">
              <div class="flex flex-col gap-2">
                <div class="flex items-center gap-2">
                  <div class="text-2xl font-extrabold text-white">{{ backup.name }}</div>
                  <div v-if="backup.latest" class="flex gap-2 font-bold text-brand">
                    <CheckIcon class="h-5 w-5" /> Latest
                  </div>
                </div>
                <div class="text-contrast flex gap-2 font-semibold">
                  <CalendarIcon /> {{ backup.daytime }}
                </div>
              </div>
              <OverflowMenu
                :options="[
                  { id: 'rename', action: () => renameBackupModal.show() },
                  { id: 'restore', action: () => restoreBackupModal.show() },
                  { id: 'download', action: () => console.log('download') },
                  { id: 'delete', action: () => deleteBackupModal.show(), color: 'red' },
                ]"
                direction="right"
              >
                <Button transparent class="icon-only">
                  <MoreHorizontalIcon class="h-5 w-5" />
                </Button>
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
    <PyroLoading v-else-if="status === 'pending'" />
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

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

import type { Server } from "~/types/servers";

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData("backupsServerData", async () =>
  serverStore.getServerData(serverId),
);

const createBackupModal = ref<Modal | null>(null);
const renameBackupModal = ref<Modal | null>(null);
const restoreBackupModal = ref<Modal | null>(null);
const deleteBackupModal = ref<Modal | null>(null);

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
