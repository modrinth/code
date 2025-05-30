<template>
  <div class="h-full w-full">
    <div class="h-full w-full gap-2 overflow-y-auto">
      <div class="card flex flex-col gap-4">
        <h1 class="m-0 text-lg font-bold text-contrast">Server preferences</h1>
        <p class="m-0">Preferences apply per server and changes are only saved in your browser.</p>
        <div
          v-for="(prefConfig, key) in preferences"
          :key="key"
          class="flex items-center justify-between gap-2"
        >
          <label :for="`pref-${key}`" class="flex flex-col gap-2">
            <div class="flex flex-row gap-2">
              <span class="text-lg font-bold text-contrast">{{ prefConfig.displayName }}</span>
              <div
                v-if="prefConfig.implemented === false"
                class="hidden items-center gap-1 rounded-full bg-table-alternateRow p-1 px-1.5 text-xs font-semibold sm:flex"
              >
                Coming Soon
              </div>
            </div>
            <span>{{ prefConfig.description }}</span>
          </label>
          <input
            :id="`pref-${key}`"
            v-model="newUserPreferences[key]"
            class="switch stylized-toggle flex-none"
            type="checkbox"
            :disabled="prefConfig.implemented === false"
          />
        </div>
      </div>
    </div>
    <UiServersSaveBanner
      :is-visible="hasUnsavedChanges"
      :server="props.server"
      :is-updating="false"
      :save="savePreferences"
      :reset="resetPreferences"
    />
  </div>
</template>

<script setup lang="ts">
import { useStorage } from "@vueuse/core";
import type { Server } from "~/composables/pyroServers";

const route = useNativeRoute();
const serverId = route.params.id as string;

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const preferences = {
  ramAsNumber: {
    displayName: "RAM as bytes",
    description:
      "When enabled, RAM will be displayed as bytes instead of a percentage in your server's Overview.",
    implemented: true,
  },
  hideSubdomainLabel: {
    displayName: "Hide subdomain label",
    description: "When enabled, the subdomain label will be hidden from the server header.",
    implemented: true,
  },
  autoRestart: {
    displayName: "Auto restart",
    description: "When enabled, your server will automatically restart if it crashes.",
    implemented: false,
  },
  powerDontAskAgain: {
    displayName: "Power actions confirmation",
    description: "When enabled, you will be prompted before stopping and restarting your server.",
    implemented: true,
  },
  backupWhileRunning: {
    displayName: "Create backups while running",
    description: "When enabled, backups will be created even if the server is running.",
    implemented: true,
  },
} as const;

type PreferenceKeys = keyof typeof preferences;

type UserPreferences = {
  [K in PreferenceKeys]: boolean;
};

const defaultPreferences: UserPreferences = {
  ramAsNumber: false,
  hideSubdomainLabel: false,
  autoRestart: false,
  powerDontAskAgain: false,
  backupWhileRunning: false,
};

const userPreferences = useStorage<UserPreferences>(
  `pyro-server-${serverId}-preferences`,
  defaultPreferences,
);

const newUserPreferences = ref<UserPreferences>(JSON.parse(JSON.stringify(userPreferences.value)));

const hasUnsavedChanges = computed(() => {
  return JSON.stringify(newUserPreferences.value) !== JSON.stringify(userPreferences.value);
});

const savePreferences = () => {
  userPreferences.value = { ...newUserPreferences.value };
  addNotification({
    group: "serverOptions",
    type: "success",
    title: "Preferences saved",
    text: "Your preferences have been saved.",
  });
};

const resetPreferences = () => {
  newUserPreferences.value = { ...userPreferences.value };
};
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
