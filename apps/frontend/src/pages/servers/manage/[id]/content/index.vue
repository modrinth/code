<template>
  <div class="h-full w-full">
    <div v-if="data && status == 'success'" class="flex h-full w-full flex-col gap-6 p-8">
      <div class="display-mode--list">
        <UiProjectCard
          v-if="data && data.project"
          :id="data.project.id"
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
      // action bar
      
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData(
  "serverData",
  async () => await serverStore.getServerData(serverId),
);
</script>
