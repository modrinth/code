<template>
  <li
    role="button"
    data-pyro-file
    :class="[
      containerClasses,
      isDragOver && type === 'directory' ? 'bg-brand-highlight' : '',
      isDragging ? 'opacity-50' : '',
    ]"
    tabindex="0"
    draggable="true"
    @click="selectItem"
    @contextmenu="openContextMenu"
    @keydown="(e) => e.key === 'Enter' && selectItem()"
    @dragstart="handleDragStart"
    @dragend="handleDragEnd"
    @dragenter.prevent="handleDragEnter"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop"
  >
    <div
      data-pyro-file-metadata
      class="pointer-events-none flex w-full items-center gap-4 truncate"
    >
      <div
        class="pointer-events-none flex size-8 items-center justify-center rounded-full bg-bg-raised p-[6px] group-hover:bg-brand-highlight group-hover:text-brand group-focus:bg-brand-highlight group-focus:text-brand"
        :class="isEditableFile ? 'group-active:scale-[0.8]' : ''"
      >
        <component :is="iconComponent" class="size-6" />
      </div>
      <div class="pointer-events-none flex w-full flex-col truncate">
        <span
          class="pointer-events-none w-[98%] truncate font-bold group-hover:text-contrast group-focus:text-contrast"
        >
          {{ name }}
        </span>
        <span class="pointer-events-none text-xs text-secondary group-hover:text-primary">
          {{ subText }}
        </span>
      </div>
    </div>
    <div
      data-pyro-file-actions
      class="pointer-events-auto flex w-fit flex-shrink-0 items-center gap-4 md:gap-12"
    >
      <span class="hidden w-[160px] text-nowrap font-mono text-sm text-secondary md:flex">
        {{ formattedCreationDate }}
      </span>
      <span class="w-[160px] text-nowrap font-mono text-sm text-secondary">
        {{ formattedModifiedDate }}
      </span>
      <ButtonStyled circular type="transparent">
        <UiServersTeleportOverflowMenu :options="menuOptions" direction="left" position="bottom">
          <MoreHorizontalIcon class="h-5 w-5 bg-transparent" />
          <template #extract><PackageOpenIcon /> Extract</template>
          <template #rename><EditIcon /> Rename</template>
          <template #move><RightArrowIcon /> Move</template>
          <template #download><DownloadIcon /> Download</template>
          <template #delete><TrashIcon /> Delete</template>
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
  PackageOpenIcon,
  FileArchiveIcon,
} from "@modrinth/assets";
import { computed, shallowRef, ref } from "vue";
import { renderToString } from "vue/server-renderer";
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
  created: number;
  path: string;
}

const props = defineProps<FileItemProps>();

const emit = defineEmits<{
  (
    e: "rename" | "move" | "download" | "delete" | "edit" | "extract",
    item: { name: string; type: string; path: string },
  ): void;
  (
    e: "moveDirectTo",
    item: { name: string; type: string; path: string; destination: string },
  ): void;
  (e: "contextmenu", x: number, y: number): void;
}>();

const isDragOver = ref(false);
const isDragging = ref(false);

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
const supportedArchiveExtensions = Object.freeze(["zip"]);
const units = Object.freeze(["B", "KB", "MB", "GB", "TB", "PB", "EB"]);

const route = shallowRef(useRoute());
const router = useRouter();

const containerClasses = computed(() => [
  "group m-0 p-0 focus:!outline-none flex w-full select-none items-center justify-between overflow-hidden border-0 border-b border-solid border-bg-raised p-3 last:border-none hover:bg-bg-raised focus:bg-bg-raised",
  isEditableFile.value ? "cursor-pointer" : props.type === "directory" ? "cursor-pointer" : "",
  isDragOver.value ? "bg-brand-highlight" : "",
]);

const fileExtension = computed(() => props.name.split(".").pop()?.toLowerCase() || "");

const isZip = computed(() => fileExtension.value === "zip");

const menuOptions = computed(() => [
  {
    id: "extract",
    shown: isZip.value,
    action: () => emit("extract", { name: props.name, type: props.type, path: props.path }),
  },
  {
    divider: true,
    shown: isZip.value,
  },
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
  if (supportedArchiveExtensions.includes(ext)) return FileArchiveIcon;
  return FileIcon;
});

const subText = computed(() => {
  if (props.type === "directory") {
    return `${props.count} ${props.count === 1 ? "item" : "items"}`;
  }
  return formattedSize.value;
});

const formattedModifiedDate = computed(() => {
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

const formattedCreationDate = computed(() => {
  const date = new Date(props.created * 1000);
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

const getDragIcon = async () => {
  let iconToUse;

  if (props.type === "directory") {
    if (props.name === "config") {
      iconToUse = UiServersIconsCogFolderIcon;
    } else if (props.name === "world") {
      iconToUse = UiServersIconsEarthIcon;
    } else if (props.name === "resourcepacks") {
      iconToUse = PaletteIcon;
    } else {
      iconToUse = FolderOpenIcon;
    }
  } else {
    const ext = fileExtension.value;
    if (codeExtensions.includes(ext)) {
      iconToUse = UiServersIconsCodeFileIcon;
    } else if (textExtensions.includes(ext)) {
      iconToUse = UiServersIconsTextFileIcon;
    } else if (imageExtensions.includes(ext)) {
      iconToUse = UiServersIconsImageFileIcon;
    } else {
      iconToUse = FileIcon;
    }
  }

  return await renderToString(h(iconToUse));
};

const handleDragStart = async (event: DragEvent) => {
  if (!event.dataTransfer) return;
  isDragging.value = true;

  const dragGhost = document.createElement("div");
  dragGhost.className =
    "fixed left-0 top-0 flex items-center max-w-[500px] flex-row gap-3 rounded-lg bg-bg-raised p-3 shadow-lg pointer-events-none";

  const iconContainer = document.createElement("div");
  iconContainer.className = "flex size-6 items-center justify-center";

  const icon = document.createElement("div");
  icon.className = "size-4";
  icon.innerHTML = await getDragIcon();
  iconContainer.appendChild(icon);

  const nameSpan = document.createElement("span");
  nameSpan.className = "font-bold truncate text-contrast";
  nameSpan.textContent = props.name;

  dragGhost.appendChild(iconContainer);
  dragGhost.appendChild(nameSpan);
  document.body.appendChild(dragGhost);

  event.dataTransfer.setDragImage(dragGhost, 0, 0);

  requestAnimationFrame(() => {
    document.body.removeChild(dragGhost);
  });

  event.dataTransfer.setData(
    "application/pyro-file-move",
    JSON.stringify({
      name: props.name,
      type: props.type,
      path: props.path,
    }),
  );
  event.dataTransfer.effectAllowed = "move";
};

const isChildPath = (parentPath: string, childPath: string) => {
  return childPath.startsWith(parentPath + "/");
};

const handleDragEnd = () => {
  isDragging.value = false;
};

const handleDragEnter = () => {
  if (props.type !== "directory") return;
  isDragOver.value = true;
};

const handleDragOver = (event: DragEvent) => {
  if (props.type !== "directory" || !event.dataTransfer) return;
  event.dataTransfer.dropEffect = "move";
};

const handleDragLeave = () => {
  isDragOver.value = false;
};

const handleDrop = (event: DragEvent) => {
  isDragOver.value = false;
  if (props.type !== "directory" || !event.dataTransfer) return;

  try {
    const dragData = JSON.parse(event.dataTransfer.getData("application/pyro-file-move"));

    if (dragData.path === props.path) return;

    if (dragData.type === "directory" && isChildPath(dragData.path, props.path)) {
      console.error("Cannot move a folder into its own subfolder");
      return;
    }

    emit("moveDirectTo", {
      name: dragData.name,
      type: dragData.type,
      path: dragData.path,
      destination: props.path,
    });
  } catch (error) {
    console.error("Error handling file drop:", error);
  }
};
</script>
