<template>
  <NewModal ref="modal" header="Creating backup" @show="focusInput">
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
        :placeholder="`Backup #${newBackupAmount}`"
        maxlength="48"
      />
      <div v-if="nameExists && !isCreating" class="flex items-center gap-1">
        <IssuesIcon class="hidden text-orange sm:block" />
        <span class="text-sm text-orange">
          You already have a backup named '<span class="font-semibold">{{ trimmedName }}</span
          >'
        </span>
      </div>
      <div v-if="isRateLimited" class="mt-2 text-sm text-red">
        You're creating backups too fast. Please wait a moment before trying again.
      </div>
    </div>
    <div class="mt-2 flex justify-start gap-2">
      <ButtonStyled color="brand">
        <button :disabled="isCreating || nameExists" @click="createBackup">
          <PlusIcon />
          Create backup
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
import { ref, nextTick, computed } from "vue";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { IssuesIcon, PlusIcon, XIcon } from "@modrinth/assets";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const modal = ref<InstanceType<typeof NewModal>>();
const input = ref<HTMLInputElement>();
const isCreating = ref(false);
const isRateLimited = ref(false);
const backupName = ref("");
const newBackupAmount = computed(() =>
  props.server.backups?.data?.length === undefined ? 1 : props.server.backups?.data?.length + 1,
);

const trimmedName = computed(() => backupName.value.trim());

const nameExists = computed(() => {
  if (!props.server.backups?.data) return false;
  return props.server.backups.data.some(
    (backup) => backup.name.trim().toLowerCase() === trimmedName.value.toLowerCase(),
  );
});

const focusInput = () => {
  nextTick(() => {
    setTimeout(() => {
      input.value?.focus();
    }, 100);
  });
};

function show() {
  backupName.value = "";
  isCreating.value = false;
  modal.value?.show();
}

const hideModal = () => {
  modal.value?.hide();
};

const createBackup = async () => {
  if (backupName.value.trim().length === 0) {
    backupName.value = `Backup #${newBackupAmount.value}`;
  }

  isCreating.value = true;
  isRateLimited.value = false;
  try {
    await props.server.backups?.create(trimmedName.value);
    hideModal();
    await props.server.refresh();
  } catch (error) {
    if (error instanceof PyroFetchError && error.statusCode === 429) {
      isRateLimited.value = true;
      addNotification({
        type: "error",
        title: "Error creating backup",
        text: "You're creating backups too fast.",
      });
    } else {
      const message = error instanceof Error ? error.message : String(error);
      addNotification({ type: "error", title: "Error creating backup", text: message });
    }
  } finally {
    isCreating.value = false;
  }
};

defineExpose({
  show,
  hide: hideModal,
});
</script>
