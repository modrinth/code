<template>
  <div data-pyro-file-manager-root class="contents">
    <!-- Create Item Modal -->
    <NewModal ref="createItemModal" :header="`Creating a ${newItemType}`">
      <form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleCreateNewItem">
        <div class="flex flex-col gap-2">
          <div class="font-semibold text-contrast">Name<span class="text-red-500">*</span></div>
          <input
            v-model="newItemName"
            autofocus
            type="text"
            class="bg-bg-input w-full rounded-lg p-4"
            required
          />
          <div v-if="createItemSubmitted && nameError" class="text-red">{{ nameError }}</div>
        </div>
        <div class="flex justify-end gap-4">
          <Button transparent @click="createItemModal?.hide()"> Cancel </Button>
          <Button :disabled="!!nameError" color="primary" type="submit"> Create </Button>
        </div>
      </form>
    </NewModal>

    <!-- Rename Item Modal -->
    <NewModal ref="renameItemModal" :header="`Renaming ${selectedItem?.type}`">
      <form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleRenameItem">
        <div class="flex flex-col gap-2">
          <div class="font-semibold text-contrast">Name<span class="text-red-500">*</span></div>
          <input
            ref="renameInput"
            v-model="newItemName"
            autofocus
            type="text"
            class="bg-bg-input w-full rounded-lg p-4"
            :placeholder="`e.g. ${newItemType === 'file' ? 'config.yml' : 'plugins'}`"
            required
          />
          <div v-if="renameItemSubmitted && nameError" class="text-red">{{ nameError }}</div>
        </div>
        <div class="flex justify-end gap-4">
          <Button transparent @click="renameItemModal?.hide()"> Cancel </Button>
          <Button :disabled="!!nameError" color="primary" type="submit"> Rename </Button>
        </div>
      </form>
    </NewModal>

    <!-- Move Item Modal -->
    <NewModal ref="moveItemModal" :header="`Moving ${selectedItem?.name}`">
      <form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleMoveItem">
        <div class="flex flex-col gap-2">
          <input
            v-model="destinationFolder"
            autofocus
            type="text"
            class="bg-bg-input w-full rounded-lg p-4"
            placeholder="e.g. mods/modname"
            required
          />
        </div>
        <div class="flex justify-end gap-4">
          <Button transparent @click="moveItemModal?.hide()"> Cancel </Button>
          <Button color="primary" type="submit"> Move </Button>
        </div>
      </form>
    </NewModal>

    <!-- Delete Item Modal -->
    <NewModal ref="deleteItemModal" danger :header="`Deleting ${selectedItem?.type}`">
      <form class="flex flex-col gap-4 md:w-[600px]" @submit.prevent="handleDeleteItem">
        <div
          class="relative flex w-full items-center gap-2 rounded-2xl border border-solid border-[#cb224436] bg-[#f57b7b0e] p-6 shadow-md dark:border-0 dark:bg-[#0e0e0ea4]"
        >
          <div
            class="flex h-9 w-9 items-center justify-center rounded-full bg-[#3f1818a4] p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
          >
            <FolderOpenIcon v-if="selectedItem?.type === 'directory'" class="h-5 w-5" />
            <FileIcon v-else-if="selectedItem?.type === 'file'" class="h-5 w-5" />
          </div>
          <div class="flex flex-col">
            <span class="font-bold group-hover:text-contrast">{{ selectedItem?.name }}</span>
            <span
              v-if="selectedItem?.type === 'directory'"
              class="text-xs text-secondary group-hover:text-primary"
              >{{ selectedItem?.count }} items</span
            >
            <span v-else class="text-xs text-secondary group-hover:text-primary"
              >{{ (selectedItem?.size / 1024 / 1024).toFixed(2) }} MB</span
            >
          </div>
        </div>
        <div class="flex justify-end gap-4">
          <Button transparent @click="deleteItemModal?.hide()"> Cancel </Button>
          <Button color="danger" type="submit"> Delete {{ selectedItem?.type }} </Button>
        </div>
      </form>
    </NewModal>
    <div
      class="relative flex w-full flex-col rounded-2xl border border-solid border-bg-raised"
      @dragenter.prevent="handleDragEnter"
      @dragover.prevent="handleDragOver"
      @dragleave.prevent="handleDragLeave"
      @drop.prevent="handleDrop"
    >
      <!-- Main Content -->
      <div ref="mainContent" class="flex h-[40rem] w-full flex-col">
        <nav
          v-if="!isEditing"
          data-pyro-files-state="browsing"
          class="top-0 flex h-12 select-none items-center justify-between rounded-t-2xl bg-table-alternateRow p-3"
        >
          <ul class="flex list-none items-center p-0 text-contrast">
            <li
              v-tooltip="'Back to home'"
              role="link"
              class="breadcrumb-link grid size-8 cursor-pointer place-content-center rounded-full bg-bg-raised p-[6px] hover:bg-brand-highlight hover:text-brand"
              @click="navigateToSegment(-1)"
            >
              <BoxIcon class="size-5" />
            </li>
            <UiServersSlashIcon class="h-5 w-5" />
            <li
              v-for="(segment, index) in breadcrumbSegments"
              :key="index"
              class="breadcrumb-link flex cursor-pointer items-center"
              @click="navigateToSegment(index)"
            >
              {{ segment || "" }}
              <UiServersSlashIcon class="h-5 w-5" />
            </li>
          </ul>
          <div class="flex items-center gap-1">
            <ButtonStyled type="transparent">
              <UiServersTeleportOverflowMenu
                position="bottom"
                direction="left"
                aria-label="Sort files"
                :options="[
                  { id: 'normal', action: () => sortFiles('default') },
                  { id: 'modified', action: () => sortFiles('modified') },
                  { id: 'filesOnly', action: () => sortFiles('filesOnly') },
                  { id: 'foldersOnly', action: () => sortFiles('foldersOnly') },
                ]"
              >
                <span class="whitespace-pre text-sm font-medium">
                  {{
                    sortMethod === "default"
                      ? "Default Sort"
                      : sortMethod === "modified"
                        ? "Recently Modified"
                        : sortMethod === "filesOnly"
                          ? "Files Only"
                          : sortMethod === "foldersOnly"
                            ? "Folders Only"
                            : "Default Sort"
                  }}
                </span>
                <SortAscendingIcon aria-hidden="true" />
                <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
                <template #normal> Default Sort </template>
                <template #modified> Recently Modified </template>
                <template #filesOnly> Files Only </template>
                <template #foldersOnly> Folders Only </template>
              </UiServersTeleportOverflowMenu>
            </ButtonStyled>
            <div class="relative w-full text-sm">
              <label for="search-folder" class="sr-only">Search folder</label>
              <SearchIcon
                class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2"
                aria-hidden="true"
              />
              <input
                id="search-folder"
                v-model="searchQuery"
                type="search"
                name="search"
                autocomplete="off"
                class="h-8 min-h-[unset] border-[1px] border-solid border-button-bg bg-transparent py-2 pl-9"
                placeholder="Search..."
              />
            </div>

            <ButtonStyled type="transparent">
              <UiServersTeleportOverflowMenu
                position="bottom"
                direction="left"
                aria-label="Create new..."
                :options="[
                  { id: 'file', action: () => showCreateModal('file') },
                  { id: 'directory', action: () => showCreateModal('directory') },
                  { id: 'upload', action: () => initiateFileUpload() },
                ]"
              >
                <PlusIcon aria-hidden="true" />
                <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
                <template #file> <BoxIcon aria-hidden="true" /> File </template>
                <template #directory> <FolderOpenIcon aria-hidden="true" /> Directory </template>
                <template #upload> <UploadIcon aria-hidden="true" /> Upload </template>
              </UiServersTeleportOverflowMenu>
            </ButtonStyled>
          </div>
        </nav>

        <nav
          v-else
          data-pyro-files-state="editing"
          class="flex h-12 select-none items-center justify-between rounded-t-2xl bg-table-alternateRow p-3"
        >
          <div class="flex items-center gap-2 text-contrast">
            <ButtonStyled type="transparent">
              <button @click="cancelEditing">
                <XIcon aria-hidden="true" />
              </button>
            </ButtonStyled>
            <span class="breadcrumb-link flex cursor-pointer items-center gap-2">
              <span class="text-lg font-bold">{{ editingFile?.name }}</span>
            </span>
          </div>

          <div class="flex gap-2">
            <Button
              v-if="editingFile.path.startsWith('logs') && editingFile.path.endsWith('.log')"
              v-tooltip="'Share to mclo.gs'"
              icon-only
              transparent
              @click="requestShareLink"
            >
              <ShareIcon />
            </Button>
            <ButtonStyled type="transparent">
              <UiServersTeleportOverflowMenu
                position="bottom"
                direction="left"
                aria-label="Save file"
                :options="[
                  { id: 'save', action: saveFileContent },
                  { id: 'save-as', action: saveFileContentAs },
                  { id: 'save&restart', action: saveFileContentRestart },
                ]"
              >
                <SaveIcon aria-hidden="true" />
                <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
                <template #save> <SaveIcon aria-hidden="true" /> Save </template>
                <template #save-as> <SaveIcon aria-hidden="true" /> Save as... </template>
                <template #save&restart>
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                    <path
                      fill-rule="evenodd"
                      d="M15.312 11.424a5.5 5.5 0 0 1-9.201 2.466l-.312-.311h2.433a.75.75 0 0 0 0-1.5H3.989a.75.75 0 0 0-.75.75v4.242a.75.75 0 0 0 1.5 0v-2.43l.31.31a7 7 0 0 0 11.712-3.138.75.75 0 0 0-1.449-.39Zm1.23-3.723a.75.75 0 0 0 .219-.53V2.929a.75.75 0 0 0-1.5 0V5.36l-.31-.31A7 7 0 0 0 3.239 8.188a.75.75 0 1 0 1.448.389A5.5 5.5 0 0 1 13.89 6.11l.311.31h-2.432a.75.75 0 0 0 0 1.5h4.243a.75.75 0 0 0 .53-.219Z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  Save & Restart
                </template>
              </UiServersTeleportOverflowMenu>
            </ButtonStyled>
          </div>
        </nav>

        <div v-if="isEditing" class="h-full w-full flex-grow">
          <component
            :is="VAceEditor"
            v-model:value="fileContent"
            lang="json"
            theme="one_dark"
            :print-margin="false"
            style="height: 750px; font-size: 1rem"
            class="ace_editor ace_hidpi ace-one-dark ace_dark rounded-b-lg"
            @init="onInit"
          />
        </div>
        <div
          v-else-if="items.length > 0"
          ref="scrollContainer"
          class="h-full w-full overflow-y-auto rounded-b-2xl"
        >
          <UiServersFileItem
            v-for="item in filteredItems"
            :key="item.name"
            :count="item.count"
            :created="item.created"
            :modified="item.modified"
            :name="item.name"
            :path="item.path"
            :type="item.type"
            :size="item.size"
            @delete="showDeleteModal(item)"
            @rename="showRenameModal(item)"
            @download="downloadFile(item)"
            @move="showMoveModal(item)"
            @edit="editFile(item)"
            @contextmenu="(x, y) => showContextMenu(item, x, y)"
          />
          <div v-if="loadError" class="flex h-10 items-center justify-center gap-2">
            <ClearIcon class="h-4 w-4" />
            Error loading more directories {{ loadError }}
          </div>
          <div
            v-else-if="isLoading"
            class="flex h-10 animate-pulse items-center justify-center gap-2"
          >
            <BrandLogoAnimated />
          </div>
        </div>

        <div
          v-else-if="!isLoading && items.length === 0"
          class="flex h-full w-full items-center justify-center p-20"
        >
          <div class="flex flex-col items-center gap-4 text-center">
            <FolderOpenIcon class="h-16 w-16 text-secondary" />
            <h3 class="text-2xl font-bold text-contrast">This folder is empty</h3>
            <p class="m-0 text-sm text-secondary">There are no files or folders in this folder.</p>
          </div>
        </div>

        <div
          v-else-if="!isLoading"
          class="flex h-full w-full items-center justify-center gap-6 p-20"
        >
          <FileIcon class="text-red-500 h-16 w-16" />
          <div class="flex flex-col gap-2">
            <h3 class="text-red-500 m-0 text-2xl font-bold">Unable to list files</h3>
            <p class="m-0 text-sm text-secondary">
              Unfortunately, we were unable to list the files in this folder. If this issue
              persists, contact support.
            </p>
          </div>
          <Button size="sm" @click="navigateToPage()">
            <XIcon class="h-5 w-5" />
            Go to homepage
          </Button>
        </div>

        <div
          v-else-if="loadError"
          class="flex h-full w-full items-center justify-center gap-6 p-20"
        >
          <FileIcon class="text-red-500 h-16 w-16" />
          <div class="flex flex-col gap-2">
            <h3 class="text-red-500 m-0 text-2xl font-bold">Unable to fetch files</h3>
            <p class="m-0 text-sm text-secondary">
              This path is no longer valid or we couldn't find it. If this issue persists, contact
              support.
            </p>
          </div>
        </div>
        <div v-else class=""></div>
      </div>
      <div
        v-if="isDragging"
        class="absolute inset-0 flex items-center justify-center rounded-xl bg-black bg-opacity-50 text-white"
      >
        <div class="text-center">
          <UploadIcon class="mx-auto h-16 w-16" />
          <p class="mt-2 text-xl">Drop files here to upload</p>
        </div>
      </div>
    </div>

    <div
      class="fixed"
      :style="{
        transform: `translateY(${isAtBottom ? '-100%' : '0'})`,
        top: `${contextMenuInfo.y}px`,
        left: `${contextMenuInfo.x}px`,
      }"
    >
      <Transition>
        <div
          v-if="contextMenuInfo.item"
          id="item-context-menu"
          ref="ctxRef"
          :style="{
            border: '1px solid var(--color-button-bg)',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--color-raised-bg)',
            padding: 'var(--gap-sm)',
            boxShadow: 'var(--shadow-floating)',
            gap: 'var(--gap-xs)',
            width: 'max-content',
            // '--translate-y': isAtBottom ? '-100%' : '0',
          }"
          class="flex h-fit w-fit select-none flex-col"
        >
          <button
            class="btn btn-transparent flex !w-full items-center"
            @click="showRenameModal(contextMenuInfo.item)"
          >
            <EditIcon class="h-5 w-5" />
            Rename
          </button>
          <button
            class="btn btn-transparent flex !w-full items-center"
            @click="showMoveModal(contextMenuInfo.item)"
          >
            <ArrowBigUpDashIcon class="h-5 w-5" />
            Move
          </button>
          <button
            v-if="contextMenuInfo.item.type !== 'directory'"
            class="btn btn-transparent flex !w-full items-center"
            @click="downloadFile(contextMenuInfo.item)"
          >
            <DownloadIcon class="h-5 w-5" />
            Download
          </button>
          <button
            class="btn btn-transparent btn-red flex !w-full items-center"
            @click="showDeleteModal(contextMenuInfo.item)"
          >
            <TrashIcon class="h-5 w-5" />
            Delete
          </button>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  BoxIcon,
  FileIcon,
  PlusIcon,
  UploadIcon,
  XIcon,
  DropdownIcon,
  FolderOpenIcon,
  SaveIcon,
  ClearIcon,
  EditIcon,
  ArrowBigUpDashIcon,
  DownloadIcon,
  TrashIcon,
  SearchIcon,
  ShareIcon,
  SortAscendingIcon,
} from "@modrinth/assets";
import { Button, NewModal, ButtonStyled } from "@modrinth/ui";
import { useInfiniteScroll } from "@vueuse/core";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const VAceEditor = ref();

const searchQuery = ref("");
const sortMethod = ref("default-sort");

const mainContent = ref<HTMLElement | null>(null);

const renameInput = ref<HTMLInputElement | null>(null);

const applyDefaultSort = (items: any[]) => {
  return items.sort((a: any, b: any) => {
    if (a.type === "directory" && b.type !== "directory") return -1;
    if (a.type !== "directory" && b.type === "directory") return 1;
    if (a.count > b.count) return -1;
    if (a.count < b.count) return 1;
    if (a.name > b.name) return 1;
    if (a.name < b.name) return -1;
    return 0;
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

const route = useRoute();
const router = useRouter();
const serverId = route.params.id.toString();
const data = computed(() => props.server.general);

const maxResults = 100;
const currentPage = ref(1);
const pages = ref(1);
const currentPath = ref(route.query.path || "");
const ctxRef = ref<HTMLElement | null>(null);

const isAtBottom = ref(false);

useHead({
  title: `Files - ${data.value?.name ?? "Server"} - Modrinth`,
});

const scrollContainer = ref<HTMLElement | null>(null);
const items = ref<any[]>([]);
const isLoading = ref(true);
const loadError = ref(false);
const contextMenuInfo = ref<{
  item: any;
  x: number;
  y: number;
}>({
  item: null,
  x: 0,
  y: 0,
});

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const { reset } = useInfiniteScroll(
  scrollContainer,
  () => {
    if (currentPage.value <= pages.value) {
      fetchData();
    }
  },
  { distance: 1000 },
);

const onInit = (editor: any) => {
  editor.commands.addCommand({
    name: "saveFile",
    bindKey: { win: "Ctrl-S", mac: "Command-S" },
    exec: () => saveFileContent(false),
  });
};

watch(contextMenuInfo, () => console.log(contextMenuInfo.value), {
  deep: true,
  immediate: true,
});

let dY = 0;
let oldScroll = 0;

const onScroll = () => {
  dY = window.scrollY - oldScroll;
  oldScroll = window.scrollY;

  contextMenuInfo.value.y -= dY;
  contextMenuInfo.value.item = null;
};

onMounted(() => {
  window.addEventListener("scroll", onScroll);
});

onUnmounted(() => {
  window.removeEventListener("scroll", onScroll);
});

const showContextMenu = async (item: any, x: number, y: number) => {
  dY = 0;
  oldScroll = window.scrollY;
  contextMenuInfo.value = { item, x, y };
  selectedItem.value = item;
  await nextTick();
  if (!ctxRef.value) return false;
  const screenHeight = window.innerHeight;
  const ctxRect = ctxRef.value.getBoundingClientRect();
  isAtBottom.value = isAtBottom.value
    ? ctxRect.bottom + ctxRect.height > screenHeight
    : ctxRect.bottom > screenHeight;
  console.log(ctxRect.bottom, screenHeight, isAtBottom.value);
};

const onAnywhereClicked = (e: MouseEvent) => {
  // if the clicked element does not have a parent with the id of item-context-menu, hide the context menu
  if (!(e.target as HTMLElement).closest("#item-context-menu")) contextMenuInfo.value.item = null;
};

onMounted(() => {
  document.addEventListener("click", onAnywhereClicked);
});

onUnmounted(() => {
  document.removeEventListener("click", onAnywhereClicked);
});

const sortFiles = (method: string) => {
  sortMethod.value = method;
};

const fetchData = async () => {
  isLoading.value = true;
  loadError.value = false;

  try {
    const path = Array.isArray(currentPath.value) ? currentPath.value.join("") : currentPath.value;
    const data = await props.server.fs?.listDirContents(path, currentPage.value, maxResults);

    if (!data || !data.items) {
      throw new Error("Invalid data structure received from server.");
    }

    data.items = applyDefaultSort(data.items);

    items.value.push(...data.items);
    pages.value = data.total;

    currentPage.value++;
    isLoading.value = false;
  } catch (error) {
    console.error("Error fetching data:", error);
    loadError.value = true;
  }
};

onMounted(async () => {
  await import("ace-builds");
  await import("ace-builds/src-noconflict/mode-json");
  await import("ace-builds/src-noconflict/theme-one_dark");
  VAceEditor.value = markRaw((await import("vue3-ace-editor")).VAceEditor);
  if (serverId) {
    await fetchData();
  }
});

watch(
  () => route.query.path,
  async (newPath) => {
    console.log(`Path changed to: ${newPath}`);
    currentPath.value = newPath || "/";
    currentPage.value = 1;
    items.value = [];
    loadError.value = false;
    searchQuery.value = "";
    sortMethod.value = "default";
    if (serverId) {
      await fetchData();
    }
  },
);

const breadcrumbSegments = computed(() => {
  if (typeof currentPath.value === "string") {
    return currentPath.value.split("/").filter(Boolean);
  }
  return [];
});

const navigateToSegment = (index: number) => {
  const newPath = breadcrumbSegments.value.slice(0, index + 1).join("/");
  router.push({ query: { path: newPath } });
};

const navigateToPage = () => {
  router.push({ query: { path: currentPath.value } });
};

const createItemModal = ref<typeof NewModal>();
const deleteItemModal = ref<typeof NewModal>();
const renameItemModal = ref<typeof NewModal>();
const moveItemModal = ref<typeof NewModal>();
const newItemType = ref<"file" | "directory">("file");
const newItemName = ref("");
const selectedItem = ref<any>(null);
const destinationFolder = ref("");
const isEditing = ref(false);
const fileContent = ref("");
const editingFile = ref<any>(null);
const closeEditor = ref(false);
const createItemSubmitted = ref(false);
const renameItemSubmitted = ref(false);

const nameError = computed(() => {
  if (!newItemName.value) {
    return "Name is required.";
  }
  if (newItemType.value === "file") {
    const validPattern = /^[a-zA-Z0-9-_]+$/;
    if (!validPattern.test(newItemName.value)) {
      return "Name must contain only alphanumeric characters, dashes, or underscores.";
    }
  } else if (newItemType.value === "directory") {
    const validPattern = /^[a-zA-Z0-9-_]+$/;
    if (!validPattern.test(newItemName.value)) {
      return "Name must contain only alphanumeric characters, dashes, or underscores.";
    }
  }
  return "";
});

const requestShareLink = async () => {
  try {
    const response = (await $fetch("https://api.mclo.gs/1/log", {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams({
        content: fileContent.value,
      }),
    })) as any;

    if (response.success) {
      navigator.clipboard.writeText(response.url);
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

const showCreateModal = (type: "file" | "directory") => {
  newItemType.value = type;
  newItemName.value = "";
  createItemModal.value?.show();
};

const isDragging = ref(false);
const dragCounter = ref(0);

const handleDragEnter = (event: DragEvent) => {
  event.preventDefault();
  dragCounter.value++;
  isDragging.value = true;
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

const handleDrop = async (event: DragEvent) => {
  event.preventDefault();
  isDragging.value = false;
  dragCounter.value = 0;
  const files = event.dataTransfer?.files;
  if (files) {
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      await uploadFile(file);
    }
  }
};

const uploadFile = async (file: File) => {
  try {
    const filePath = `${currentPath.value}/${file.name}`.replace("//", "/");
    await props.server.fs?.uploadFile(filePath, file);
    await fetchData();

    addNotification({
      group: "files",
      title: "File uploaded",
      text: "Your file has been uploaded.",
      type: "success",
    });
  } catch (error) {
    console.error("Error uploading file:", error);
  }
};

const initiateFileUpload = () => {
  const input = document.createElement("input");
  input.type = "file";
  input.onchange = async () => {
    const file = input.files?.[0];
    if (file) {
      await uploadFile(file);
    }
  };
  input.click();
  input.remove();
};

const showRenameModal = async (item: any) => {
  selectedItem.value = item;
  newItemName.value = item.name;
  newItemType.value = item.type;
  renameItemModal.value?.show();
  contextMenuInfo.value.item = null;
  await nextTick();
  // god forgive me
  setTimeout(() => {
    renameInput.value?.focus();
  }, 100);
};

const showMoveModal = (item: any) => {
  selectedItem.value = item;
  destinationFolder.value = Array.isArray(currentPath.value)
    ? currentPath.value.join("/")
    : currentPath.value;
  moveItemModal.value?.show();
  contextMenuInfo.value.item = null;
};

const showDeleteModal = (item: any) => {
  selectedItem.value = item;
  deleteItemModal.value?.show();
  contextMenuInfo.value.item = null;
};

const handleCreateNewItem = () => {
  createItemSubmitted.value = true;
  if (!nameError.value) {
    createNewItem();
  }
};

const handleRenameItem = () => {
  renameItemSubmitted.value = true;
  if (!nameError.value) {
    renameItem();
  }
};

const handleMoveItem = () => {
  moveItem();
};

const handleDeleteItem = () => {
  deleteItem();
};

const createNewItem = async () => {
  if (!nameError.value) {
    try {
      const path = `${currentPath.value}/${newItemName.value}`.replace("//", "/");
      await props.server.fs?.createFileOrFolder(path, newItemType.value);

      currentPage.value = 1;
      items.value = [];
      await fetchData();
      createItemModal.value?.hide();
      createItemSubmitted.value = false;

      addNotification({
        group: "files",
        title: "File created",
        text: "Your file has been created.",
        type: "success",
      });
      newItemName.value = "";
    } catch (error) {
      console.error("Error creating item:", error);
      if (error instanceof PyroFetchError && error.statusCode === 400) {
        addNotification({
          group: "files",
          title: "Error creating item",
          text: "Invalid file",
          type: "error",
        });
      } else if (error instanceof PyroFetchError && error.statusCode === 500) {
        addNotification({
          group: "files",
          title: "Error creating item",
          text: "File already exists",
          type: "error",
        });
      }
    }
  }
};

const renameItem = async () => {
  try {
    await props.server.fs?.renameFileOrFolder(
      `${currentPath.value}/${selectedItem.value.name}`,
      newItemName.value,
    );

    currentPage.value = 1;
    items.value = [];
    await fetchData();
    renameItemModal.value?.hide();
    renameItemSubmitted.value = false;

    if (closeEditor.value) {
      await props.server.refresh();
      isEditing.value = false;
      editingFile.value = null;
      closeEditor.value = false;
      await fetchData();
    }

    addNotification({
      group: "files",
      title: "File renamed",
      text: "Your file has been renamed.",
      type: "success",
    });
  } catch (error) {
    console.error("Error renaming item:", error);
    if (error instanceof PyroFetchError && error.statusCode === 400) {
      addNotification({
        group: "files",
        title: "Could not rename item",
        text: "This item already exists or is invalid.",
        type: "error",
      });
    } else if (error instanceof PyroFetchError && error.statusCode === 500) {
      addNotification({
        group: "files",
        title: "Could not rename item",
        text: "Invalid file",
        type: "error",
      });
    }
  }
};

const moveItem = async () => {
  if (!selectedItem.value || !destinationFolder.value) return;

  try {
    await props.server.fs?.moveFileOrFolder(
      `${currentPath.value}/${selectedItem.value.name}`.replace("//", "/"),
      `${destinationFolder.value}/${selectedItem.value.name}`.replace("//", "/"),
    );

    currentPage.value = 1;
    items.value = [];
    await fetchData();
    moveItemModal.value?.hide();

    addNotification({
      group: "files",
      title: "File moved",
      text: "Your file has been moved.",
      type: "success",
    });
  } catch (error) {
    console.error("Error moving item:", error);
  }
};

const downloadFile = async (item: any) => {
  if (item.type === "file") {
    try {
      const path = `${currentPath.value}/${item.name}`.replace("//", "/");
      const fileData = await props.server.fs?.downloadFile(path);
      if (fileData) {
        console.log(fileData);

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

const deleteItem = async () => {
  try {
    const path = `${currentPath.value}/${selectedItem.value.name}`.replace("//", "/");
    await props.server.fs?.deleteFileOrFolder(path, selectedItem.value.type === "directory");
    currentPage.value = 1;
    items.value = [];
    await fetchData();
    deleteItemModal.value?.hide();

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

const editFile = async (item: { name: string; type: string; path: string }) => {
  try {
    const path = `${currentPath.value}/${item.name}`.replace("//", "/");
    const content = (await props.server.fs?.downloadFile(path)) as string;

    window.scrollTo(0, 0);

    fileContent.value = content;
    editingFile.value = item;
    isEditing.value = true;
  } catch (error) {
    console.error("Error fetching file content:", error);
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
};
</script>

<style>
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
