<template>
  <div>
    <div v-if="data && status === 'success'">
      <section class="card">
        <h2 class="text-3xl font-bold">{{ formatMessage(messages.title) }}</h2>
        <div class="flex flex-col gap-2">
          <p class="m-0">Server ID: <UiCopyCode v-if="serverId" :text="serverId" /></p>
          <p class="m-0">Pack ID: <UiCopyCode v-if="data.modpack" :text="data.modpack" /></p>
        </div>
      </section>
    </div>
    <UiServersPyroLoading v-else-if="status === 'pending'" />
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
