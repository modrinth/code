<template>
  <div
    data-pyro-file
    :class="[
      'group flex w-full select-none items-center justify-between overflow-hidden border-0 border-b border-solid border-bg-raised p-3 last:border-none hover:bg-bg-raised',
      isEditableFile ? 'cursor-pointer' : type === 'directory' ? 'cursor-pointer' : '',
    ]"
    @click="selectItem"
    @contextmenu="openContextMenu"
  >
    <div data-pyro-file-metadata class="flex w-full items-center gap-2 truncate">
      <div
        class="flex size-8 items-center justify-center rounded-full bg-bg-raised p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
      >
        <template v-if="type === 'directory'">
          <UiServersIconsCogFolderIcon v-if="name === 'config'" class="size-6" />
          <UiServersIconsEarthIcon v-else-if="name === 'world'" class="size-6" />
          <PaletteIcon v-else-if="name === 'resourcepacks'" class="size-6" />
          <FolderOpenIcon v-else class="size-6" />
        </template>
        <template v-else>
          <UiServersIconsCodeFileIcon v-if="isCodeFile" class="size-6" />
          <UiServersIconsTextFileIcon v-else-if="isTextFile" class="size-6" />
          <UiServersIconsImageFileIcon v-else-if="isImageFile" class="size-6" />
          <FileIcon v-else class="size-6" />
        </template>
      </div>
      <div class="flex w-full flex-col truncate">
        <span class="w-[98%] truncate font-bold group-hover:text-contrast">{{ name }}</span>
        <span v-if="type === 'directory'" class="text-xs text-secondary group-hover:text-primary">
          {{ count }} {{ count === 1 ? "item" : "items" }}
        </span>
        <span v-else class="text-xs text-secondary group-hover:text-primary">
          {{ formattedSize }}
        </span>
      </div>
    </div>

    <div data-pyro-file-actions class="flex w-fit flex-shrink-0 items-center gap-4">
      <span class="w-full text-nowrap font-mono text-sm text-secondary">
        {{
          new Date(modified * 1000).toLocaleDateString("en-US", {
            month: "2-digit",
            day: "2-digit",
            year: "2-digit",
          })
        }},
        {{
          new Date(modified * 1000).toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "numeric",
            hour12: true,
          })
        }}
      </span>
      <ButtonStyled type="transparent">
        <UiServersTeleportOverflowMenu
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
          position="bottom"
        >
          <MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
          <template #rename> <EditIcon /> Rename </template>
          <template #move> <ArrowBigUpDashIcon /> Move </template>
          <template #download> <DownloadIcon /> Download </template>
          <template #delete> <TrashIcon /> Delete </template>
        </UiServersTeleportOverflowMenu>
      </ButtonStyled>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
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
import PaletteIcon from "~/assets/icons/palette.svg?component";

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

const fileExtension = computed(() => props.name.split(".").pop()?.toLowerCase() || "");

// i dont expect many people to actually see these being used
// but json and yaml for sure
const codeExtensions = [
  "json",
  "java",
  "kt",
  "kts",
  "sh",
  "bat",
  "ps1",
  "yml",
  "yaml",
  "toml",
  "js",
  "ts",
  "py",
  "rb",
  "php",
  "html",
  "css",
  "cpp",
  "c",
  "h",
  "rs",
  "go",
];

const textExtensions = ["txt", "md", "log", "cfg", "conf", "properties", "ini"];

const imageExtensions = ["png", "jpg", "jpeg", "gif", "svg", "webp"];

const isCodeFile = computed(
  () => props.type === "file" && codeExtensions.includes(fileExtension.value),
);

const isTextFile = computed(
  () => props.type === "file" && textExtensions.includes(fileExtension.value),
);

const isImageFile = computed(
  () => props.type === "file" && imageExtensions.includes(fileExtension.value),
);

const openContextMenu = (event: MouseEvent) => {
  event.preventDefault();
  emit("contextmenu", event.clientX, event.clientY);
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

const editableExtensions = [...textExtensions, ...codeExtensions, ...imageExtensions];

const isEditableFile = computed(() => {
  if (props.type === "file") {
    return !props.name.includes(".") || editableExtensions.includes(fileExtension.value);
  }
  return false;
});

const selectItem = () => {
  if (props.type === "directory") {
    navigateToFolder();
  } else if (props.type === "file" && isEditableFile.value) {
    emit("edit", { name: props.name, type: props.type, path: props.path });
  }
};
</script>
