<template>
  <NewModal ref="modal" danger header="Deleting backup">
    <div class="flex flex-col gap-4">
      <div class="relative flex w-full flex-col gap-2 rounded-2xl bg-[#0e0e0ea4] p-6">
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
      <ButtonStyled color="red">
        <button :disabled="isDeleting" @click="deleteBackup">
          <TrashIcon />
          Delete backup
        </button>
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
import { TrashIcon, CalendarIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
  backupId: string;
  backupName: string;
  backupCreatedAt: string;
}>();

const emit = defineEmits(["backupDeleted"]);

const modal = ref<InstanceType<typeof NewModal>>();
const isDeleting = ref(false);
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

const deleteBackup = async () => {
  if (!props.backupId) {
    emit("backupDeleted", { success: false, message: "No backup selected" });
    return;
  }

  isDeleting.value = true;
  try {
    await props.server.backups?.delete(props.backupId);
    await props.server.refresh();
    hideModal();
    emit("backupDeleted", { success: true, message: "Backup deleted successfully" });
  } catch (error) {
    backupError.value = error instanceof Error ? error.message : String(error);
    emit("backupDeleted", { success: false, message: backupError.value });
  } finally {
    isDeleting.value = false;
  }
};

defineExpose({
  show: () => modal.value?.show(),
  hide: hideModal,
});
</script>
