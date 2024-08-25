<template>
  <div>
    <div v-if="data && status === 'success'">
      <section class="card">
        <div class="flex flex-col gap-6">
          <h2 class="text-3xl font-bold">Network</h2>
          <div class="h-[2px] w-full bg-divider"></div>
          <div class="ml-1 flex items-center justify-between">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-white">Subdomain</span>
              <span> Change the subdomain to connect to your server </span>
            </label>
            <div class="flex items-center gap-2">
              <input
                v-model="serverSubdomain"
                @keyup.enter="updateServerData"
                class="h-[50%] w-[63%]"
              />
              .{{ data.net.domain.split(".").slice(1).join(".") }}
            </div>
          </div>
          <div class="h-[2px] w-full bg-divider"></div>
          <button
            type="submit"
            class="btn btn-primary"
            @click=""
            :disabled="isUpdating || !hasUnsavedChanges"
          >
            {{ isUpdating ? "Saving..." : "Save" }}
          </button>
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
const { data, status } = await useAsyncData(
  "data",
  async () => await serverStore.getServerData(serverId),
);

const serverSubdomain = ref(data?.value?.net?.domain.split(".")[0] ?? "");

const hasUnsavedChanges = computed(
  () => serverSubdomain.value !== data?.value?.net?.domain.split(".")[0],
);

const updateServerData = async () => {
  try {
    isUpdating.value = true;
    const available = (await serverStore.checkSubdomainAvailability(serverSubdomain.value)) as {
      available: boolean;
    };
    if (!available.available) {
      // @ts-ignore
      app.$notify({
        group: "serverOptions",
        type: "error",
        title: "Subdomain is taken",
        text: "The subdomain you entered is already in use.",
      });
    } else {
      await serverStore.changeSubdomain(serverId, serverSubdomain.value);
    }

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
