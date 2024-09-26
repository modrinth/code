<template>
  <div class="relative h-full w-full">
    <div v-if="data && status == 'success'" class="flex h-full w-full flex-col gap-6 p-8">
      <h2 class="text-3xl font-bold">Startup</h2>
    </div>
    <UiServersPyroLoading v-else />
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        v-if="hasUnsavedChanges"
        :is-updating="isUpdating"
        :save="saveStartup"
        :reset="resetStartup"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData(
  "data",
  async () => await serverStore.getServerData(serverId),
);

const isUpdating = ref(false);
const hasUnsavedChanges = ref(false);

const saveStartup = async () => {
  try {
    isUpdating.value = true;
    await new Promise((resolve) => setTimeout(resolve, 500));
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
    await refreshNuxtData("data");
  } catch (error) {
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "error",
      title: "Failed to update server arguments",
      text: "Please try again later.",
    });
  } finally {
    isUpdating.value = false;
  }
};

const resetStartup = () => {};
</script>
