<template>
  <div class="relative h-full w-full overflow-y-auto">
    <div v-if="data" class="flex h-full w-full flex-col justify-between gap-6">
      <div class="gap-2">
        <div class="card flex flex-col gap-4">
          <label for="server-name-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Server name</span>
            <span> Change the name of your server. This name is only visible on Modrinth.</span>
          </label>
          <div class="flex flex-col gap-2">
            <input
              id="server-name-field"
              v-model="serverName"
              class="w-full md:w-[50%]"
              maxlength="48"
              minlength="1"
              @keyup.enter="!serverName && saveGeneral"
            />
            <span v-if="!serverName" class="text-sm text-rose-400">
              Server name must be at least 1 character long.
            </span>
            <span v-if="!isValidServerName" class="text-sm text-rose-400">
              Server name can contain any character.
            </span>
          </div>
        </div>
        <!-- WIP - disable for now
        <div class="card flex flex-col gap-4">
          <label for="server-motd-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Server MOTD</span>
            <span>
              The message of the day is the message that players see when they log in to the server.
            </span>
          </label>
          <UiServersMOTDEditor :server="props.server" />
        </div>
        -->

        <div class="card flex flex-col gap-4">
          <label for="server-icon-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Server icon</span>
            <span>
              Change your server's icon. Changes will be visible on the Minecraft server list and on
              Modrinth.
            </span>
          </label>
          <div
            v-tooltip="'Upload a custom Icon'"
            class="group relative flex w-fit cursor-pointer items-center gap-2 rounded-xl bg-table-alternateRow"
            @dragover.prevent="onDragOver"
            @dragleave.prevent="onDragLeave"
            @drop.prevent="onDrop"
            @click="triggerFileInput"
          >
            <input
              v-if="data?.image"
              id="server-icon-field"
              type="file"
              accept="image/png,image/jpeg,image/gif,image/webp"
              hidden
              @change="uploadFile"
            />
            <div
              class="absolute top-0 hidden size-[6rem] flex-col items-center justify-center rounded-xl bg-button-bg p-2 opacity-80 group-hover:flex"
            >
              <EditIcon class="h-8 w-8 text-contrast" />
            </div>
            <img
              v-if="data?.image"
              no-shadow
              alt="Server Icon"
              class="h-[6rem] w-[6rem] rounded-xl"
              :src="data.image"
            />
            <img
              v-else
              no-shadow
              alt="Server Icon"
              class="h-[6rem] w-[6rem] rounded-xl"
              src="~/assets/images/servers/minecraft_server_icon.png"
            />
          </div>
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
    <UiServersSaveBanner
      :is-visible="!!hasUnsavedChanges && !!isValidServerName"
      :server="props.server"
      :is-updating="isUpdating"
      :save="saveGeneral"
      :reset="resetGeneral"
    />
  </div>
</template>

<script setup lang="ts">
import { EditIcon } from "@modrinth/assets";

import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const data = computed(() => props.server.general);
const serverName = ref(data.value?.name);

const isUpdating = ref(false);
const hasUnsavedChanges = computed(() => serverName.value && serverName.value !== data.value?.name);

const isValidServerName = computed(() => (serverName.value?.length ?? 0) > 0);

watch(serverName, (oldValue) => {
  if (!isValidServerName.value) {
    serverName.value = oldValue;
  }
});

const saveGeneral = async () => {
  if (!isValidServerName.value) return;

  try {
    isUpdating.value = true;
    await data.value?.updateName(serverName.value ?? "");
    await new Promise((resolve) => setTimeout(resolve, 500));
    await props.server.refresh();
    addNotification({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
  } catch (error) {
    console.error(error);
    addNotification({
      group: "serverOptions",
      type: "error",
      title: "Failed to update server settings",
      text: "An error occurred while attempting to update your server settings.",
    });
  } finally {
    isUpdating.value = false;
  }
};

const resetGeneral = () => {
  serverName.value = data.value?.name || "";
};

const uploadFile = async (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0];
  // down scale the image to 64x64
  const scaledFile = await new Promise<File>((resolve, reject) => {
    if (!file) {
      addNotification({
        group: "serverOptions",
        type: "error",
        title: "No file selected",
        text: "Please select a file to upload.",
      });
      reject(new Error("No file selected"));
      return;
    }
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");
    const img = new Image();
    img.src = URL.createObjectURL(file);
    img.onload = () => {
      canvas.width = 64;
      canvas.height = 64;
      ctx?.drawImage(img, 0, 0, 64, 64);
      // turn the downscaled image back to a png file
      canvas.toBlob((blob) => {
        if (blob) {
          const data = new File([blob], "server-icon.png", { type: "image/png" });
          resolve(data);
        } else {
          reject(new Error("Canvas toBlob failed"));
        }
      }, "image/png");
    };
    img.onerror = reject;
  });
  if (!file) return;
  if (data.value?.image) {
    await props.server.fs?.deleteFileOrFolder("/server-icon.png", false);
    await props.server.fs?.deleteFileOrFolder("/server-icon-original.png", false);
  }
  await props.server.fs?.uploadFile("/server-icon.png", scaledFile);
  await props.server.fs?.uploadFile("/server-icon-original.png", file);
  await props.server.refresh();

  addNotification({
    group: "serverOptions",
    type: "success",
    title: "Server icon updated",
    text: "Your server icon was successfully changed.",
  });
};

const onDragOver = (e: DragEvent) => {
  e.preventDefault();
};

const onDragLeave = (e: DragEvent) => {
  e.preventDefault();
};

const onDrop = (e: DragEvent) => {
  e.preventDefault();
  uploadFile(e);
};

const triggerFileInput = () => {
  const input = document.createElement("input");
  input.type = "file";
  input.id = "server-icon-field";
  input.accept = "image/png,image/jpeg,image/gif,image/webp";
  input.onchange = uploadFile;
  input.click();
};
</script>
