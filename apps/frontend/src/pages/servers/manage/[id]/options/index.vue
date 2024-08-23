<template>
  <div>
    <div v-if="serverData && !isLoading">
      <section class="card">
        <div class="flex flex-col gap-6">
          <h2 class="text-3xl font-bold">General</h2>
          <div class="h-[2px] w-full bg-divider"></div>
          <div class="ml-1 flex flex-col gap-4">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-white">Server Name</span>
              <span> Change the name of your server as it appears on Modrinth </span>
            </label>
            <input
              v-model="newName"
              :placeholder="serverData.name"
              @keyup.enter="updateServerName"
              class="h-[50%] w-[40%]"
            />
          </div>
          <div class="h-[2px] w-full bg-divider"></div>
          <button
            type="submit"
            class="btn btn-primary"
            @click="updateServerName"
            :disabled="isUpdating"
          >
            {{ isUpdating ? "Saving..." : "Save" }}
          </button>
        </div>
      </section>
    </div>
    <PyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";
import type { Server } from "~/types/servers";
import { EditIcon } from "@modrinth/assets";
import { Button, SearchDropdown } from "@modrinth/ui";
import { useFetch } from "#app";
import type { subtle } from "node:crypto";
import Checkbox from "~/components/ui/Checkbox.vue";

const app = useNuxtApp();
const route = useRoute();
const serverId = route.params.id as string;

const serverStore = useServerStore();

const newName = ref("");
const isLoading = ref(true);
const isUpdating = ref(false);
const serverData = ref<Server | null>(null);
const versions = ref<any>([]);
const version = ref<string | null>(serverData.value?.project?.versions[0] ?? null);

const fetchServerData = async () => {
  try {
    isLoading.value = true;
    await serverStore.fetchServerData(serverId);
    serverData.value = serverStore.getServerData(serverId) ?? null;
    versions.value = await useBaseFetch(`project/${serverData.value?.project?.id}/version`);
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
    await new Promise((resolve) => setTimeout(resolve, 500));
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
