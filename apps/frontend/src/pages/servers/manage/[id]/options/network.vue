<template>
  <div class="relative h-full w-full">
    <div
      v-if="data && status == 'success'"
      class="flex h-full w-full flex-col justify-between gap-6 p-8"
    >
      <h2 class="text-3xl font-bold">Network</h2>
      <div class="flex h-full flex-col gap-2">
        <div class="card flex flex-col gap-4">
          <label for="username-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Subdomain</span>
            <span> Change the subdomain to connect to your server </span>
          </label>
          <div class="flex w-full items-center gap-2 md:w-[60%]">
            <input v-model="serverSubdomain" class="h-[50%] w-[63%]" @keyup.enter="saveNetwork" />
            .modrinth.gg
          </div>
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        v-if="hasUnsavedChanges"
        :is-updating="isUpdating"
        :save="saveNetwork"
        :reset="resetNetwork"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useServerStore } from "~/stores/servers.ts";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const isUpdating = ref(false);
const { data, status } = await useLazyAsyncData("networkData", async () => {
  await serverStore.fetchServerData(serverId);
  return serverStore.getServerData(serverId);
});

const serverSubdomain = ref(data?.value?.net?.domain ?? "");

const hasUnsavedChanges = computed(() => serverSubdomain.value !== data?.value?.net?.domain);

const saveNetwork = async () => {
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
        title: "Subdomain not available",
        text: "The subdomain you entered is already in use.",
      });
      return;
    }
    await serverStore.changeSubdomain(serverId, serverSubdomain.value);
    await new Promise((resolve) => setTimeout(resolve, 500));
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
    resetNetwork();
    await refreshNuxtData("networkData");
    resetNetwork();
  }
};

const resetNetwork = () => {
  serverSubdomain.value = data?.value?.net?.domain ?? "";
};
</script>
