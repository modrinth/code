<template>
  <div class="h-full w-full gap-2 overflow-y-auto px-4">
    <div class="card">
      <div class="flex flex-col gap-2">
        <div class="flex justify-between">
          <h2 class="text-3xl font-bold">SFTP</h2>
          <Button> Launch SFTP </Button>
        </div>
        <div class="flex w-full flex-col gap-2 rounded-xl bg-table-alternateRow p-4">
          <span class="font-bold text-contrast">
            sftp://geezyippeedrat@us-lax2.kyros.pyro.host:2022/
          </span>
          <span class="text-xs uppercase text-secondary">server address</span>
        </div>
        <div class="flex gap-2">
          <div class="flex w-full flex-col gap-2 rounded-xl bg-table-alternateRow p-4">
            <span class="font-bold text-contrast">geezyippeedrat</span>
            <span class="text-xs uppercase text-secondary">username</span>
          </div>
          <div class="flex w-full flex-col gap-2 rounded-xl bg-table-alternateRow p-4">
            <span class="font-bold text-contrast">/nEL8>]23&kM</span>
            <span class="text-xs uppercase text-secondary">password</span>
          </div>
        </div>
      </div>
    </div>
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
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";
const route = useNativeRoute();
const serverId = route.params.id as string;
const server = await usePyroServer(serverId, ["general"]);

const data = computed(() => server.general);

const properties = [
  { name: "Server ID", value: serverId ?? "Unknown" },
  { name: "Kind", value: data.value?.upstream?.kind ?? "Unknown" },
  { name: "Project ID", value: data.value?.upstream?.project_id ?? "Unknown" },
  { name: "Version ID", value: data.value?.upstream?.version_id ?? "Unknown" },
];
</script>
