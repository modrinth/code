<template>
  <ConfirmModal
    ref="modal"
    danger
    title="Are you sure you want to delete this backup?"
    proceed-label="Delete backup"
    :confirmation-text="currentBackup?.name ?? 'null'"
    has-to-type
    @proceed="emit('delete', currentBackup)"
  >
    <BackupItem
      v-if="currentBackup"
      :backup="currentBackup"
      preview
      class="border-px border-solid border-button-border"
    />
  </ConfirmModal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { ConfirmModal } from "@modrinth/ui";
import type { Server } from "~/composables/pyroServers";
import BackupItem from "~/components/ui/servers/BackupItem.vue";

defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const emit = defineEmits<{
  (e: "delete", backup: Backup | undefined): void;
}>();

const modal = ref<InstanceType<typeof ConfirmModal>>();
const currentBackup = ref<Backup | undefined>(undefined);

function show(backup: Backup) {
  currentBackup.value = backup;
  modal.value?.show();
}

defineExpose({
  show,
});
</script>
