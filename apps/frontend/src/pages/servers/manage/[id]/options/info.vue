<template>
  <div>
    <div v-if="data && status === 'success'">
      <section class="card">
        <h2 class="text-3xl font-bold">{{ formatMessage(messages.title) }}</h2>
        <div class="flex flex-col gap-2">
          <p>Server ID: <CopyCode v-if="serverId" :text="serverId" /></p>
          <p>Pack ID: <CopyCode v-if="data.modpack_id" :text="data.modpack_id" /></p>
        </div>
      </section>
    </div>
    <PyroLoading v-else-if="status === 'pending'" />
  </div>
</template>

<script setup lang="ts">
import CopyCode from "~/components/ui/CopyCode.vue";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";
import { useServerStore } from "~/stores/servers";

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
const { data, status } = await useLazyAsyncData("infoServerData", async () =>
  serverStore.getServerData(serverId),
);
</script>
