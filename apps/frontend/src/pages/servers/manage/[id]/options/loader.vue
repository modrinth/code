<template>
  <ServerInstallation
    :server="props.server"
    :backup-in-progress="props.backupInProgress"
    @reinstall="emit('reinstall')"
  />
</template>

<script setup lang="ts">
import type { BackupInProgressReason } from "~/pages/servers/manage/[id].vue";
import ServerInstallation from "~/components/ui/servers/ServerInstallation.vue";
import { ModrinthServer } from "~/composables/servers/modrinth-servers.ts";

const props = defineProps<{
  server: ModrinthServer;
  backupInProgress?: BackupInProgressReason;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();
</script>
