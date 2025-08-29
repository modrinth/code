<template>
  <ConfirmModal
    ref="modal"
    title="Do you want to overwrite these conflicting files?"
    :proceed-label="`Overwrite`"
    :proceed-icon="CheckIcon"
    @proceed="proceed"
  >
    <div class="flex max-w-120 flex-col gap-4">
      <p class="m-0 leading-normal font-semibold">
        <template v-if="hasMany">
          Over 100 files will be overwritten if you proceed with extraction; here is just some of
          them:
        </template>
        <template v-else>
          The following {{ files.length }} files already exist on your server, and will be
          overwritten if you proceed with extraction:
        </template>
      </p>
      <ul class="bg-bg m-0 max-h-80 list-none overflow-auto rounded-2xl px-4 py-3">
        <li v-for="file in files" :key="file" class="flex items-center gap-1 py-1 font-medium">
          <XIcon class="text-red shrink-0" /> {{ file }}
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

const hasMany = computed(() => files.value.length > 100);

const show = (zipPath: string, conflictingFiles: string[]) => {
  path.value = zipPath;
  files.value = conflictingFiles;
  modal.value?.show();
};

const proceed = () => {
  emit("proceed", path.value);
};

defineExpose({ show });
</script>
