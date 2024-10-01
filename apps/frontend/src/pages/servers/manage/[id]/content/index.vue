<template>
  <div class="h-full w-full">
    <div
      v-if="data && status == 'success' && version"
      class="flex h-full w-full flex-col gap-6 p-8"
    >
      <div class="display-mode--list">
        <UiProjectCard
          v-if="data && data.upstream"
          :id="data.upstream.project_id"
          :icon-url="data.project.icon_url"
          :name="data.project.title"
          :description="data.project.description"
          :client-side="data.project.client_side"
          :server-side="data.project.server_side"
          type="modpack"
          :show-updated-date="false"
          :show-created-date="false"
        />
      </div>
      <DropdownSelect v-model="version" :options="options" placeholder="Select version..." />
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { DropdownSelect } from "@modrinth/ui";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const { data, status } = await useLazyAsyncData("serverData", async () => {
  await serverStore.fetchServerData(serverId);
  return serverStore.getServerData(serverId);
});

const { data: versions, refresh: refreshVersions } = await useLazyAsyncData(
  "modpackVersions",
  async () => await useBaseFetch(`project/${data.value?.upstream?.project_id}/version`),
);

console.log(versions.value);
console.log(data.value);

const options = (versions.value as any[]).map((x) => x.version_number);

const version = ref<any>(null);
</script>
