<template>
  <div class="h-full w-full gap-2 overflow-y-auto">
    <div
      class="mb-4 flex h-full w-full items-center gap-2 rounded-xl border border-solid border-blue bg-bg-blue p-4 text-contrast"
    >
      <UnknownIcon class="h-8 w-8 text-blue" />
      SFTP is currently unavailable. This feature is under construction and will be available soon.
    </div>
    <div class="card">
      <div class="flex flex-col gap-4">
        <div class="flex justify-between">
          <label class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">SFTP</span>
            <span> SFTP is a way to access your server's files from outside of Modrinth. </span>
          </label>
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
            <span class="font-bold text-contrast"> {{ data?.sftp_username }} </span>
            <span class="text-xs uppercase text-secondary">username</span>
          </div>
          <div class="flex w-full flex-col gap-2 rounded-xl bg-table-alternateRow p-4">
            <span class="font-bold text-contrast">
              {{ data?.sftp_password }}
            </span>
            <span class="text-xs uppercase text-secondary">password</span>
          </div>
        </div>
      </div>
    </div>
    <div class="card">
      <h2 class="text-xl font-bold">Info</h2>
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
import { UnknownIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";
const route = useNativeRoute();
const serverId = route.params.id as string;

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const data = computed(() => props.server.general);

const properties = [
  { name: "Server ID", value: serverId ?? "Unknown" },
  { name: "Kind", value: data.value?.upstream?.kind ?? "Unknown" },
  { name: "Project ID", value: data.value?.upstream?.project_id ?? "Unknown" },
  { name: "Version ID", value: data.value?.upstream?.version_id ?? "Unknown" },
];
</script>
