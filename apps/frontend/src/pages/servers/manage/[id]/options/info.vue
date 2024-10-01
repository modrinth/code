<template>
  <div class="h-full w-full">
    <div v-if="data && status == 'success'" class="flex h-full w-full flex-col gap-2 p-8">
      <h2 class="text-3xl font-bold">Info</h2>
      <table
        class="min-w-full border-collapse overflow-hidden rounded-lg border-2 border-gray-300 shadow-lg"
      >
        <tbody>
          <tr
            v-for="property in properties"
            :key="property.name"
            class="border-0 border-b border-solid border-bg-raised"
          >
            <td class="py-3">{{ property.name }}</td>
            <td class="px-4 py-3"><UiCopyCode :text="property.value" /></td>
          </tr>
        </tbody>
      </table>
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
  "infoServerData",
  async () => await serverStore.getServerData(serverId),
);

const properties = [
  { name: "Server ID", value: serverId },
  { name: "IP", value: data.value?.net?.ip },
  { name: "Port", value: data.value?.net?.port },
  { name: "Kind", value: data.value?.upstream?.kind },
  { name: "Project ID", value: data.value?.upstream?.project_id },
  { name: "Version ID", value: data.value?.upstream?.version_id },
];
</script>
