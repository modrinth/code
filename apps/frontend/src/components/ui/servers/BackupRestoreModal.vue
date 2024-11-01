<template>
  <NewModal ref="modal" header="Restoring backup">
    <div class="flex flex-col gap-4">
      <div class="relative flex w-full flex-col gap-2 rounded-2xl bg-bg p-6">
        <div class="text-2xl font-extrabold text-contrast">
          {{ backupName }}
        </div>
        <div class="flex gap-2 font-semibold text-contrast">
          <CalendarIcon />
          {{ formattedDate }}
        </div>
      </div>
    </div>
    <div class="mb-1 mt-4 flex justify-end gap-4">
      <ButtonStyled color="brand">
        <button :disabled="isRestoring" @click="restoreBackup">Restore backup</button>
      </ButtonStyled>
      <ButtonStyled type="transparent">
        <button @click="hideModal">Cancel</button>
      </ButtonStyled>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { CalendarIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
  backupId: string;
  backupName: string;
  backupCreatedAt: string;
}>();

const emit = defineEmits(["backupRestored"]);

const modal = ref<InstanceType<typeof NewModal>>();
const isRestoring = ref(false);
const backupError = ref<string | null>(null);

const formattedDate = computed(() => {
  return new Date(props.backupCreatedAt).toLocaleString("en-US", {
    month: "numeric",
    day: "numeric",
    year: "2-digit",
    hour: "numeric",
    minute: "numeric",
    hour12: true,
  });
});

const hideModal = () => {
  modal.value?.hide();
};

const restoreBackup = async () => {
  if (!props.backupId) {
    emit("backupRestored", { success: false, message: "No backup selected" });
    return;
  }

  isRestoring.value = true;
  try {
    await props.server.backups?.restore(props.backupId);
    hideModal();
    emit("backupRestored", { success: true, message: "Backup restored successfully" });
  } catch (error) {
    backupError.value = error instanceof Error ? error.message : String(error);
    emit("backupRestored", { success: false, message: backupError.value });
  } finally {
    isRestoring.value = false;
  }
};

defineExpose({
  show: () => modal.value?.show(),
  hide: hideModal,
});
</script>
