<template>
  <div
    class="fixed"
    :style="{
      transform: `translateY(${isAtBottom ? '-100%' : '0'})`,
      top: `${y}px`,
      left: `${x}px`,
    }"
  >
    <Transition>
      <div
        v-if="item"
        id="item-context-menu"
        ref="ctxRef"
        :style="{
          border: '1px solid var(--color-divider)',
          borderRadius: 'var(--radius-lg)',
          backgroundColor: 'var(--color-raised-bg)',
          padding: 'var(--gap-sm)',
          boxShadow: 'var(--shadow-floating)',
          gap: 'var(--gap-xs)',
          width: 'max-content',
        }"
        class="flex h-fit w-fit select-none flex-col"
      >
        <button
          class="btn btn-transparent flex !w-full items-center"
          @click="$emit('rename', item)"
        >
          <EditIcon class="h-5 w-5" />
          Rename
        </button>
        <button class="btn btn-transparent flex !w-full items-center" @click="$emit('move', item)">
          <RightArrowIcon />
          Move
        </button>
        <button
          v-if="item.type !== 'directory'"
          class="btn btn-transparent flex !w-full items-center"
          @click="$emit('download', item)"
        >
          <DownloadIcon class="h-5 w-5" />
          Download
        </button>
        <button
          class="btn btn-transparent btn-red flex !w-full items-center"
          @click="$emit('delete', item)"
        >
          <TrashIcon class="h-5 w-5" />
          Delete
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { EditIcon, DownloadIcon, TrashIcon, RightArrowIcon } from "@modrinth/assets";

interface FileItem {
  type: string;
  name: string;
  [key: string]: any;
}

defineProps<{
  item: FileItem | null;
  x: number;
  y: number;
  isAtBottom: boolean;
}>();

const ctxRef = ref<HTMLElement | null>(null);

defineEmits<{
  (e: "rename", item: FileItem): void;
  (e: "move", item: FileItem): void;
  (e: "download", item: FileItem): void;
  (e: "delete", item: FileItem): void;
}>();

defineExpose({
  ctxRef,
});
</script>

<style scoped>
#item-context-menu {
  transition:
    transform 0.1s ease,
    opacity 0.1s ease;
  transform-origin: top left;
}

#item-context-menu.v-enter-active,
#item-context-menu.v-leave-active {
  transform: scale(1);
  opacity: 1;
}

#item-context-menu.v-enter-from,
#item-context-menu.v-leave-to {
  transform: scale(0.5);
  opacity: 0;
}
</style>
