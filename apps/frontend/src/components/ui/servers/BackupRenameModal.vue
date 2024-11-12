<template>
  <NewModal ref="modal" header="Renaming backup" @show="focusInput">
    <div class="flex flex-col gap-2 md:w-[600px]">
      <div class="font-semibold text-contrast">Name</div>
      <input
        ref="input"
        v-model="backupName"
        type="text"
        class="bg-bg-input w-full rounded-lg p-4"
        placeholder="e.g. Before 1.21"
      />
    </div>
    <div class="mb-1 mt-4 flex justify-start gap-4">
      <ButtonStyled color="brand">
        <button :disabled="isRenaming" @click="renameBackup">
          <SaveIcon />
          Rename backup
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button @click="hideModal">
          <XIcon />
          Cancel
        </button>
      </ButtonStyled>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { ref, nextTick } from "vue";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { SaveIcon, XIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
  currentBackupId: string;
}>();

const emit = defineEmits(["backupRenamed"]);

const modal = ref<InstanceType<typeof NewModal>>();
const input = ref<HTMLInputElement>();
const backupName = ref("");
const isRenaming = ref(false);
const backupError = ref<string | null>(null);

const focusInput = () => {
  nextTick(() => {
    setTimeout(() => {
      input.value?.focus();
    }, 100);
  });
};

const hideModal = () => {
  backupName.value = "";
  modal.value?.hide();
};

const renameBackup = async () => {
  if (!backupName.value.trim() || !props.currentBackupId) {
    emit("backupRenamed", { success: false, message: "Backup name cannot be empty" });
    return;
  }

  isRenaming.value = true;
  try {
    await props.server.backups?.rename(props.currentBackupId, backupName.value);
    await props.server.refresh();
    hideModal();
    emit("backupRenamed", { success: true, message: "Backup renamed successfully" });
  } catch (error) {
    backupError.value = error instanceof Error ? error.message : String(error);
    emit("backupRenamed", { success: false, message: backupError.value });
  } finally {
    isRenaming.value = false;
  }
};

defineExpose({
  show: () => modal.value?.show(),
  hide: hideModal,
});
</script>
