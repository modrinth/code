<template>
  <div class="relative h-full w-full">
    <div
      v-if="data && status == 'success'"
      class="flex h-full w-full flex-col justify-between gap-6 p-8"
    >
      <h2 class="text-3xl font-bold">General</h2>
      <div class="flex h-full flex-col gap-2">
        <div class="card flex flex-col gap-4">
          <label for="username-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Server name</span>
            <span> Change the name of your server as it appears on Modrinth </span>
          </label>
          <input v-model="serverName" class="h-[50%] w-[40%]" @keyup.enter="saveGeneral" />
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        v-if="hasUnsavedChanges"
        :isUpdating="isUpdating"
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

const { data, status } = await useAsyncData(
  "data",
  async () => await serverStore.getServerData(serverId),
);

const serverName = ref(data.value?.name || "");

const isUpdating = ref(false);
const hasUnsavedChanges = computed(() => serverName.value !== data.value?.name);

const saveGeneral = async () => {
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
</script>
