<template>
  <div>
    <!-- Create Item Modal -->
    <Modal ref="createItemModal" header="">
      <div class="flex flex-col gap-4 p-6">
        <div class="flex items-center justify-between gap-4">
          <div class="flex items-center gap-4">
            <div class="text-2xl font-extrabold text-contrast">Create {{ newItemType }}</div>
          </div>
          <button
            class="h-8 w-8 rounded-full bg-[#ffffff10] p-2 text-contrast"
            @click="createItemModal?.hide()"
          >
            <XIcon class="h-4 w-4" />
          </button>
        </div>
        <div class="mt-2 flex flex-col gap-2">
          <div class="font-semibold text-contrast">Name<span class="text-red-500">*</span></div>
          <UiServersFileEditor
            v-model="newItemName"
            :placeholder="`e.g. ${newItemType === 'file' ? 'config.yml' : 'plugins'}`"
            @update:model-value="newItemName = $event"
          />
        </div>
        <div class="mb-4 mt-4 flex justify-end gap-4">
          <Button transparent @click="createItemModal?.hide()"> Cancel </Button>
          <Button color="primary" @click="createNewItem"> Create </Button>
        </div>
      </div>
    </Modal>

    <!-- Delete Item Modal -->
    <Modal ref="deleteItemModal" header="">
      <div
        class="flex flex-col gap-4 rounded-2xl border-2 border-solid border-[#FF496E] bg-[#270B11] p-6"
      >
        <div class="flex items-center justify-between gap-4">
          <div class="flex items-center gap-4">
            <div class="text-2xl font-extrabold text-contrast">Delete {{ selectedItem?.type }}</div>
          </div>
          <button
            class="h-8 w-8 rounded-full bg-[#ffffff10] p-2 text-contrast"
            @click="deleteItemModal?.hide()"
          >
            <XIcon class="h-4 w-4" />
          </button>
        </div>
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
      </div>
    </Modal>

    <!-- Rename Item Modal -->
    <Modal ref="renameItemModal" header="">
      <div class="flex flex-col gap-4 p-6">
        <div class="flex items-center justify-between gap-4">
          <div class="flex items-center gap-4">
            <div class="text-2xl font-extrabold text-contrast">Rename {{ selectedItem?.type }}</div>
          </div>
          <button
            class="h-8 w-8 rounded-full bg-[#ffffff10] p-2 text-contrast"
            @click="renameItemModal?.hide()"
          >
            <XIcon class="h-4 w-4" />
          </button>
        </div>
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
    </Modal>

    <!-- Move Item Modal -->
    <Modal ref="moveItemModal" header="">
      <div class="flex flex-col gap-4 p-6">
        <div class="flex items-center justify-between gap-4">
          <div class="flex items-center gap-4">
            <div class="text-2xl font-extrabold text-contrast">Move {{ selectedItem?.name }}</div>
          </div>
          <button
            class="h-8 w-8 rounded-full bg-[#ffffff10] p-2 text-contrast"
            @click="moveItemModal?.hide()"
          >
            <XIcon class="h-4 w-4" />
          </button>
        </div>
        <div class="mt-2 flex flex-col gap-2">
          <div class="font-semibold text-contrast">Destination Folder</div>
          <input
            v-model="destinationFolder"
            type="text"
            class="bg-bg-input w-full rounded-lg p-4"
            placeholder="e.g. /path/to/destination"
          />
        </div>
        <div class="mb-4 mt-4 flex justify-end gap-4">
          <Button transparent @click="moveItemModal?.hide()"> Cancel </Button>
          <Button color="primary" @click="moveItem"> Move </Button>
        </div>
      </div>
    </Modal>

    <!-- Main Content -->
    <div class="flex h-[631px] w-full flex-col rounded-xl border border-solid border-bg-raised">
      <div
        class="flex h-12 items-center justify-between gap-2 rounded-t-xl bg-table-alternateRow px-4 py-2"
        v-if="!isEditing"
      >
        <div class="flex items-center gap-2 text-contrast">
          <span
            class="breadcrumb-link flex cursor-pointer items-center gap-2"
            @click="navigateToSegment(-1)"
          >
            <BoxIcon class="h-6 w-6 text-brand" />
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
              { id: 'upload', action: () => showUploadModal() },
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
          <span
            class="breadcrumb-link flex cursor-pointer items-center gap-2"
            @click="navigateToSegment(-1)"
          >
            <BoxIcon class="h-6 w-6 text-brand" />
            <span class="text-lg font-bold">{{ editingFile?.name }}</span>
          </span>
        </div>

        <ButtonStyled type="transparent">
          <Button @click="cancelEditing">
            <XIcon aria-hidden="true" />
          </Button>
        </ButtonStyled>
      </div>

      <div v-if="isEditing" class="flex-grow overflow-hidden">
        <textarea
          v-model="fileContent"
          class="h-full w-full resize-none rounded-t-none bg-bg-raised p-4 font-mono text-sm"
        ></textarea>
      </div>
      <div
        v-else-if="items.length > 0"
        class="snap-y overflow-y-auto overflow-x-hidden"
        ref="scrollContainer"
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
          :auth="auth"
          @delete="showDeleteModal(item)"
          @rename="showRenameModal(item)"
          @download="downloadFile(item)"
          @move="showMoveModal(item)"
          @edit="editFile(item)"
        />
        <div v-if="!isLoading && items.length < totalItems" ref="sentinel"></div>
      </div>

      <div v-else-if="!auth" class="flex h-full w-full items-center justify-center gap-6 p-20">
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
} from "@modrinth/assets";
import { Button, Modal, ButtonStyled, OverflowMenu } from "@modrinth/ui";
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";

const route = useRoute();
const router = useRouter();
const serverId = route.params.id.toString();
const serverStore = useServerStore();

const maxResults = 20;
const currentPage = ref(1);
const currentPath = ref(route.query.path || "");

useHead({
  title: `Files - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

const auth = ref<any>(null);
const fetchAuth = async () => {
  try {
    const apiInfo = await serverStore.getFileApiInfo(serverId);
    auth.value = apiInfo;
  } catch (error) {
    console.error("Error fetching file api info:", error);
    auth.value = null;
  }
};

// Reactive list of items
const items = ref<any[]>([]);

// Track total items if available
const totalItems = ref(0);

// Loading state
const isLoading = ref(false);

// Error state
const loadError = ref(false);

// References to DOM elements
const sentinel = ref<HTMLElement | null>(null);
const scrollContainer = ref<HTMLElement | null>(null);

// Intersection Observer
let observer: IntersectionObserver | null = null;

const fetchData = async () => {
  if (isLoading.value || (totalItems.value && items.value.length >= totalItems.value)) {
    return;
  }

  isLoading.value = true;
  loadError.value = false;

  try {
    const path = Array.isArray(currentPath.value) ? currentPath.value.join("") : currentPath.value;
    const data = await serverStore.listDirContents(auth.value, path, currentPage.value, maxResults);

    if (!data || !data.items) {
      throw new Error("Invalid data structure received from server.");
    }

    data.items.sort((a, b) => {
      if (a.type === "directory" && b.type !== "directory") return -1;
      if (a.type !== "directory" && b.type === "directory") return 1;
      if (a.count > b.count) return -1;
      if (a.count < b.count) return 1;
      if (a.name > b.name) return 1;
      if (a.name < b.name) return -1;
      return 0;
    });

    items.value.push(...data.items);
    totalItems.value = data.total;

    currentPage.value++;
  } catch (error) {
    console.error("Error fetching data:", error);
    loadError.value = true;
  } finally {
    isLoading.value = false;
  }
};

const setupObserver = () => {
  if (observer) {
    observer.disconnect();
  }

  observer = new IntersectionObserver(
    (entries) => {
      if (entries[0].isIntersecting) {
        fetchData();
      }
    },
    {
      root: scrollContainer.value,
      rootMargin: "0px",
      threshold: 1.0,
    },
  );

  if (sentinel.value) {
    observer.observe(sentinel.value);
  }
};

onMounted(async () => {
  await fetchAuth();
  if (auth.value) {
    await fetchData();
    setupObserver();
  }
});

onBeforeUnmount(() => {
  if (observer) {
    observer.disconnect();
  }
});

watch(
  () => route.query.path,
  async (newPath) => {
    console.log(`Path changed to: ${newPath}`);
    currentPath.value = newPath || "/";
    currentPage.value = 1;
    items.value = [];
    totalItems.value = 0;
    loadError.value = false;
    if (auth.value) {
      await fetchData();
      setupObserver();
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
  router.push({ query: { page: page, path: currentPath.value } });
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

const showUploadModal = () => {
  // Implement upload functionality or show another modal
  console.warn("Upload functionality is not implemented yet.");
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
    const path =
      typeof currentPath.value === "string" ? currentPath.value : currentPath.value.join("/");
    await serverStore.createFileOrFolder(auth.value, path, newItemName.value, newItemType.value);

    currentPage.value = 1;
    items.value = [];
    totalItems.value = 0;
    await fetchData();
    setupObserver();
    createItemModal.value?.hide();
    console.log("Created new item and refreshed data.");
  } catch (error) {
    console.error("Error creating item:", error);
  }
};

const renameItem = async () => {
  try {
    await serverStore.renameFileOrFolder(
      auth.value,
      `${currentPath.value}/${selectedItem.value.name}`,
      newItemName.value,
    );

    currentPage.value = 1;
    items.value = [];
    totalItems.value = 0;
    await fetchData();
    setupObserver();
    renameItemModal.value?.hide();
    console.log("Renamed item and refreshed data.");
  } catch (error) {
    console.error("Error renaming item:", error);
  }
};

const moveItem = async () => {
  if (!selectedItem.value || !destinationFolder.value) return;

  try {
    await serverStore.moveFileOrFolder(
      auth.value,
      `${currentPath.value}/${selectedItem.value.name}`,
      `${destinationFolder.value}/${selectedItem.value.name}`,
    );

    currentPage.value = 1;
    items.value = [];
    totalItems.value = 0;
    await fetchData();
    setupObserver();
    moveItemModal.value?.hide();
    console.log("Moved item and refreshed data.");
  } catch (error) {
    console.error("Error moving item:", error);
  }
};

const downloadFile = async (item: any) => {
  if (item.type === "file") {
    try {
      const path = `${currentPath.value}/${item.name}`.replace("//", "/");
      const fileData = await serverStore.downloadFile(auth.value, path);
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
      auth.value,
      `${currentPath.value}/${selectedItem.value.name}`,
      selectedItem.value.type === "directory",
    );
    currentPage.value = 1;
    items.value = [];
    totalItems.value = 0;
    await fetchData();
    setupObserver();
    deleteItemModal.value?.hide();
    console.log("Deleted item and refreshed data.");
  } catch (error) {
    console.error("Error deleting item:", error);
  }
};

const editFile = async (item: { name: string; type: string; path: string }) => {
  try {
    const path = `${currentPath.value}/${item.name}`.replace("//", "/");
    const content = (await serverStore.downloadFile(auth.value, path)) as string;

    fileContent.value = content;
    editingFile.value = item;
    isEditing.value = true;
    console.log(item);
  } catch (error) {
    console.error("Error fetching file content:", error);
  }
};

const saveFileContent = async () => {
  if (!editingFile.value) return;

  try {
    await refreshNuxtData("filesData");
    isEditing.value = false;
    editingFile.value = null;
  } catch (error) {
    console.error("Error saving file content:", error);
  }
};

const cancelEditing = () => {
  isEditing.value = false;
  editingFile.value = null;
  fileContent.value = "";
};
</script>
