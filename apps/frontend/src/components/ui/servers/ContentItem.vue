<template>
  <div
    class="group flex w-full items-center justify-between border-0 border-b border-solid border-bg-raised p-[0.7rem] hover:bg-bg-raised"
  >
    <div class="flex items-center gap-2">
      <component
        :is="icon"
        class="h-8 w-8 rounded-full bg-bg-raised p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
      />
      <div class="flex flex-col">
        <span class="font-bold group-hover:text-contrast">{{ name }}</span>
        <span class="text-xs text-secondary group-hover:text-primary">{{ size }} MB</span>
      </div>
    </div>
    <OverflowMenu
      :options="[
        {
          id: 'rename',
          action: () => {},
        },
        {
          id: 'restore',
          action: () => {},
        },
        { id: 'download', action: () => {} },
        {
          id: 'delete',
          action: () => {},
          color: 'red',
        },
      ]"
      direction="right"
      class="bg-transparent"
    >
      <MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
      <template #rename> <EditIcon /> Rename </template>
      <template #restore> <ClipboardCopyIcon /> Restore </template>
      <template #download> <DownloadIcon /> Download </template>
      <template #delete> <TrashIcon /> Delete </template>
    </OverflowMenu>
  </div>
</template>

<script lang="ts">
import { OverflowMenu } from "@modrinth/ui";
import {
  MoreHorizontalIcon,
  EditIcon,
  ClipboardCopyIcon,
  DownloadIcon,
  TrashIcon,
} from "@modrinth/assets";

export default defineComponent({
  name: "ContentItem",
  props: {
    name: {
      type: String as PropType<string>,
      required: true,
    },
    icon: {
      type: Object as PropType<Record<string, unknown>>,
      required: true,
    },
    version: {
      type: String as PropType<string>,
      required: true,
    },
    size: {
      type: Number as PropType<number>,
      required: true,
    },
  },
});
</script>
