<template>
  <div class="relative h-full w-full">
    <div
      v-if="data && status == 'success'"
      class="flex h-full w-full flex-col justify-between gap-6 p-8"
    >
      <h2 class="text-3xl font-bold">General</h2>
      <div class="flex h-full flex-col gap-2">
        <div class="card flex justify-between gap-4">
          <label for="username-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Server icon</span>
            <span>
              Upload a new server icon for your server will show up both on Modrinth and in the
              server list
            </span>
          </label>
          <div
            @dragover.prevent="onDragOver"
            @dragleave.prevent="onDragLeave"
            @drop.prevent="onDrop"
            @click="triggerFileInput"
            class="flex items-center gap-2 rounded-xl bg-bg p-4"
          >
            <input
              v-if="data?.image"
              type="file"
              accept="image/png,image/jpeg,image/gif,image/webp"
              @change="uploadFile"
              hidden
            />
            <img
              v-if="data?.image"
              no-shadow
              alt="Server Icon"
              class="h-[6rem] w-[6rem]"
              :src="data.image"
            />
            <img
              v-else
              no-shadow
              alt="Server Icon"
              class="h-[6rem] w-[6rem]"
              src="~/assets/images/servers/minecraft_server_icon.png"
            />
          </div>
        </div>
        <div class="card flex flex-col gap-4">
          <label for="username-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Server name</span>
            <span> Change the name of your server as it appears on Modrinth </span>
          </label>
          <input v-model="serverName" class="w-full md:w-[50%]" @keyup.enter="saveGeneral" />
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        v-if="hasUnsavedChanges"
        :is-updating="isUpdating"
        :save="saveGeneral"
        :reset="resetGeneral"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
const app = useNuxtApp();
const route = useRoute();
const serverId = route.params.id as string;

const serverStore = useServerStore();

const { data, status } = await useLazyAsyncData("data", async () => {
  await serverStore.fetchServerData(serverId);
  return serverStore.getServerData(serverId);
});

const serverName = ref<string>(data.value?.name as string);

const isUpdating = ref(false);
const hasUnsavedChanges = computed(() => serverName.value !== data.value?.name);

const saveGeneral = async () => {
  try {
    isUpdating.value = true;
    await serverStore.updateServerName(serverId, serverName.value);
    await new Promise((resolve) => setTimeout(resolve, 500));
    await refreshNuxtData("data");
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
  } catch (error) {
    console.error(error);
    // @ts-ignore
    app.$notify({
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
      const dataURL = canvas.toDataURL("image/png");
      const data = dataURL.replace("data:image/png;base64,", "");
      resolve(new File([data], "server-icon.png"));
    };
    img.onerror = reject;
  });
  if (!file) return;
  await serverStore.uploadFile(serverId, "/server-icon.png", scaledFile);
  await serverStore.uploadFile(serverId, "/server-icon-original.png", file);

  refreshNuxtData("data");
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
  input.accept = "image/png,image/jpeg,image/gif,image/webp";
  input.onchange = uploadFile;
  input.click();
};
</script>
