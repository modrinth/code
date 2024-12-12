<template>
  <NewModal ref="modal" header="Creating backup" @show="focusInput">
    <div class="flex flex-col gap-2 md:w-[600px]">
      <div class="font-semibold text-contrast">Name</div>
      <input
        ref="input"
        v-model="backupName"
        type="text"
        class="bg-bg-input w-full rounded-lg p-4"
        placeholder="e.g. Before 1.21"
        maxlength="64"
      />
      <div class="flex items-center gap-2">
        <InfoIcon class="hidden sm:block" />
        <span class="text-sm text-secondary">
          If left empty, the backup name will default to
          <span class="font-semibold"> Backup #{{ newBackupAmount }}</span>
        </span>
      </div>
      <div v-if="isRateLimited" class="mt-2 text-sm text-red">
        You're creating backups too fast. Please wait a moment before trying again.
      </div>
    </div>
    <div class="mb-1 mt-4 flex justify-start gap-4">
      <ButtonStyled color="brand">
        <button :disabled="isCreating" @click="createBackup">
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
import { PlusIcon, XIcon, InfoIcon } from "@modrinth/assets";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const emit = defineEmits(["backupCreated"]);

const modal = ref<InstanceType<typeof NewModal>>();
const input = ref<HTMLInputElement>();
const isCreating = ref(false);
const isRateLimited = ref(false);
const backupError = ref<string | null>(null);
const backupName = ref("");
const newBackupAmount = computed(() =>
  props.server.backups?.data?.length === undefined ? 1 : props.server.backups?.data?.length + 1,
);

const focusInput = () => {
  nextTick(() => {
    setTimeout(() => {
      input.value?.focus();
    }, 100);
  });
};

const hideModal = () => {
  modal.value?.hide();
  backupName.value = "";
};

const createBackup = async () => {
  if (!backupName.value.trim()) {
    backupName.value = `Backup #${newBackupAmount.value}`;
  }

  isCreating.value = true;
  isRateLimited.value = false;
  try {
    await props.server.backups?.create(backupName.value);
    await props.server.refresh();
    hideModal();
    emit("backupCreated", { success: true, message: "Backup created successfully" });
  } catch (error) {
    if (error instanceof PyroFetchError && error.statusCode === 429) {
      isRateLimited.value = true;
      backupError.value = "You're creating backups too fast.";
    } else {
      backupError.value = error instanceof Error ? error.message : String(error);
      emit("backupCreated", { success: false, message: backupError.value });
    }
  } finally {
    isCreating.value = false;
  }
};

defineExpose({
  show: () => modal.value?.show(),
  hide: hideModal,
});
</script>
