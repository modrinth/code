<template>
  <div
    @dragenter.prevent="handleDragEnter"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop"
  >
    <slot />
    <div
      v-if="isDragging"
      :class="[
        'absolute inset-0 flex items-center justify-center rounded-2xl bg-black bg-opacity-50 text-white',
        overlayClass,
      ]"
    >
      <div class="text-center">
        <UploadIcon class="mx-auto h-16 w-16" />
        <p class="mt-2 text-xl">
          Drop {{ type ? type.toLocaleLowerCase() : "file" }}s here to upload
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { UploadIcon } from "@modrinth/assets";
import { ref } from "vue";

const emit = defineEmits<{
  (event: "filesDropped", files: File[]): void;
}>();

defineProps<{
  overlayClass?: string;
  type?: string;
}>();

const isDragging = ref(false);
const dragCounter = ref(0);

const handleDragEnter = (event: DragEvent) => {
  event.preventDefault();
  if (!event.dataTransfer?.types.includes("application/pyro-file-move")) {
    dragCounter.value++;
    isDragging.value = true;
  }
};

const handleDragOver = (event: DragEvent) => {
  event.preventDefault();
};

const handleDragLeave = (event: DragEvent) => {
  event.preventDefault();
  dragCounter.value--;
  if (dragCounter.value === 0) {
    isDragging.value = false;
  }
};

const handleDrop = (event: DragEvent) => {
  event.preventDefault();
  isDragging.value = false;
  dragCounter.value = 0;

  const isInternalMove = event.dataTransfer?.types.includes("application/pyro-file-move");
  if (isInternalMove) return;

  const files = event.dataTransfer?.files;
  if (files) {
    emit("filesDropped", Array.from(files));
  }
};
</script>
