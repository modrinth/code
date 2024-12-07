<template>
  <li
    role="button"
    data-pyro-file
    :class="containerClasses"
    tabindex="0"
    @click="selectItem"
    @contextmenu="openContextMenu"
    @keydown="(e) => e.key === 'Enter' && selectItem()"
  >
    <div data-pyro-file-metadata class="flex w-full items-center gap-4 truncate">
      <div
        class="flex size-8 items-center justify-center rounded-full bg-bg-raised p-[6px] group-hover:bg-brand-highlight group-hover:text-brand group-focus:bg-brand-highlight group-focus:text-brand"
      >
        <component :is="iconComponent" class="size-6" />
      </div>
      <div class="flex w-full flex-col truncate">
        <span
          class="w-[98%] truncate font-bold group-hover:text-contrast group-focus:text-contrast"
          >{{ name }}</span
        >
        <span class="text-xs text-secondary group-hover:text-primary">
          {{ subText }}
        </span>
      </div>
    </div>

    <div data-pyro-file-actions class="flex w-fit flex-shrink-0 items-center gap-4 md:gap-12">
      <span class="w-[160px] text-nowrap text-right font-mono text-sm text-secondary">{{
        formattedDate
      }}</span>
      <ButtonStyled circular type="transparent">
        <UiServersTeleportOverflowMenu :options="menuOptions" direction="left" position="bottom">
          <MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
          <template #rename> <EditIcon /> Rename </template>
          <template #move> <RightArrowIcon /> Move </template>
          <template #download> <DownloadIcon /> Download </template>
          <template #delete> <TrashIcon /> Delete </template>
        </UiServersTeleportOverflowMenu>
      </ButtonStyled>
    </div>
  </li>
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
  RightArrowIcon,
} from "@modrinth/assets";
import { computed, shallowRef, ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  UiServersIconsCogFolderIcon,
  UiServersIconsEarthIcon,
  UiServersIconsCodeFileIcon,
  UiServersIconsTextFileIcon,
  UiServersIconsImageFileIcon,
} from "#components";
import PaletteIcon from "~/assets/icons/palette.svg?component";

interface FileItemProps {
  name: string;
  type: "directory" | "file";
  size?: number;
  count?: number;
  modified: number;
  path: string;
}

const props = defineProps<FileItemProps>();

const emit = defineEmits(["rename", "download", "delete", "move", "edit", "contextmenu"]);

const codeExtensions = Object.freeze([
  "json",
  "json5",
  "jsonc",
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
]);

const textExtensions = Object.freeze(["txt", "md", "log", "cfg", "conf", "properties", "ini"]);
const imageExtensions = Object.freeze(["png", "jpg", "jpeg", "gif", "svg", "webp"]);
const units = Object.freeze(["B", "KB", "MB", "GB", "TB", "PB", "EB"]);

const route = shallowRef(useRoute());
const router = useRouter();

const containerClasses = computed(() => [
  "group m-0 p-0 focus:!outline-none flex w-full select-none items-center justify-between overflow-hidden border-0 border-b border-solid border-bg-raised p-3 last:border-none hover:bg-bg-raised focus:bg-bg-raised",
  isEditableFile.value ? "cursor-pointer" : props.type === "directory" ? "cursor-pointer" : "",
]);

const fileExtension = computed(() => props.name.split(".").pop()?.toLowerCase() || "");

const menuOptions = computed(() => [
  {
    id: "rename",
    action: () => emit("rename", { name: props.name, type: props.type, path: props.path }),
  },
  {
    id: "move",
    action: () => emit("move", { name: props.name, type: props.type, path: props.path }),
  },
  {
    id: "download",
    action: () => emit("download", { name: props.name, type: props.type, path: props.path }),
    shown: props.type !== "directory",
  },
  {
    id: "delete",
    action: () => emit("delete", { name: props.name, type: props.type, path: props.path }),
    color: "red" as const,
  },
]);

const iconComponent = computed(() => {
  if (props.type === "directory") {
    if (props.name === "config") return UiServersIconsCogFolderIcon;
    if (props.name === "world") return UiServersIconsEarthIcon;
    if (props.name === "resourcepacks") return PaletteIcon;
    return FolderOpenIcon;
  }

  const ext = fileExtension.value;
  if (codeExtensions.includes(ext)) return UiServersIconsCodeFileIcon;
  if (textExtensions.includes(ext)) return UiServersIconsTextFileIcon;
  if (imageExtensions.includes(ext)) return UiServersIconsImageFileIcon;
  return FileIcon;
});

const subText = computed(() => {
  if (props.type === "directory") {
    return `${props.count} ${props.count === 1 ? "item" : "items"}`;
  }
  return formattedSize.value;
});

const formattedDate = computed(() => {
  const date = new Date(props.modified * 1000);
  return `${date.toLocaleDateString("en-US", {
    month: "2-digit",
    day: "2-digit",
    year: "2-digit",
  })}, ${date.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "numeric",
    hour12: true,
  })}`;
});

const isEditableFile = computed(() => {
  if (props.type === "file") {
    const ext = fileExtension.value;
    return (
      !props.name.includes(".") ||
      textExtensions.includes(ext) ||
      codeExtensions.includes(ext) ||
      imageExtensions.includes(ext)
    );
  }
  return false;
});

const formattedSize = computed(() => {
  if (props.size === undefined) return "";
  const bytes = props.size;
  if (bytes === 0) return "0 B";

  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const size = (bytes / Math.pow(1024, exponent)).toFixed(2);
  return `${size} ${units[exponent]}`;
});

const openContextMenu = (event: MouseEvent) => {
  event.preventDefault();
  emit("contextmenu", event.clientX, event.clientY);
};

const navigateToFolder = () => {
  const currentPath = route.value.query.path?.toString() || "";
  const newPath = currentPath.endsWith("/")
    ? `${currentPath}${props.name}`
    : `${currentPath}/${props.name}`;
  router.push({ query: { path: newPath, page: 1 } });
};

const isNavigating = ref(false);

const selectItem = () => {
  if (isNavigating.value) return;
  isNavigating.value = true;

  if (props.type === "directory") {
    navigateToFolder();
  } else if (props.type === "file" && isEditableFile.value) {
    emit("edit", { name: props.name, type: props.type, path: props.path });
  }

  setTimeout(() => {
    isNavigating.value = false;
  }, 500);
};
</script>
