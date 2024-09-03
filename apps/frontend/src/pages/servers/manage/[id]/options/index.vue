<template>
  <div>
    <div v-if="data && status == 'success'">
      <section class="card">
        <div class="flex flex-col gap-6">
          <h2 class="text-3xl font-bold">General</h2>
          <div class="h-[2px] w-full bg-divider"></div>
          <div class="ml-1 flex flex-col gap-4">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-white">Server Name</span>
              <span> Change the name of your server as it appears on Modrinth </span>
            </label>
            <input v-model="serverName" @keyup.enter="updateServerNameReq" class="h-[50%] w-[40%]" />
          </div>
          <div class="h-[2px] w-full bg-divider"></div>
          <button
            type="submit"
            class="btn btn-primary"
            @click="updateServerNameReq"
            :disabled="isUpdating || !hasUnsavedChanges"
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

const app = useNuxtApp();
const route = useRoute();
const serverId = route.params.id as string;

const serverStore = useServerStore();

const isUpdating = ref(false);
const { data, status } = await useAsyncData(
  "data",
  async () => await serverStore.getServerData(serverId),
);

const serverName = ref(data.value?.name || "");

const hasUnsavedChanges = computed(() => serverName.value !== data.value?.name);

const updateServerNameReq = async () => {
  try {
    isUpdating.value = true;
    await serverStore.updateServerName(serverId, serverName.value);
    await new Promise((resolve) => setTimeout(resolve, 500));
    await refreshNuxtData("backupsData");
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
  } catch (error) {
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "error",
      title: "Failed to update server settings",
      text: "An error occurred while attempting to update your server settings.",
    });
  } finally {
    isUpdating.value = false;
  }
};
</script>
