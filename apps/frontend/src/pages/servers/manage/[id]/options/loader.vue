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
import type { Server } from "~/composables/servers/contentType.ts";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
  backupInProgress?: BackupInProgressReason;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();
</script>
