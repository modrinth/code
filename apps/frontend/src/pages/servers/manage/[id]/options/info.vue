<template>
  <div class="h-full w-full overflow-y-auto">
    <div v-if="data" class="flex h-full w-full flex-col gap-2 px-4">
      <div class="card">
        <h2 class="text-3xl font-bold">Info</h2>
        <div class="rounded-xl bg-table-alternateRow p-4">
          <table
            class="min-w-full border-collapse overflow-hidden rounded-lg border-2 border-gray-300"
          >
            <tbody>
              <tr v-for="property in properties" :key="property.name">
                <td class="py-3">{{ property.name }}</td>
                <td class="px-4">
                  <UiCopyCode :text="property.value" />
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const data = computed(() => serverStore.serverData[serverId]);

const properties = [
  { name: "Server ID", value: serverId ?? "Unknown" },
  { name: "Kind", value: data.value?.upstream?.kind ?? "Unknown" },
  { name: "Project ID", value: data.value?.upstream?.project_id ?? "Unknown" },
  { name: "Version ID", value: data.value?.upstream?.version_id ?? "Unknown" },
];
</script>
