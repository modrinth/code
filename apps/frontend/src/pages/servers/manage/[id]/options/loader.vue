<template>
  <ServerInstallation
    :server="props.server"
    :backup-in-progress="props.backupInProgress"
    :tags="tags"
    @reinstall="emit('reinstall')"
  />
</template>

<script setup lang="ts">
import { ModrinthServer } from "@modrinth/ui";
import type { BackupInProgressReason } from "@modrinth/utils";
import ServerInstallation from "~/components/ui/servers/ServerInstallation.vue";

const tags = useTags();

const props = defineProps<{
  server: ModrinthServer;
  backupInProgress?: BackupInProgressReason;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();
</script>
