<template>
  <div>
    <div v-if="data && status === 'success'">
      <section class="card">
        <div class="flex flex-col gap-6">
          <h2 class="text-3xl font-bold">Launch Arguments</h2>
          <div class="h-[2px] w-full bg-divider"></div>
        </div>
      </section>
    </div>
    <UiServersPyroLoading v-else-if="status === 'pending'" />
  </div>
</template>

<script setup lang="ts">
import { useServerStore } from "~/stores/servers.ts";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();
const isUpdating = ref(false);
const temdata = ref("");

const updateServerArgs = async () => {
  try {
    isUpdating.value = true;
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

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData(
  "infoServerData",
  async () => await serverStore.getServerData(serverId),
);
</script>
