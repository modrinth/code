<template>
  <ServerInstallation
    :server="props.server"
    :backup-in-progress="props.backupInProgress"
    @reinstall="emit('reinstall')"
  />
</template>

<script setup lang="ts">
import { ModrinthServer } from "@modrinth/ui";
import type { BackupInProgressReason } from "~/pages/servers/manage/[id].vue";
import ServerInstallation from "~/components/ui/servers/ServerInstallation.vue";

const props = defineProps<{
  server: ModrinthServer;
  backupInProgress?: BackupInProgressReason;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();
</script>
