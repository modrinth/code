<template>
  <div
    class="group flex w-full cursor-pointer items-center justify-between border-0 border-b border-solid border-bg-raised p-[0.7rem] last:rounded-b-xl last:border-none hover:bg-bg-raised"
  >
    <div class="flex w-full items-center gap-2" @click="navigateToFolder">
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
          action: () => $emit('rename', { name, type, path }),
        },
        {
          id: 'move',
          action: () => $emit('move', { name, type, path }),
        },
        {
          id: 'download',
          action: () => $emit('download', { name, type, path }),
          shown: type !== 'directory',
        },
        {
          id: 'delete',
          action: () => $emit('delete', { name, type, path }),
          color: 'red',
        },
      ]"
      direction="right"
      class="bg-transparent"
    >
      <MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
      <template #rename> <EditIcon /> Rename </template>
      <template #move> <ArrowBigUpDashIcon /> Move </template>
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
  DownloadIcon,
  TrashIcon,
  FolderOpenIcon,
  FileIcon,
  ArrowBigUpDashIcon,
} from "@modrinth/assets";

const router = useRouter();
const route = useRoute();

const props = defineProps({
  name: {
    type: String,
    required: true,
  },
  type: {
    type: String as PropType<"directory" | "file">,
    required: true,
  },
  size: {
    type: Number,
    required: false,
    default: 0,
  },
  count: {
    type: Number,
    required: false,
    default: 0,
  },
  created: {
    type: Number,
    required: false,
    default: 0,
  },
  modified: {
    type: Number,
    required: false,
    default: 0,
  },
  path: {
    type: String,
    required: true,
  },
});

const emit = defineEmits(["rename", "download", "delete", "move"]);

const sizeInMB = computed(() => parseFloat((props.size / 1024 / 1024).toFixed(2)));

const navigateToFolder = () => {
  if (props.type === "directory") {
    const newPath = `${route.query.path ?? ""}/${props.name}`;
    router.push({ query: { path: newPath } });
  }
};
</script>
