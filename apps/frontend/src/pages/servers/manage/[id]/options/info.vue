<template>
  <section class="card">
    <h2 class="text-3xl font-bold">{{ formatMessage(messages.title) }}</h2>
    <div class="flex flex-col gap-2">
      <p>Server ID: <CopyCode v-if="serverId" :text="serverId" /></p>
      <p>Pack ID: <CopyCode v-if="serverId" :text="packId" /></p>
    </div>
  </section>
</template>

<script setup lang="ts">
import CopyCode from "~/components/ui/CopyCode.vue";

const { formatMessage } = useVIntl();
const messages = defineMessages({
  title: {
    id: "server.options.info.title",
    defaultMessage: "Info",
  },
});

const route = useNativeRoute();
const serverId = route.params.id as string;
const packId = "somepackid";
</script>
