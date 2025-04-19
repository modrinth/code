<template>
  <NewModal ref="modal" header="Renaming backup" @show="focusInput">
    <div class="flex flex-col gap-2 md:w-[600px]">
      <label for="backup-name-input">
        <span class="text-lg font-semibold text-contrast"> Name </span>
      </label>
      <input
        id="backup-name-input"
        ref="input"
        v-model="backupName"
        type="text"
        class="bg-bg-input w-full rounded-lg p-4"
        :placeholder="`Backup #${backupNumber}`"
        maxlength="48"
      />
      <div v-if="nameExists" class="flex items-center gap-1">
        <IssuesIcon class="hidden text-orange sm:block" />
        <span class="text-sm text-orange">
          You already have a backup named '<span class="font-semibold">{{ trimmedName }}</span
          >'
        </span>
      </div>
    </div>
    <div class="mt-2 flex justify-start gap-2">
      <ButtonStyled color="brand">
        <button :disabled="isRenaming || nameExists" @click="renameBackup">
          <template v-if="isRenaming">
            <SpinnerIcon class="animate-spin" />
            Renaming...
          </template>
          <template v-else>
            <SaveIcon />
            Save changes
          </template>
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button @click="hide">
          <XIcon />
          Cancel
        </button>
      </ButtonStyled>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { ref, nextTick, computed } from "vue";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { SpinnerIcon, SaveIcon, XIcon, IssuesIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const modal = ref<InstanceType<typeof NewModal>>();
const input = ref<HTMLInputElement>();
const backupName = ref("");
const originalName = ref("");
const isRenaming = ref(false);

const currentBackup = ref<Backup | null>(null);

const trimmedName = computed(() => backupName.value.trim());

const nameExists = computed(() => {
  if (!props.server.backups?.data || trimmedName.value === originalName.value || isRenaming.value) {
    return false;
  }

  return props.server.backups.data.some(
    (backup) => backup.name.trim().toLowerCase() === trimmedName.value.toLowerCase(),
  );
});

const backupNumber = computed(
  () => (props.server.backups?.data?.findIndex((b) => b.id === currentBackup.value?.id) ?? 0) + 1,
);

const focusInput = () => {
  nextTick(() => {
    setTimeout(() => {
      input.value?.focus();
    }, 100);
  });
};

function show(backup: Backup) {
  currentBackup.value = backup;
  backupName.value = backup.name;
  originalName.value = backup.name;
  isRenaming.value = false;
  modal.value?.show();
}

function hide() {
  modal.value?.hide();
}

const renameBackup = async () => {
  if (!currentBackup.value) {
    addNotification({
      type: "error",
      title: "Error renaming backup",
      text: "Current backup is null",
    });
    return;
  }

  if (trimmedName.value === originalName.value) {
    hide();
    return;
  }

  isRenaming.value = true;
  try {
    let newName = trimmedName.value;

    if (newName.length === 0) {
      newName = `Backup #${backupNumber.value}`;
    }

    await props.server.backups?.rename(currentBackup.value.id, newName);
    hide();
    await props.server.refresh();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    addNotification({ type: "error", title: "Error renaming backup", text: message });
  } finally {
    hide();
    isRenaming.value = false;
  }
};

defineExpose({
  show,
  hide,
});
</script>
