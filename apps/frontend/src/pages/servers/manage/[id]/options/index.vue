<template>
  <div class="relative h-full w-full overflow-y-auto">
    <div v-if="data" class="flex h-full w-full flex-col">
      <div class="gap-2">
        <div class="card flex flex-col gap-4">
          <label for="server-name-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Server name</span>
            <span> This name is only visible on Modrinth.</span>
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
          <label for="server-subdomain" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Custom URL</span>
            <span> Your friends can connect to your server using this URL. </span>
          </label>
          <div class="flex w-full items-center gap-2 md:w-[60%]">
            <input
              id="server-subdomain"
              v-model="serverSubdomain"
              class="h-[50%] w-[63%]"
              maxlength="32"
              @keyup.enter="saveGeneral"
            />
            .modrinth.gg
          </div>
          <div v-if="!isValidSubdomain" class="flex flex-col text-sm text-rose-400">
            <span v-if="!isValidLengthSubdomain">
              Subdomain must be at least 5 characters long.
            </span>
            <span v-if="!isValidCharsSubdomain">
              Subdomain can only contain alphanumeric characters and dashes.
            </span>
          </div>
        </div>

        <div class="card flex flex-col gap-4">
          <label for="server-icon-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Server icon</span>
            <span> This icon will be visible on the Minecraft server list and on Modrinth. </span>
          </label>
          <div class="flex gap-4">
            <div
              v-tooltip="'Upload a custom Icon'"
              class="group relative flex w-fit cursor-pointer items-center gap-2 rounded-xl bg-table-alternateRow"
              @dragover.prevent="onDragOver"
              @dragleave.prevent="onDragLeave"
              @drop.prevent="onDrop"
              @click="triggerFileInput"
            >
              <input
                v-if="icon"
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
              <UiServersServerIcon :image="icon" />
            </div>
            <ButtonStyled>
              <button v-tooltip="'Synchronize icon with installed modpack'" @click="resetIcon">
                <TransferIcon class="h-6 w-6" />
                <span>Sync icon</span>
              </button>
            </ButtonStyled>
          </div>
        </div>
      </div>
    </div>
    <div v-else />
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
import { EditIcon, TransferIcon } from "@modrinth/assets";
import ButtonStyled from "@modrinth/ui/src/components/base/ButtonStyled.vue";

import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const data = computed(() => props.server.general);
const serverName = ref(data.value?.name);
const serverSubdomain = ref(data.value?.net?.domain ?? "");
const isValidLengthSubdomain = computed(() => serverSubdomain.value.length >= 5);
const isValidCharsSubdomain = computed(() => /^[a-zA-Z0-9-]+$/.test(serverSubdomain.value));
const isValidSubdomain = computed(
  () => isValidLengthSubdomain.value && isValidCharsSubdomain.value,
);
const icon = computed(() => data.value?.image);

const isUpdating = ref(false);
const hasUnsavedChanges = computed(
  () =>
    (serverName.value && serverName.value !== data.value?.name) ||
    serverSubdomain.value !== data.value?.net?.domain,
);
const isValidServerName = computed(() => (serverName.value?.length ?? 0) > 0);

watch(serverName, (oldValue) => {
  if (!isValidServerName.value) {
    serverName.value = oldValue;
  }
});

const saveGeneral = async () => {
  if (!isValidServerName.value || !isValidSubdomain.value) return;

  try {
    isUpdating.value = true;
    if (serverName.value !== data.value?.name) {
      await data.value?.updateName(serverName.value ?? "");
    }
    if (serverSubdomain.value !== data.value?.net?.domain) {
      try {
        // type shit backend makes me do
        const response = await props.server.network?.checkSubdomainAvailability(
          serverSubdomain.value,
        );
        if (response === undefined) {
          throw new Error("Failed to check subdomain availability");
        }

        if (typeof response === "object" && response !== null && "available" in response) {
          const typedResponse = response as { available: boolean };
          if (!typedResponse.available) {
            addNotification({
              group: "serverOptions",
              type: "error",
              title: "Subdomain not available",
              text: "The subdomain you entered is already in use.",
            });
            return;
          }
        } else {
          throw new Error("Invalid response format from availability check");
        }

        await props.server.network?.changeSubdomain(serverSubdomain.value);
      } catch (error) {
        console.error("Error checking subdomain availability:", error);
        addNotification({
          group: "serverOptions",
          type: "error",
          title: "Error checking availability",
          text: "Failed to verify if the subdomain is available.",
        });
        return;
      }
    }
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
  serverSubdomain.value = data.value?.net?.domain ?? "";
};

const uploadFile = async (e: Event) => {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) {
    addNotification({
      group: "serverOptions",
      type: "error",
      title: "No file selected",
      text: "Please select a file to upload.",
    });
    return;
  }

  const scaledFile = await new Promise<File>((resolve, reject) => {
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");
    const img = new Image();
    img.onload = () => {
      canvas.width = 64;
      canvas.height = 64;
      ctx?.drawImage(img, 0, 0, 64, 64);
      canvas.toBlob((blob) => {
        if (blob) {
          resolve(new File([blob], "server-icon.png", { type: "image/png" }));
        } else {
          reject(new Error("Canvas toBlob failed"));
        }
      }, "image/png");
      URL.revokeObjectURL(img.src);
    };
    img.onerror = reject;
    img.src = URL.createObjectURL(file);
  });

  try {
    if (data.value?.image) {
      await props.server.fs?.deleteFileOrFolder("/server-icon.png", false);
      await props.server.fs?.deleteFileOrFolder("/server-icon-original.png", false);
    }

    await props.server.fs?.uploadFile("/server-icon.png", scaledFile);
    await props.server.fs?.uploadFile("/server-icon-original.png", file);

    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");
    const img = new Image();
    await new Promise<void>((resolve) => {
      img.onload = () => {
        canvas.width = 512;
        canvas.height = 512;
        ctx?.drawImage(img, 0, 0, 512, 512);
        const dataURL = canvas.toDataURL("image/png");
        useState(`server-icon-${props.server.serverId}`).value = dataURL;
        if (data.value) data.value.image = dataURL;
        resolve();
        URL.revokeObjectURL(img.src);
      };
      img.src = URL.createObjectURL(file);
    });

    addNotification({
      group: "serverOptions",
      type: "success",
      title: "Server icon updated",
      text: "Your server icon was successfully changed.",
    });
  } catch (error) {
    console.error("Error uploading icon:", error);
    addNotification({
      group: "serverOptions",
      type: "error",
      title: "Upload failed",
      text: "Failed to upload server icon.",
    });
  }
};

const resetIcon = async () => {
  if (data.value?.image) {
    try {
      await props.server.fs?.deleteFileOrFolder("/server-icon.png", false);
      await props.server.fs?.deleteFileOrFolder("/server-icon-original.png", false);

      useState(`server-icon-${props.server.serverId}`).value = undefined;
      if (data.value) data.value.image = undefined;

      await props.server.refresh(["general"]);

      addNotification({
        group: "serverOptions",
        type: "success",
        title: "Server icon reset",
        text: "Your server icon was successfully reset.",
      });
    } catch (error) {
      console.error("Error resetting icon:", error);
      addNotification({
        group: "serverOptions",
        type: "error",
        title: "Reset failed",
        text: "Failed to reset server icon.",
      });
    }
  }
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
