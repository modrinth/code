<template>
  <ConfirmModal
    ref="modal"
    danger
    title="Are you sure you want to restore from this backup?"
    proceed-label="Restore from backup"
    description="This will **overwrite all files on your server** and replace them with the files from the backup."
    @proceed="restoreBackup"
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
import { ConfirmModal, NewModal } from "@modrinth/ui";
import type { Server } from "~/composables/pyroServers";
import BackupItem from "~/components/ui/servers/BackupItem.vue";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const modal = ref<InstanceType<typeof NewModal>>();
const currentBackup = ref<Backup | null>(null);

function show(backup: Backup) {
  currentBackup.value = backup;
  modal.value?.show();
}

const restoreBackup = async () => {
  if (!currentBackup.value) {
    addNotification({
      type: "error",
      title: "Failed to restore backup",
      text: "Current backup is null",
    });
    return;
  }

  try {
    await props.server.backups?.restore(currentBackup.value.id);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    addNotification({ type: "error", title: "Failed to restore backup", text: message });
  }
};

defineExpose({
  show,
});
</script>
