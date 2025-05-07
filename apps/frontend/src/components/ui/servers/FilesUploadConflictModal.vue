<template>
  <ConfirmModal
    ref="modal"
    title="Do you want to overwrite these conflicting files?"
    :proceed-label="`Overwrite`"
    :proceed-icon="CheckIcon"
    @proceed="proceed"
  >
    <div class="flex max-w-[30rem] flex-col gap-4">
      <p class="m-0 font-bold">
        The following files already exist on your server, and will be overwritten if you proceed
        with extraction:
      </p>
      <ul class="m-0 grid list-none grid-cols-1 rounded-2xl bg-bg px-4 py-3 sm:grid-cols-2">
        <li v-for="file in files" :key="file" class="flex items-center gap-1 py-1 font-medium">
          <XIcon class="text-red" /> {{ file }}
        </li>
      </ul>
    </div>
  </ConfirmModal>
</template>

<script setup lang="ts">
import { ConfirmModal } from "@modrinth/ui";
import { ref } from "vue";
import { XIcon, CheckIcon } from "@modrinth/assets";

const path = ref("");
const files = ref<string[]>([]);

const emit = defineEmits<{
  (e: "proceed", path: string): void;
}>();

const modal = ref<typeof ConfirmModal>();

const show = (zipPath: string, conflictingFiles: string[]) => {
  path.value = zipPath;
  files.value = conflictingFiles;
  modal.value?.show();
};

const proceed = () => {
  emit("proceed", path.value);
  hide();
};

const hide = () => {
  modal.value?.hide();
};

defineExpose({ show, hide });
</script>
