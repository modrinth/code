<template>
  <div class="h-full w-full">
    <div
      v-if="data && status == 'success'"
      class="flex h-full w-full flex-col justify-between gap-6 p-8"
    >
      <h2 class="text-3xl font-bold">General</h2>
      <div class="flex h-full flex-col gap-2">
        <div class="card flex flex-col gap-4">
          <label for="username-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-white">Server name</span>
            <span> Change the name of your server as it appears on Modrinth </span>
          </label>
          <input v-model="serverName" class="h-[50%] w-[40%]" @keyup.enter="updateServerNameReq" />
        </div>
      </div>
      <button
        type="submit"
        class="btn btn-primary"
        :disabled="isUpdating || !hasUnsavedChanges"
        @click="updateServerNameReq"
      >
        {{ isUpdating ? "Saving..." : "Save" }}
      </button>
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

const app = useNuxtApp();
const route = useRoute();
const serverId = route.params.id as string;

const serverStore = useServerStore();

const isUpdating = ref(false);
const { data, status } = await useAsyncData(
  "data",
  async () => await serverStore.getServerData(serverId),
);

const serverName = ref(data.value?.name || "");

const hasUnsavedChanges = computed(() => serverName.value !== data.value?.name);

const updateServerNameReq = async () => {
  try {
    isUpdating.value = true;
    await serverStore.updateServerName(serverId, serverName.value);
    await new Promise((resolve) => setTimeout(resolve, 500));
    await refreshNuxtData("backupsData");
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
  } catch (error) {
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
</script>
