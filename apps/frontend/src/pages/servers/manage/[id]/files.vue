<template>
  <div
    class="relative z-10 flex min-h-[800px] w-full flex-col overflow-visible rounded-xl border border-solid border-bg-raised"
    @dragenter.prevent="handleDragEnter"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop"
  >
    <!-- Create Item Modal -->
    <Modal ref="createItemModal" header="">
      <UiServersPyroModal
        :header="`Create ${newItemType}`"
        :data="data"
        @modal="createItemModal?.hide()"
      >
        <div class="px-4">
          <div class="mt-2 flex flex-col gap-2">
            <div class="font-semibold text-contrast">Name<span class="text-red-500">*</span></div>
            <input v-model="newItemName" type="text" class="bg-bg-input w-full rounded-lg p-4" />
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="createItemModal?.hide()"> Cancel </Button>
            <Button color="primary" @click="createNewItem"> Create </Button>
          </div>
        </div>
      </UiServersPyroModal>
    </Modal>

    <!-- Rename Item Modal -->
    <Modal ref="renameItemModal" header="">
      <UiServersPyroModal
        :header="`Rename ${selectedItem?.type}`"
        :data="data"
        @modal="renameItemModal?.hide()"
      >
        <div class="px-4">
          <div class="mt-2 flex flex-col gap-2">
            <div class="font-semibold text-contrast">Name<span class="text-red-500">*</span></div>
            <input
              v-model="newItemName"
              type="text"
              class="bg-bg-input w-full rounded-lg p-4"
              :placeholder="`e.g. ${newItemType === 'file' ? 'config.yml' : 'plugins'}`"
            />
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="renameItemModal?.hide()"> Cancel </Button>
            <Button color="primary" @click="renameItem"> Rename </Button>
          </div>
        </div>
      </UiServersPyroModal>
    </Modal>

    <!-- Move Item Modal -->
    <Modal ref="moveItemModal" header="">
      <UiServersPyroModal
        :header="`Move ${selectedItem?.name}`"
        :data="data"
        @modal="moveItemModal?.hide()"
      >
        <div class="px-4">
          <div class="mt-2 flex flex-col gap-2">
            <input
              v-model="destinationFolder"
              type="text"
              class="bg-bg-input w-full rounded-lg p-4"
              placeholder="e.g. mods/modname"
            />
          </div>
          <div class="mb-4 mt-4 flex justify-end gap-4">
            <Button transparent @click="moveItemModal?.hide()"> Cancel </Button>
            <Button color="primary" @click="moveItem"> Move </Button>
          </div>
        </div>
      </UiServersPyroModal>
    </Modal>

    <!-- Delete Item Modal -->
    <Modal ref="deleteItemModal" header="">
      <UiServersPyroModal
        :header="`Delete ${selectedItem?.type}`"
        :data="data"
        danger
        @modal="deleteItemModal?.hide()"
      >
        <div class="flex flex-col gap-4">
          <div class="relative flex w-full items-center gap-2 rounded-2xl bg-[#0e0e0ea4] p-6">
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
        </div>
        <div class="mb-4 mt-4 flex justify-end gap-4">
          <Button transparent @click="deleteItemModal?.hide()"> Cancel </Button>
          <Button color="danger" @click="deleteItem"> Delete {{ selectedItem?.type }} </Button>
        </div>
      </UiServersPyroModal>
    </Modal>

    <!-- Main Content -->
    <div
      class="flex min-h-[800px] w-full flex-col overflow-visible rounded-xl border border-solid border-bg-raised"
    >
      <div
        v-if="!isEditing"
        class="flex h-12 select-none items-center justify-between gap-2 rounded-t-xl bg-table-alternateRow px-4 py-2"
      >
        <div class="flex items-center gap-2 text-contrast">
          <span
            class="breadcrumb-link flex cursor-pointer items-center gap-2"
            @click="navigateToSegment(-1)"
          >
            <BoxIcon class="h-5 w-5" />
            <span class="opacity-50">/</span>
          </span>
          <span
            v-for="(segment, index) in breadcrumbSegments"
            :key="index"
            class="breadcrumb-link cursor-pointer"
            @click="navigateToSegment(index)"
          >
            {{ segment || "" }}
            <span class="ms-1 opacity-50">/</span>
          </span>
        </div>
        <ButtonStyled type="transparent">
          <OverflowMenu
            class="btn-dropdown-animation flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
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
          </OverflowMenu>
        </ButtonStyled>
      </div>
      <div
        v-else
        class="flex h-12 items-center justify-between gap-2 rounded-t-xl bg-table-alternateRow px-4 py-2"
      >
        <div class="flex items-center gap-2 text-contrast">
          <ButtonStyled type="transparent">
            <Button @click="cancelEditing">
              <XIcon aria-hidden="true" />
            </Button>
          </ButtonStyled>
          <span class="breadcrumb-link flex cursor-pointer items-center gap-2">
            <span class="text-lg font-bold">{{ editingFile?.name }}</span>
          </span>
        </div>

        <div class="flex gap-2">
          <ButtonStyled type="transparent">
            <OverflowMenu
              class="btn-dropdown-animation flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
              position="bottom"
              direction="left"
              aria-label="Save file"
              :options="[
                { id: 'save', action: () => saveFileContent },
                { id: 'save&restart', action: () => saveFileContentRestart },
              ]"
            >
              <SaveIcon aria-hidden="true" />
              <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
              <template #save> <SaveIcon aria-hidden="true" /> Save </template>
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
            </OverflowMenu>
          </ButtonStyled>
        </div>
      </div>

      <div v-if="isEditing" class="h-full w-full flex-grow overflow-visible">
        <component
          :is="VAceEditor"
          v-model:value="fileContent"
          lang="json"
          theme="one_dark"
          :print-margin="false"
          style="height: 100%"
        />
      </div>
      <div
        v-else-if="items.length > 0"
        ref="scrollContainer"
        class="h-full w-full snap-y overflow-visible"
      >
        <UiServersFileItem
          v-for="item in items"
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
        />
        <div v-if="loadError" class="flex h-10 items-center justify-center gap-2">
          <ClearIcon class="h-4 w-4" />
          Error loading more directories {{ loadError }}
        </div>
        <div
          v-else-if="isLoading"
          class="flex h-10 animate-pulse items-center justify-center gap-2"
        >
          <PyroIcon class="h-4 w-4" /> Loading...
        </div>
      </div>

      <div v-else-if="!isLoading" class="flex h-full w-full items-center justify-center gap-6 p-20">
        <FileIcon class="text-red-500 h-16 w-16" />
        <div class="flex flex-col gap-2">
          <h3 class="text-red-500 m-0 text-2xl font-bold">Unable to fetch file api info</h3>
          <p class="m-0 text-sm text-secondary">
            Don't worry, your server is safe. We just can't get to your files right now.
            <br />
            Wait a few seconds and try again. If your issue persists, contact support.
          </p>
        </div>
        <Button size="sm" @click="navigateToPage(1)">
          <XIcon class="h-5 w-5" />
          Go to homepage
        </Button>
      </div>
      <div v-else-if="loadError" class="flex h-full w-full items-center justify-center gap-6 p-20">
        <FileIcon class="text-red-500 h-16 w-16" />
        <div class="flex flex-col gap-2">
          <h3 class="text-red-500 m-0 text-2xl font-bold">Unable to fetch files</h3>
          <p class="m-0 text-sm text-secondary">
            This path is no longer valid or we couldn't find it. If this issue persists, contact
            support.
          </p>
        </div>
      </div>
      <div v-else class="flex h-full w-full items-center justify-center p-20">
        <UiServersPyroLoading />
      </div>
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
  PyroIcon,
  ClearIcon,
} from "@modrinth/assets";
import { Button, Modal, ButtonStyled, OverflowMenu } from "@modrinth/ui";
import { useInfiniteScroll } from "@vueuse/core";
const VAceEditor = ref();

const app = useNuxtApp();
const route = useRoute();
const router = useRouter();
const serverId = route.params.id.toString();
const serverStore = useServerStore();

const maxResults = 100;
const currentPage = ref(1);
const pages = ref(1);
const currentPath = ref(route.query.path || "");

useHead({
  title: `Files - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

const data = computed(() => serverStore.serverData[serverId]);

const scrollContainer = ref<HTMLElement | null>(null);
const items = ref<any[]>([]);
const isLoading = ref(true);
const loadError = ref(false);

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const { reset } = useInfiniteScroll(
  scrollContainer,
  () => {
    if (currentPage.value <= pages.value) {
      fetchData();
    }
  },
  { distance: 200 },
);

const fetchData = async () => {
  isLoading.value = true;
  loadError.value = false;

  try {
    const path = Array.isArray(currentPath.value) ? currentPath.value.join("") : currentPath.value;
    const data = await serverStore.listDirContents(serverId, path, currentPage.value, maxResults);

    if (!data || !data.items) {
      throw new Error("Invalid data structure received from server.");
    }

    data.items.sort((a: any, b: any) => {
      if (a.type === "directory" && b.type !== "directory") return -1;
      if (a.type !== "directory" && b.type === "directory") return 1;
      if (a.count > b.count) return -1;
      if (a.count < b.count) return 1;
      if (a.name > b.name) return 1;
      if (a.name < b.name) return -1;
      return 0;
    });

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

const navigateToPage = (page: number) => {
  router.push({ query: { page, path: currentPath.value } });
};

const createItemModal = ref<typeof Modal>();
const deleteItemModal = ref<typeof Modal>();
const renameItemModal = ref<typeof Modal>();
const moveItemModal = ref<typeof Modal>();
const newItemType = ref<"file" | "directory">("file");
const newItemName = ref("");
const selectedItem = ref<any>(null);
const destinationFolder = ref("");
const isEditing = ref(false);
const fileContent = ref("");
const editingFile = ref<any>(null);

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
    await serverStore.uploadFile(serverId, filePath, file);
    await fetchData();
    // @ts-ignore
    app.$notify({
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

const showRenameModal = (item: any) => {
  selectedItem.value = item;
  newItemName.value = item.name;
  newItemType.value = item.type;
  renameItemModal.value?.show();
};

const showMoveModal = (item: any) => {
  selectedItem.value = item;
  destinationFolder.value = Array.isArray(currentPath.value)
    ? currentPath.value.join("/")
    : currentPath.value;
  moveItemModal.value?.show();
};

const showDeleteModal = (item: any) => {
  selectedItem.value = item;
  deleteItemModal.value?.show();
};

const createNewItem = async () => {
  try {
    const path = `${currentPath.value}/${newItemName.value}`.replace("//", "/");
    await serverStore.createFileOrFolder(serverId, path, newItemType.value);

    currentPage.value = 1;
    newItemName.value = "";
    items.value = [];
    await fetchData();
    createItemModal.value?.hide();
    // @ts-ignore
    app.$notify({
      group: "files",
      title: "File created",
      text: "Your file has been created.",
      type: "success",
    });
  } catch (error) {
    console.error("Error creating item:", error);
  }
};

const renameItem = async () => {
  try {
    await serverStore.renameFileOrFolder(
      serverId,
      `${currentPath.value}/${selectedItem.value.name}`,
      newItemName.value,
    );

    currentPage.value = 1;
    items.value = [];
    await fetchData();
    renameItemModal.value?.hide();
    // @ts-ignore
    app.$notify({
      group: "files",
      title: "File renamed",
      text: "Your file has been renamed.",
      type: "success",
    });
  } catch (error) {
    console.error("Error renaming item:", error);
  }
};

const moveItem = async () => {
  if (!selectedItem.value || !destinationFolder.value) return;

  try {
    await serverStore.moveFileOrFolder(
      serverId,
      `${currentPath.value}/${selectedItem.value.name}`.replace("//", "/"),
      `${destinationFolder.value}/${selectedItem.value.name}`.replace("//", "/"),
    );

    currentPage.value = 1;
    items.value = [];
    await fetchData();
    moveItemModal.value?.hide();
    // @ts-ignore
    app.$notify({
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
      const fileData = await serverStore.downloadFile(serverId, path);
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
  }
};

const deleteItem = async () => {
  try {
    await serverStore.deleteFileOrFolder(
      serverId,
      `${currentPath.value}/${selectedItem.value.name}`,
      selectedItem.value.type === "directory",
    );
    currentPage.value = 1;
    items.value = [];
    await fetchData();
    deleteItemModal.value?.hide();
    // @ts-ignore
    app.$notify({
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
    const content = (await serverStore.downloadFile(serverId, path)) as string;

    fileContent.value = content;
    editingFile.value = item;
    isEditing.value = true;
  } catch (error) {
    console.error("Error fetching file content:", error);
  }
};

const saveFileContent = async () => {
  if (!editingFile.value) return;

  try {
    await serverStore.updateFile(serverId, editingFile.value.path, fileContent.value);
    await refreshNuxtData("files-data");
    isEditing.value = false;
    editingFile.value = null;
    // @ts-ignore
    app.$notify({
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
  await serverStore.sendPowerAction(serverId, "Restart");
};

const cancelEditing = () => {
  isEditing.value = false;
  editingFile.value = null;
  fileContent.value = "";
};
</script>
