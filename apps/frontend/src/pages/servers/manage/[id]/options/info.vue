<template>
  <div class="h-full w-full">
    <div
      class="flex h-full w-full flex-col justify-between gap-6 p-8"
      v-if="data && status == 'success'"
    >
      <h2 class="text-3xl font-bold">Info</h2>
      <div class="flex h-full flex-col gap-4">
        <p class="m-0">Server ID: <UiCopyCode v-if="serverId" :text="serverId" /></p>
        <p class="m-0">Pack ID: <UiCopyCode v-if="data.modpack" :text="data.modpack" /></p>
      </div>
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { useServerStore } from "~/stores/servers.ts";

const { formatMessage } = useVIntl();
const messages = defineMessages({
  title: {
    id: "server.options.info.title",
    defaultMessage: "Info",
  },
});

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData(
  "infoServerData",
  async () => await serverStore.getServerData(serverId),
);
</script>
