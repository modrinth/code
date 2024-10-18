<template>
  <div
    :class="[
      'group flex w-full items-center justify-between border-0 border-b border-solid border-bg-raised p-[0.7rem] last:border-none hover:bg-bg-raised',
      isNonEditableFile ? '' : 'cursor-pointer',
    ]"
    @contextmenu="openContextMenu"
  >
    <div class="flex w-full items-center gap-2" @click="selectItem">
      <div
        class="flex h-8 w-8 items-center justify-center rounded-full bg-bg-raised p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
      >
        <FolderOpenIcon v-if="type === 'directory'" class="h-6 w-6" />
        <FileIcon v-else-if="type === 'file'" class="h-6 w-6" />
      </div>
      <div class="flex flex-col">
        <span class="font-bold group-hover:text-contrast">{{ name }}</span>
        <span v-if="type === 'directory'" class="text-xs text-secondary group-hover:text-primary">
          {{ count }} items
        </span>
        <span v-else class="text-xs text-secondary group-hover:text-primary">
          {{ formattedSize }}
        </span>
      </div>
    </div>
    <div class="flex w-fit items-center gap-4">
      <span class="w-full text-nowrap text-sm text-secondary">
        {{ new Date(modified * 1000).toLocaleString() }}
      </span>
      <ButtonStyled type="transparent">
        <OverflowMenu
          class="btn-dropdown-animation flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
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
          direction="left"
        >
          <MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
          <template #rename> <EditIcon /> Rename </template>
          <template #move> <ArrowBigUpDashIcon /> Move </template>
          <template #download> <DownloadIcon /> Download </template>
          <template #delete> <TrashIcon /> Delete </template>
        </OverflowMenu>
      </ButtonStyled>
    </div>
  </div>
</template>

<script setup lang="ts">
import { OverflowMenu, ButtonStyled } from "@modrinth/ui";
import {
  MoreHorizontalIcon,
  EditIcon,
  DownloadIcon,
  TrashIcon,
  FolderOpenIcon,
  FileIcon,
  ArrowBigUpDashIcon,
} from "@modrinth/assets";
import { computed } from "vue";
import { useRouter, useRoute } from "vue-router";

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

const emit = defineEmits(["rename", "download", "delete", "move", "edit", "contextmenu"]);

const openContextMenu = (event: MouseEvent) => {
  event.preventDefault();
  // emit("contextmenu", event.clientX, event.clientY);
  // accounting for scroll top:
  emit("contextmenu", event.clientX, event.clientY + window.scrollY);
};

const formattedSize = computed(() => {
  const bytes = props.size;
  if (bytes === 0) return "0 B";

  const units = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
  const exponent = Math.floor(Math.log(bytes) / Math.log(1024));
  const clampedExponent = Math.min(exponent, units.length - 1);
  const size = bytes / Math.pow(1024, clampedExponent);

  return `${size.toFixed(2)} ${units[clampedExponent]}`;
});

const navigateToFolder = () => {
  if (props.type === "directory") {
    const currentPath = route.query.path?.toString() || "";
    const newPath = currentPath.endsWith("/")
      ? `${currentPath}${props.name}`
      : `${currentPath}/${props.name}`;
    router.push({ query: { path: newPath, page: 1 } });
  }
};

const nonEditableExtensions = [
  "jpg",
  "jpeg",
  "png",
  "gif",
  "bmp",
  "tiff",
  "jar",
  "exe",
  "zip",
  "rar",
  "7z",
  "tar",
  "gz",
  "bz2",
  "xz",
  "pdf",
  "doc",
  "docx",
  "xls",
  "xlsx",
  "ppt",
  "pptx",
  "dat",
  "lock",
  "mca",
];

const isNonEditableFile = computed(() => {
  if (props.type === "file") {
    const fileExtension = props.name.split(".").pop()?.toLowerCase();
    return nonEditableExtensions.includes(fileExtension || "");
  }
  return false;
});

const selectItem = () => {
  if (props.type === "directory") {
    navigateToFolder();
  } else if (props.type === "file" && !isNonEditableFile.value) {
    emit("edit", { name: props.name, type: props.type, path: props.path });
  }
};
</script>
