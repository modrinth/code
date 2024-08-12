<template>
  <div>
    <div v-if="data && status === 'success'">
      <section class="card">
        <h2 class="text-3xl font-bold">{{ formatMessage(messages.title) }}</h2>
        <label for="username-field">
          <span class="label__title">{{ formatMessage(messages.servernameTitle) }}</span>
          <span class="label__description">{{
            formatMessage(messages.servernameDescription)
          }}</span>
        </label>
        <input v-model="newName" :placeholder="data.name" />
        <button type="submit" class="btn btn-primary" @click="() => updateServerName()">
          Save
        </button>
      </section>
    </div>
    <PyroLoading v-else-if="status === 'pending'" />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";
import type { Server } from "~/types/servers";

const { formatMessage } = useVIntl();

const route = useNativeRoute();
const serverId = route.params.id;

const app = useNuxtApp();
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
  try {
    await usePyroFetch(
      auth.value.token,
      `servers/${serverId}/name`,
      0,
      "POST",
      "application/json",
      {
        name: newName.value,
      },
    );
  } catch (error) {
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "error",
      title: "Could not update server name",
      text: "Your server name could not be changed. Please try again later.",
    });
  }
};
</script>
