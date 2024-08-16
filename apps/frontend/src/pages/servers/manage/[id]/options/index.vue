<template>
  <div>
    <div v-if="serverData && !isLoading">
      <section class="card">
        <h2 class="text-3xl font-bold">{{ $t(messages.title) }}</h2>
        <label for="username-field">
          <span class="label__title">{{ $t(messages.servernameTitle) }}</span>
          <span class="label__description">
            {{ $t(messages.servernameDescription) }}
          </span>
        </label>
        <input v-model="newName" :placeholder="serverData.name" @keyup.enter="updateServerName" />
        <button
          type="submit"
          class="btn btn-primary"
          @click="updateServerName"
          :disabled="isUpdating"
        >
          {{ isUpdating ? "Saving..." : "Save" }}
        </button>
      </section>
    </div>
    <PyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";
import type { Server } from "~/types/servers";

const app = useNuxtApp();
const route = useRoute();
const serverId = route.params.id as string;

const serverStore = useServerStore();

const messages = {
  title: "General",
  description: "Settings that affect your server globally.",
  servernameTitle: "Server Name",
  servernameDescription: "Change the name of your server as it appears on Modrinth",
};

const newName = ref("");
const isLoading = ref(true);
const isUpdating = ref(false);
const serverData = ref<Server | null>(null);

const fetchServerData = async () => {
  try {
    isLoading.value = true;
    await serverStore.fetchServerData(serverId);
    serverData.value = serverStore.getServerData(serverId) ?? null;
  } catch (error) {
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "error",
      title: "Failed to fetch server data",
      text: "Please try again later.",
    });
  } finally {
    isLoading.value = false;
  }
};

const updateServerName = async () => {
  if (!newName.value.trim()) return;

  try {
    isUpdating.value = true;
    await serverStore.updateServerName(serverId, newName.value);
    await fetchServerData();
    newName.value = "";
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "success",
      title: "Server name updated",
      text: "Your server name has been successfully changed.",
    });
  } catch (error) {
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "error",
      title: "Could not update server name",
      text: "Your server name could not be changed. Please try again later.",
    });
  } finally {
    isUpdating.value = false;
  }
};

onMounted(fetchServerData);
</script>
