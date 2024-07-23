<template>
  <div>
    <section class="card">
      <h2 class="text-3xl font-bold">{{ formatMessage(messages.title) }}</h2>
      <label for="username-field">
        <span class="label__title">{{ formatMessage(messages.servernameTitle) }}</span>
        <span class="label__description">{{ formatMessage(messages.servernameDescription) }}</span>
      </label>
      <input v-model="newName" :placeholder="data.name" />
      <button type="submit" class="btn btn-primary" @click="() => updateServerName()">Save</button>
    </section>
  </div>
</template>

<script setup lang="ts">
import type { Server } from "~/types/servers";
import { ref } from "vue";

const { formatMessage } = useVIntl();

const route = useNativeRoute();
const serverId = route.params.id;

const auth = await useAuth();

const messages = defineMessages({
  title: {
    id: "server.options.general.title",
    defaultMessage: "General",
  },
  description: {
    id: "server.options.general.description",
    defaultMessage: "Your server settings.",
  },
  servernameTitle: {
    id: "server.options.general.servername.title",
    defaultMessage: "Server Name",
  },
  servernameDescription: {
    id: "server.options.general.servername.description",
    defaultMessage: "A name to help identify your server.",
  },
});

const { data, status } = await useLazyAsyncData("Server", async () => {
  return await usePyroFetch<Server>(auth.value.token, `servers/${serverId}`);
});

const newName = ref("");

const updateServerName = async () => {
  await usePyroFetch(auth.value.token, `servers/${serverId}/name`, 0, "POST", "application/json", {
    name: newName.value,
  });
};
</script>
