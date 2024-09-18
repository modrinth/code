<template>
  <div
    class="group flex w-full items-center justify-between border-0 border-b border-solid border-bg-raised p-[0.7rem] hover:bg-bg-raised"
    @click="navigateToFolder"
  >
    <div class="flex items-center gap-2">
      <div
        class="flex h-8 w-8 items-center justify-center rounded-full bg-bg-raised p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
      >
        <FolderOpenIcon v-if="type === 'directory'" class="h-6 w-6" />
        <FileIcon v-else-if="type === 'file'" class="h-6 w-6" />
      </div>
      <div class="flex flex-col">
        <span class="font-bold group-hover:text-contrast">{{ name }}</span>
        <span v-if="type === 'directory'" class="text-xs text-secondary group-hover:text-primary"
          >{{ count }} items</span
        >
        <span v-else class="text-xs text-secondary group-hover:text-primary"
          >{{ sizeInMB }} MB</span
        >
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

<script setup lang="ts">
import { OverflowMenu } from "@modrinth/ui";
import {
  MoreHorizontalIcon,
  EditIcon,
  ClipboardCopyIcon,
  DownloadIcon,
  TrashIcon,
  FolderOpenIcon,
  FileIcon,
} from "@modrinth/assets";

const router = useRouter();
const route = useRoute();

const props = defineProps({
  name: {
    type: String as PropType<string>,
    required: true,
  },
  type: {
    type: String as PropType<"directory" | "file">,
    required: true,
  },
  size: {
    type: Number as PropType<number>,
    required: false,
    default: 0,
  },
  count: {
    type: Number as PropType<number>,
    required: false,
    default: 0,
  },
  created: {
    type: Number as PropType<number>,
    required: false,
    default: 0,
  },
  modified: {
    type: Number as PropType<number>,
    required: false,
    default: 0,
  },
});

const sizeInMB = computed(() => parseFloat((props.size / 1024 / 1024).toFixed(2)));

const currentPath = ref(route.query.path || "/");

const navigateToFolder = () => {
  const newPath = `${route.query.path ?? ""}/${props.name}`;
  // if file append /edit to end of path
  router.push({ query: { path: newPath + (props.type === "file" ? "&edit=true" : "") } });
};
</script>
