<template>
  <div data-pyro-file-manager-root class="contents">
    <LazyUiServersFilesCreateItemModal
      ref="createItemModal"
      :type="newItemType"
      @create="handleCreateNewItem"
    />

    <LazyUiServersFilesRenameItemModal
      ref="renameItemModal"
      :item="selectedItem"
      @rename="handleRenameItem"
    />

    <LazyUiServersFilesMoveItemModal
      ref="moveItemModal"
      :item="selectedItem"
      :current-path="currentPath"
      @move="handleMoveItem"
    />

    <LazyUiServersFilesDeleteItemModal
      ref="deleteItemModal"
      :item="selectedItem"
      @delete="handleDeleteItem"
    />

    <div
      class="relative flex w-full flex-col rounded-2xl border border-solid border-bg-raised"
      @dragenter.prevent="handleDragEnter"
      @dragover.prevent="handleDragOver"
      @dragleave.prevent="handleDragLeave"
      @drop.prevent="handleDrop"
    >
      <div ref="mainContent" class="relative isolate flex w-full flex-col">
        <div v-if="!isEditing" class="contents">
          <UiServersFilesBrowseNavbar
            :breadcrumb-segments="breadcrumbSegments"
            :search-query="searchQuery"
            :sort-method="sortMethod"
            @navigate="navigateToSegment"
            @sort="sortFiles"
            @create="showCreateModal"
            @upload="initiateFileUpload"
            @update:search-query="searchQuery = $event"
          />
          <Transition
            name="upload-status"
            @enter="onUploadStatusEnter"
            @leave="onUploadStatusLeave"
          >
            <div
              v-if="isUploading"
              ref="uploadStatusRef"
              class="upload-status rounded-b-xl border-0 border-t border-solid border-bg bg-table-alternateRow text-contrast"
            >
              <div class="flex flex-col p-4 text-sm">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2 font-bold">
                    <FolderOpenIcon class="size-4" />
                    <span>
                      File Uploads{{
                        activeUploads.length > 0 ? ` - ${activeUploads.length} left` : ""
                      }}
                    </span>
                  </div>
                </div>

                <div class="mt-2 space-y-2">
                  <div
                    v-for="item in uploadQueue"
                    :key="item.file.name"
                    class="flex h-6 items-center justify-between gap-2 text-xs"
                  >
                    <div class="flex flex-1 items-center gap-2 truncate">
                      <transition-group name="status-icon" mode="out-in">
                        <UiServersPanelSpinner
                          v-show="item.status === 'uploading'"
                          key="spinner"
                          class="absolute !size-4"
                        />
                        <CheckCircleIcon
                          v-show="item.status === 'completed'"
                          key="check"
                          class="absolute size-4 text-green"
                        />
                        <XCircleIcon
                          v-show="item.status === 'error' || item.status === 'cancelled'"
                          key="error"
                          class="absolute size-4 text-red"
                        />
                      </transition-group>
                      <span class="ml-6 truncate">{{ item.file.name }}</span>
                      <span class="text-secondary">{{ item.size }}</span>
                    </div>
                    <div class="flex min-w-[80px] items-center justify-end gap-2">
                      <template v-if="item.status === 'completed'">
                        <span>Done</span>
                      </template>
                      <template v-else-if="item.status === 'error'">
                        <span class="text-red">Failed - File already exists</span>
                      </template>
                      <template v-else>
                        <template v-if="item.status === 'uploading'">
                          <span>{{ item.progress }}%</span>
                          <div class="h-1 w-20 overflow-hidden rounded-full bg-bg">
                            <div
                              class="h-full bg-contrast transition-all duration-200"
                              :style="{ width: item.progress + '%' }"
                            />
                          </div>
                          <ButtonStyled color="red" type="transparent" @click="cancelUpload(item)">
                            <button>Cancel</button>
                          </ButtonStyled>
                        </template>
                        <template v-else-if="item.status === 'cancelled'">
                          <span class="text-red">Cancelled</span>
                        </template>
                        <template v-else>
                          <span>{{ item.progress }}%</span>
                          <div class="h-1 w-20 overflow-hidden rounded-full bg-bg">
                            <div
                              class="h-full bg-contrast transition-all duration-200"
                              :style="{ width: item.progress + '%' }"
                            />
                          </div>
                        </template>
                      </template>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </Transition>
        </div>

        <UiServersFilesEditingNavbar
          v-else
          :file-name="editingFile?.name"
          :is-image="isEditingImage"
          :file-path="editingFile?.path"
          :breadcrumb-segments="breadcrumbSegments"
          @cancel="cancelEditing"
          @save="() => saveFileContent(true)"
          @save-as="saveFileContentAs"
          @save-restart="saveFileContentRestart"
          @share="requestShareLink"
          @navigate="navigateToSegment"
        />

        <div v-if="isEditing" class="h-full w-full flex-grow">
          <component
            :is="VAceEditor"
            v-if="!isEditingImage"
            v-model:value="fileContent"
            :lang="
              (() => {
                const ext = editingFile?.name?.split('.')?.pop()?.toLowerCase() ?? '';
                return ext === 'json'
                  ? 'json'
                  : ext === 'toml'
                    ? 'toml'
                    : ext === 'sh'
                      ? 'sh'
                      : ['yml', 'yaml'].includes(ext)
                        ? 'yaml'
                        : 'text';
              })()
            "
            theme="one_dark"
            :print-margin="false"
            style="height: 750px; font-size: 1rem"
            class="ace_editor ace_hidpi ace-one-dark ace_dark rounded-b-lg"
            @init="onInit"
          />
          <UiServersFilesImageViewer v-else :image-blob="imagePreview" />
        </div>

        <div v-else-if="items.length > 0" class="h-full w-full overflow-hidden rounded-b-2xl">
          <UiServersFilesLabelBar />
          <UiServersFileVirtualList
            :items="filteredItems"
            @delete="showDeleteModal"
            @rename="showRenameModal"
            @download="downloadFile"
            @move="showMoveModal"
            @move-direct-to="handleDirectMove"
            @edit="editFile"
            @contextmenu="showContextMenu"
            @load-more="handleLoadMore"
          />
        </div>

        <div
          v-else-if="!isLoading && items.length === 0 && !loadError"
          class="flex h-full w-full items-center justify-center p-20"
        >
          <div class="flex flex-col items-center gap-4 text-center">
            <FolderOpenIcon class="h-16 w-16 text-secondary" />
            <h3 class="m-0 text-2xl font-bold text-contrast">This folder is empty</h3>
            <p class="m-0 text-sm text-secondary">There are no files or folders.</p>
          </div>
        </div>

        <LazyUiServersFileManagerError
          v-else-if="loadError"
          title="Unable to load files"
          message="The folder may not exist."
          @refetch="refreshList"
          @home="navigateToSegment(-1)"
        />
      </div>

      <div
        v-if="isDragging"
        class="absolute inset-0 flex items-center justify-center rounded-2xl bg-black bg-opacity-50 text-white"
      >
        <div class="text-center">
          <UploadIcon class="mx-auto h-16 w-16" />
          <p class="mt-2 text-xl">Drop files here to upload</p>
        </div>
      </div>
    </div>

    <UiServersFilesContextMenu
      ref="contextMenu"
      :item="contextMenuInfo.item"
      :x="contextMenuInfo.x"
      :y="contextMenuInfo.y"
      :is-at-bottom="isAtBottom"
      @rename="showRenameModal"
      @move="showMoveModal"
      @download="downloadFile"
      @delete="showDeleteModal"
    />
  </div>
</template>

<script setup lang="ts">
import { useInfiniteScroll } from "@vueuse/core";
import { UploadIcon, FolderOpenIcon, CheckCircleIcon, XCircleIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import type { DirectoryResponse, DirectoryItem, Server } from "~/composables/pyroServers";

interface BaseOperation {
  type: "move" | "rename";
  itemType: string;
  fileName: string;
}

interface MoveOperation extends BaseOperation {
  type: "move";
  sourcePath: string;
  destinationPath: string;
}

interface RenameOperation extends BaseOperation {
  type: "rename";
  path: string;
  oldName: string;
  newName: string;
}

type Operation = MoveOperation | RenameOperation;

interface UploadItem {
  file: File;
  progress: number;
  status: "pending" | "uploading" | "completed" | "error" | "cancelled";
  size: string;
  uploader?: any;
}

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const route = useRoute();
const router = useRouter();

const VAceEditor = ref();
const mainContent = ref<HTMLElement | null>(null);
const scrollContainer = ref<HTMLElement | null>(null);
const contextMenu = ref();
const operationHistory = ref<Operation[]>([]);
const redoStack = ref<Operation[]>([]);

const searchQuery = ref("");
const sortMethod = ref("default");

const maxResults = 100;
const currentPage = ref(1);

const currentPath = ref(typeof route.query.path === "string" ? route.query.path : "");

const isAtBottom = ref(false);
const contextMenuInfo = ref<any>({ item: null, x: 0, y: 0 });

const createItemModal = ref();
const renameItemModal = ref();
const moveItemModal = ref();
const deleteItemModal = ref();

const newItemType = ref<"file" | "directory">("file");
const selectedItem = ref<any>(null);
const fileContent = ref("");

const isEditing = ref(false);
const editingFile = ref<any>(null);
const closeEditor = ref(false);
const isEditingImage = ref(false);
const imagePreview = ref();

const isDragging = ref(false);
const dragCounter = ref(0);

const uploadStatusRef = ref<HTMLElement | null>(null);
const isUploading = computed(() => uploadQueue.value.length > 0);
const uploadQueue = ref<UploadItem[]>([]);

const activeUploads = computed(() =>
  uploadQueue.value.filter((item) => item.status === "pending" || item.status === "uploading"),
);

const onUploadStatusEnter = (el: Element) => {
  const height = (el as HTMLElement).scrollHeight;
  (el as HTMLElement).style.height = "0";
  // eslint-disable-next-line no-void
  void (el as HTMLElement).offsetHeight;
  (el as HTMLElement).style.height = `${height}px`;
};

const onUploadStatusLeave = (el: Element) => {
  const height = (el as HTMLElement).scrollHeight;
  (el as HTMLElement).style.height = `${height}px`;
  // eslint-disable-next-line no-void
  void (el as HTMLElement).offsetHeight;
  (el as HTMLElement).style.height = "0";
};

watch(
  uploadQueue,
  () => {
    if (!uploadStatusRef.value) return;
    const el = uploadStatusRef.value;
    const itemsHeight = uploadQueue.value.length * 32;
    const headerHeight = 12;
    const gap = 8;
    const padding = 32;
    const totalHeight = padding + headerHeight + gap + itemsHeight;
    el.style.height = `${totalHeight}px`;
  },
  { deep: true },
);

const data = computed(() => props.server.general);

useHead({
  title: computed(() => `Files - ${data.value?.name ?? "Server"} - Modrinth`),
});

const fetchDirectoryContents = async (): Promise<DirectoryResponse> => {
  const path = Array.isArray(currentPath.value) ? currentPath.value.join("") : currentPath.value;
  try {
    const data = await props.server.fs?.listDirContents(path, currentPage.value, maxResults);

    if (!data || !data.items) {
      throw new Error("Invalid data structure received from server.");
    }

    if (currentPage.value === 1) {
      return {
        items: applyDefaultSort(data.items),
        total: data.total,
      };
    }

    return {
      items: [...(directoryData.value?.items || []), ...applyDefaultSort(data.items)],
      total: data.total,
    };
  } catch (error) {
    console.error("Error fetching directory contents:", error);
    if (error instanceof PyroFetchError && error.statusCode === 400) {
      return directoryData.value || { items: [], total: 0 };
    }
    throw error;
  }
};

const {
  data: directoryData,
  refresh: refreshData,
  status,
  error: loadError,
} = useLazyAsyncData(() => fetchDirectoryContents(), {
  watch: [],
  default: () => ({ items: [], total: 0 }),
  immediate: true,
});

const isLoading = computed(() => status.value === "pending");

const items = computed(() => directoryData.value?.items || []);

const refreshList = () => {
  currentPage.value = 1;
  refreshData();
  reset();
};

const undoLastOperation = async () => {
  const lastOperation = operationHistory.value.pop();
  if (!lastOperation) return;

  try {
    switch (lastOperation.type) {
      case "move":
        await props.server.fs?.moveFileOrFolder(
          `${lastOperation.destinationPath}/${lastOperation.fileName}`.replace("//", "/"),
          `${lastOperation.sourcePath}/${lastOperation.fileName}`.replace("//", "/"),
        );
        break;
      case "rename":
        await props.server.fs?.renameFileOrFolder(
          `${lastOperation.path}/${lastOperation.newName}`.replace("//", "/"),
          lastOperation.oldName,
        );
        break;
    }

    redoStack.value.push(lastOperation);

    refreshList();
    addNotification({
      group: "files",
      title: `${lastOperation.type === "move" ? "Move" : "Rename"} undone`,
      text: `${lastOperation.fileName} has been restored to its original ${lastOperation.type === "move" ? "location" : "name"}`,
      type: "success",
    });
  } catch (error) {
    console.error(`Error undoing ${lastOperation.type}:`, error);
    addNotification({
      group: "files",
      title: "Undo failed",
      text: `Failed to undo the last ${lastOperation.type} operation`,
      type: "error",
    });
  }
};

const redoLastOperation = async () => {
  const lastOperation = redoStack.value.pop();
  if (!lastOperation) return;

  try {
    switch (lastOperation.type) {
      case "move":
        await props.server.fs?.moveFileOrFolder(
          `${lastOperation.sourcePath}/${lastOperation.fileName}`.replace("//", "/"),
          `${lastOperation.destinationPath}/${lastOperation.fileName}`.replace("//", "/"),
        );
        break;
      case "rename":
        await props.server.fs?.renameFileOrFolder(
          `${lastOperation.path}/${lastOperation.oldName}`.replace("//", "/"),
          lastOperation.newName,
        );
        break;
    }

    operationHistory.value.push(lastOperation);

    refreshList();
    addNotification({
      group: "files",
      title: `${lastOperation.type === "move" ? "Move" : "Rename"} redone`,
      text: `${lastOperation.fileName} has been ${lastOperation.type === "move" ? "moved" : "renamed"} again`,
      type: "success",
    });
  } catch (error) {
    console.error(`Error redoing ${lastOperation.type}:`, error);
    addNotification({
      group: "files",
      title: "Redo failed",
      text: `Failed to redo the last ${lastOperation.type} operation`,
      type: "error",
    });
  }
};

const handleCreateNewItem = async (name: string) => {
  try {
    const path = `${currentPath.value}/${name}`.replace("//", "/");
    await props.server.fs?.createFileOrFolder(path, newItemType.value);

    refreshList();

    addNotification({
      group: "files",
      title: `${newItemType.value === "directory" ? "Folder" : "File"} created`,
      text: `New ${newItemType.value === "directory" ? "folder" : "file"} ${name} has been created.`,
      type: "success",
    });
  } catch (error) {
    handleCreateError(error);
  }
};

const handleRenameItem = async (newName: string) => {
  try {
    const path = `${currentPath.value}/${selectedItem.value.name}`.replace("//", "/");
    await props.server.fs?.renameFileOrFolder(path, newName);

    redoStack.value = [];
    operationHistory.value.push({
      type: "rename",
      itemType: selectedItem.value.type,
      fileName: selectedItem.value.name,
      path: currentPath.value,
      oldName: selectedItem.value.name,
      newName,
    });

    refreshList();

    if (closeEditor.value) {
      await props.server.refresh();
      isEditing.value = false;
      editingFile.value = null;
      closeEditor.value = false;
      router.push({ query: { ...route.query, path: currentPath.value } });
    }

    addNotification({
      group: "files",
      title: `${selectedItem.value.type === "directory" ? "Folder" : "File"} renamed`,
      text: `${selectedItem.value.name} has been renamed to ${newName}`,
      type: "success",
    });
  } catch (error) {
    console.error("Error renaming item:", error);
    if (error instanceof PyroFetchError) {
      if (error.statusCode === 400) {
        addNotification({
          group: "files",
          title: "Could not rename",
          text: `An item named "${newName}" already exists in this location`,
          type: "error",
        });
        return;
      }
      addNotification({
        group: "files",
        title: "Could not rename item",
        text: "An unexpected error occurred",
        type: "error",
      });
    }
  }
};

const handleMoveItem = async (destination: string) => {
  try {
    const itemType = selectedItem.value.type;
    const sourcePath = currentPath.value;
    const newPath = `${destination}/${selectedItem.value.name}`.replace("//", "/");

    await props.server.fs?.moveFileOrFolder(
      `${sourcePath}/${selectedItem.value.name}`.replace("//", "/"),
      newPath,
    );

    redoStack.value = [];
    operationHistory.value.push({
      type: "move",
      sourcePath,
      destinationPath: destination,
      fileName: selectedItem.value.name,
      itemType,
    });

    refreshList();
    addNotification({
      group: "files",
      title: `${itemType === "directory" ? "Folder" : "File"} moved`,
      text: `${selectedItem.value.name} has been moved to ${newPath}`,
      type: "success",
    });
  } catch (error) {
    console.error("Error moving item:", error);
  }
};

const handleDirectMove = async (moveData: {
  name: string;
  type: string;
  path: string;
  destination: string;
}) => {
  try {
    const newPath = `${moveData.destination}/${moveData.name}`.replace("//", "/");
    const sourcePath = moveData.path.substring(0, moveData.path.lastIndexOf("/"));

    await props.server.fs?.moveFileOrFolder(moveData.path, newPath);

    redoStack.value = [];
    operationHistory.value.push({
      type: "move",
      sourcePath,
      destinationPath: moveData.destination,
      fileName: moveData.name,
      itemType: moveData.type,
    });

    refreshList();
    addNotification({
      group: "files",
      title: `${moveData.type === "directory" ? "Folder" : "File"} moved`,
      text: `${moveData.name} has been moved to ${newPath}`,
      type: "success",
    });
  } catch (error) {
    console.error("Error moving item:", error);
  }
};

const handleDeleteItem = async () => {
  try {
    const path = `${currentPath.value}/${selectedItem.value.name}`.replace("//", "/");
    await props.server.fs?.deleteFileOrFolder(path, selectedItem.value.type === "directory");

    refreshList();
    addNotification({
      group: "files",
      title: "File deleted",
      text: "Your file has been deleted.",
      type: "success",
    });
  } catch (error) {
    console.error("Error deleting item:", error);
  }
};

const showCreateModal = (type: "file" | "directory") => {
  newItemType.value = type;
  createItemModal.value?.show();
};

const showRenameModal = (item: any) => {
  selectedItem.value = item;
  renameItemModal.value?.show(item);
  contextMenuInfo.value.item = null;
};

const showMoveModal = (item: any) => {
  selectedItem.value = item;
  moveItemModal.value?.show();
  contextMenuInfo.value.item = null;
};

const showDeleteModal = (item: any) => {
  selectedItem.value = item;
  deleteItemModal.value?.show();
  contextMenuInfo.value.item = null;
};

const handleCreateError = (error: any) => {
  console.error("Error creating item:", error);
  if (error instanceof PyroFetchError) {
    if (error.statusCode === 400) {
      addNotification({
        group: "files",
        title: "Error creating item",
        text: "Invalid file",
        type: "error",
      });
    } else if (error.statusCode === 500) {
      addNotification({
        group: "files",
        title: "Error creating item",
        text: "Something went wrong. The file may already exist.",
        type: "error",
      });
    }
  }
};

const applyDefaultSort = (items: DirectoryItem[]) => {
  return items.sort((a: any, b: any) => {
    if (a.type === "directory" && b.type !== "directory") return -1;
    if (a.type !== "directory" && b.type === "directory") return 1;

    return a.name.localeCompare(b.name);
  });
};

const filteredItems = computed(() => {
  let result = [...items.value];

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter((item) => item.name.toLowerCase().includes(query));
  }

  switch (sortMethod.value) {
    case "modified":
      result.sort((a, b) => new Date(b.modified).getTime() - new Date(a.modified).getTime());
      break;
    case "created":
      result.sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime());
      break;
    case "filesOnly":
      result = result.filter((item) => item.type !== "directory");
      break;
    case "foldersOnly":
      result = result.filter((item) => item.type === "directory");
      break;
    default:
      result = applyDefaultSort(result);
  }

  return result;
});

const { reset } = useInfiniteScroll(
  scrollContainer,
  async () => {
    if (status.value === "pending") return;

    try {
      const totalPages = directoryData.value?.total || 0;

      if (currentPage.value < totalPages) {
        currentPage.value++;
        const newData = await fetchDirectoryContents();

        if (newData && newData.items) {
          directoryData.value = {
            items: [...directoryData.value.items, ...newData.items],
            total: newData.total,
          };
        }
      }
    } catch (error) {
      console.error("Error during infinite scroll:", error);
    }
  },
  { distance: 1000 },
);

const handleLoadMore = async () => {
  if (status.value === "pending") return;

  const totalPages = directoryData.value?.total || 0;

  if (currentPage.value < totalPages) {
    currentPage.value++;
    await refreshData();
  }
};

const onInit = (editor: any) => {
  editor.commands.addCommand({
    name: "saveFile",
    bindKey: { win: "Ctrl-S", mac: "Command-S" },
    exec: () => saveFileContent(false),
  });
};

const showContextMenu = async (item: any, x: number, y: number) => {
  contextMenuInfo.value = { item, x, y };
  selectedItem.value = item;
  await nextTick();
  if (!contextMenu.value?.ctxRef) return false;
  const screenHeight = window.innerHeight;
  const ctxRect = contextMenu.value.ctxRef.getBoundingClientRect();
  isAtBottom.value = ctxRect.bottom > screenHeight;
};

const onAnywhereClicked = (e: MouseEvent) => {
  if (!(e.target as HTMLElement).closest("#item-context-menu")) {
    contextMenuInfo.value.item = null;
  }
};

const sortFiles = (method: string) => {
  sortMethod.value = method;
};

const imageExtensions = ["png", "jpg", "jpeg", "gif", "webp"];

const editFile = async (item: { name: string; type: string; path: string }) => {
  try {
    const path = `${currentPath.value}/${item.name}`.replace("//", "/");
    const content = (await props.server.fs?.downloadFile(path, true)) as any;
    window.scrollTo(0, 0);

    fileContent.value = await content.text();
    editingFile.value = item;
    isEditing.value = true;
    const extension = item.name.split(".").pop();
    if (item.type === "file" && extension && imageExtensions.includes(extension)) {
      isEditingImage.value = true;
      imagePreview.value = content;
    }
    router.push({ query: { ...route.query, path: currentPath.value, editing: item.path } });
  } catch (error) {
    console.error("Error fetching file content:", error);
  }
};

onMounted(async () => {
  await import("ace-builds");
  await import("ace-builds/src-noconflict/mode-json");
  await import("ace-builds/src-noconflict/mode-yaml");
  await import("ace-builds/src-noconflict/mode-toml");
  await import("ace-builds/src-noconflict/mode-sh");
  await import("ace-builds/src-noconflict/theme-one_dark");
  await import("ace-builds/src-noconflict/ext-searchbox");
  VAceEditor.value = markRaw((await import("vue3-ace-editor")).VAceEditor);
  document.addEventListener("click", onAnywhereClicked);
  window.addEventListener("scroll", onScroll);

  document.addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === "z") {
      e.preventDefault();
      undoLastOperation();
    }
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === "z") {
      e.preventDefault();
      redoLastOperation();
    }
  });
});

onUnmounted(() => {
  document.removeEventListener("click", onAnywhereClicked);
  window.removeEventListener("scroll", onScroll);
  document.removeEventListener("keydown", () => {});
});

watch(
  () => route.query,
  async (newQuery) => {
    currentPage.value = 1;
    searchQuery.value = "";
    sortMethod.value = "default";

    currentPath.value = Array.isArray(newQuery.path)
      ? newQuery.path.join("")
      : newQuery.path || "/";

    if (newQuery.editing) {
      await editFile({
        name: newQuery.editing as string,
        type: "file",
        path: newQuery.editing as string,
      });
    } else {
      isEditing.value = false;
      editingFile.value = null;
    }

    await refreshData();
    reset();
  },
  { immediate: true, deep: true },
);

const breadcrumbSegments = computed(() => {
  if (typeof currentPath.value === "string") {
    return currentPath.value.split("/").filter(Boolean);
  }
  return [];
});

const navigateToSegment = (index: number) => {
  const newPath = breadcrumbSegments.value.slice(0, index + 1).join("/");
  router.push({ query: { ...route.query, path: newPath } });
  if (isEditing.value) {
    isEditing.value = false;
    editingFile.value = null;
    closeEditor.value = false;

    const newQuery = { ...route.query };
    delete newQuery.editing;
    router.replace({ query: newQuery });
  }
};

// const navigateToPage = () => {
//   router.push({ query: { path: currentPath.value } });
// };

const requestShareLink = async () => {
  try {
    const response = (await $fetch("https://api.mclo.gs/1/log", {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({ content: fileContent.value }),
    })) as any;

    if (response.success) {
      await navigator.clipboard.writeText(response.url);
      addNotification({
        group: "files",
        title: "Log URL copied",
        text: "Your log file URL has been copied to your clipboard.",
        type: "success",
      });
    } else {
      throw new Error(response.error);
    }
  } catch (error) {
    console.error("Error sharing file:", error);
  }
};

const handleDragEnter = (event: DragEvent) => {
  if (isEditing.value) return;
  event.preventDefault();
  if (!event.dataTransfer?.types.includes("application/pyro-file-move")) {
    dragCounter.value++;
    isDragging.value = true;
  }
};

const handleDragOver = (event: DragEvent) => {
  if (isEditing.value) return;
  event.preventDefault();
};

const handleDragLeave = (event: DragEvent) => {
  if (isEditing.value) return;
  event.preventDefault();
  dragCounter.value--;
  if (dragCounter.value === 0) {
    isDragging.value = false;
  }
};

// eslint-disable-next-line require-await
const handleDrop = async (event: DragEvent) => {
  if (isEditing.value) return;
  event.preventDefault();
  isDragging.value = false;
  dragCounter.value = 0;

  const isInternalMove = event.dataTransfer?.types.includes("application/pyro-file-move");
  if (isInternalMove) return;

  const files = event.dataTransfer?.files;
  if (files) {
    Array.from(files).forEach((file) => {
      uploadFile(file);
    });
  }
};

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
};

const cancelUpload = (item: UploadItem) => {
  if (item.uploader && item.status === "uploading") {
    item.uploader.cancel();
    item.status = "cancelled";

    setTimeout(async () => {
      const index = uploadQueue.value.findIndex((qItem) => qItem.file.name === item.file.name);
      if (index !== -1) {
        uploadQueue.value.splice(index, 1);
        await nextTick();
      }
    }, 5000);
  }
};

const uploadFile = async (file: File) => {
  const uploadItem: UploadItem = {
    file,
    progress: 0,
    status: "pending",
    size: formatFileSize(file.size),
  };

  uploadQueue.value.push(uploadItem);

  try {
    uploadItem.status = "uploading";
    const filePath = `${currentPath.value}/${file.name}`.replace("//", "/");
    const uploader = await props.server.fs?.uploadFile(filePath, file);
    uploadItem.uploader = uploader;

    if (uploader?.onProgress) {
      uploader.onProgress(({ progress }: { progress: number }) => {
        const index = uploadQueue.value.findIndex((item) => item.file.name === file.name);
        if (index !== -1) {
          uploadQueue.value[index].progress = Math.round(progress);
        }
      });
    }

    await uploader?.promise;
    const index = uploadQueue.value.findIndex((item) => item.file.name === file.name);
    if (index !== -1 && uploadQueue.value[index].status !== "cancelled") {
      uploadQueue.value[index].status = "completed";
      uploadQueue.value[index].progress = 100;
    }

    await nextTick();

    setTimeout(async () => {
      const removeIndex = uploadQueue.value.findIndex((item) => item.file.name === file.name);
      if (removeIndex !== -1) {
        uploadQueue.value.splice(removeIndex, 1);
        await nextTick();
      }
    }, 5000);

    await refreshList();
  } catch (error) {
    console.error("Error uploading file:", error);
    const index = uploadQueue.value.findIndex((item) => item.file.name === file.name);
    if (index !== -1 && uploadQueue.value[index].status !== "cancelled") {
      uploadQueue.value[index].status = "error";
    }

    setTimeout(async () => {
      const removeIndex = uploadQueue.value.findIndex((item) => item.file.name === file.name);
      if (removeIndex !== -1) {
        uploadQueue.value.splice(removeIndex, 1);
        await nextTick();
      }
    }, 5000);

    if (error instanceof Error && error.message !== "Upload cancelled") {
      addNotification({
        group: "files",
        title: "Upload failed",
        text: `Failed to upload ${file.name}`,
        type: "error",
      });
    }
  }
};

const initiateFileUpload = () => {
  const input = document.createElement("input");
  input.type = "file";
  input.multiple = true;
  input.onchange = () => {
    if (input.files) {
      Array.from(input.files).forEach((file) => {
        uploadFile(file);
      });
    }
  };
  input.click();
};

const downloadFile = async (item: any) => {
  if (item.type === "file") {
    try {
      const path = `${currentPath.value}/${item.name}`.replace("//", "/");
      const fileData = await props.server.fs?.downloadFile(path);
      if (fileData) {
        const blob = new Blob([fileData], { type: "application/octet-stream" });
        const link = document.createElement("a");
        link.href = window.URL.createObjectURL(blob);
        link.download = item.name;
        link.click();
        window.URL.revokeObjectURL(link.href);
      } else {
        throw new Error("File data is undefined");
      }
    } catch (error) {
      console.error("Error downloading file:", error);
    }
    contextMenuInfo.value.item = null;
  }
};

const saveFileContent = async (exit: boolean = true) => {
  if (!editingFile.value) return;

  try {
    await props.server.fs?.updateFile(editingFile.value.path, fileContent.value);
    if (exit) {
      await props.server.refresh();
      isEditing.value = false;
      editingFile.value = null;
      router.push({ query: { ...route.query, path: currentPath.value } });
    }

    addNotification({
      group: "files",
      title: "File saved",
      text: "Your file has been saved.",
      type: "success",
    });
  } catch (error) {
    console.error("Error saving file content:", error);
  }
};

const saveFileContentRestart = async () => {
  await saveFileContent();
  await props.server.general?.power("Restart");

  addNotification({
    group: "files",
    title: "Server restarted",
    text: "Your server has been restarted.",
    type: "success",
  });
};

const saveFileContentAs = async () => {
  await saveFileContent(false);
  closeEditor.value = true;
  showRenameModal(editingFile.value);
};

const cancelEditing = () => {
  isEditing.value = false;
  editingFile.value = null;
  fileContent.value = "";
  isEditingImage.value = false;
  imagePreview.value = null;
  router.push({ query: { ...route.query, path: currentPath.value } });
  const newQuery = { ...route.query };
  delete newQuery.editing;
  router.replace({ query: newQuery });
};

const onScroll = () => {
  if (contextMenuInfo.value.item) {
    contextMenuInfo.value.y = Math.max(0, contextMenuInfo.value.y - window.scrollY);
  }
};
</script>

<style scoped>
.upload-status {
  overflow: hidden;
  transition: height 0.2s ease;
}

.upload-status-enter-active,
.upload-status-leave-active {
  transition: height 0.2s ease;
  overflow: hidden;
}

.upload-status-enter-from,
.upload-status-leave-to {
  height: 0 !important;
}

.status-icon-enter-active,
.status-icon-leave-active {
  transition: all 0.25s ease;
}

.status-icon-enter-from,
.status-icon-leave-to {
  transform: scale(0);
  opacity: 0;
}

.status-icon-enter-to,
.status-icon-leave-from {
  transform: scale(1);
  opacity: 1;
}
</style>
