<template>
  <div class="h-full w-full">
    <div class="h-full w-full gap-2 overflow-y-auto">
      <div class="card flex flex-col gap-4">
        <h1 class="m-0 text-2xl font-bold">Server Preferences</h1>
        <div v-for="(value, key) in userPreferences" :key="key" class="flex justify-between">
          <label for="server-name-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">{{ preferences[key].displayName }}</span>
            <span> {{ preferences[key].description }} </span>
          </label>
          <input v-model="newUserPreferences[key]" class="switch stylized-toggle" type="checkbox" />
        </div>
      </div>
    </div>
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        :is-visible="!!hasUnsavedChanges"
        :server="props.server"
        :is-updating="false"
        :save="savePreferences"
        :reset="resetPreferences"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useStorage } from "@vueuse/core";
import type { Server } from "~/composables/pyroServers";

const route = useNativeRoute();
const serverId = route.params.id as string;
const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const hasUnsavedChanges = computed(() => {
  return JSON.stringify(newUserPreferences.value) !== JSON.stringify(userPreferences.value);
});

const savePreferences = () => {
  userPreferences.value = newUserPreferences.value;
  newUserPreferences.value = JSON.parse(JSON.stringify(userPreferences.value));
  addNotification({
    group: "serverOptions",
    type: "success",
    title: "Preferences saved",
    text: "Your preferences have been saved.",
  });
};

const resetPreferences = () => {
  newUserPreferences.value = userPreferences.value;
};

const preferences = {
  ramAsNumber: {
    displayName: "RAM as bytes",
    description: "Display RAM usage in number of bytes instead of percentage",
  },
  autoRestart: {
    displayName: "Auto restart (not implemented)",
    description: "Automatically restart the server if it crashes",
  },
  backupWhileRunning: {
    displayName: "Backup while running (not implemented)",
    description:
      "Allow creation of a backup without stopping the server. This may lead to corrupted backups, use with caution",
  },
};

const defaultPreferences = {
  ramAsNumber: false,
  autoRestart: false,
  backupWhileRunning: false,
};

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, defaultPreferences);
const newUserPreferences = ref(JSON.parse(JSON.stringify(userPreferences.value)));
</script>
