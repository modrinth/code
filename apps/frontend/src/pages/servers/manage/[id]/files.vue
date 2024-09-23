<template>
  <div>
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
          <input
            v-model="newItemName"
            type="text"
            class="bg-bg-input w-full rounded-lg p-4"
            :placeholder="`e.g. ${newItemType === 'file' ? 'config.yml' : 'plugins'}`"
          />
        </div>
        <div class="mb-4 mt-4 flex justify-end gap-4">
          <Button transparent @click="createItemModal?.hide()"> Cancel </Button>
          <Button color="primary" @click="createNewItem"> Create </Button>
        </div>
      </div>
    </Modal>

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
              class="flex h-9 w-9 items-center justify-center rounded-full bg-[#4d3737a4] p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
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
            placeholder="e.g. Before 1.21"
          />
        </div>
        <div class="mb-4 mt-4 flex justify-end gap-4">
          <Button transparent @click="renameItemModal?.hide()"> Cancel </Button>
          <Button color="primary" @click="renameItem"> Rename </Button>
        </div>
      </div>
    </Modal>

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
        </div>
        <div class="mb-4 mt-4 flex justify-end gap-4">
          <Button transparent @click="moveItemModal?.hide()"> Cancel </Button>
          <Button color="primary" @click="moveItem"> Move </Button>
        </div>
      </div>
    </Modal>

    <div class="flex h-[631px] w-full flex-col rounded-xl border border-solid border-bg-raised">
      <div
        class="flex items-center justify-between gap-2 rounded-t-xl bg-table-alternateRow px-4 py-2"
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
        <div class="flex items-center gap-2">
          <Button size="sm" icon-only @click="showCreateModal('file')">
            <PlusIcon />
          </Button>
          <Button size="sm" icon-only @click="showCreateModal('directory')">
            <PlusIcon />
          </Button>
          <Button size="sm" icon-only @click="showUploadModal">
            <UploadIcon />
          </Button>
        </div>
      </div>
      <div v-if="data && status == 'success'" class="overflow-y-contain overflow-x-none snap-y">
        <UiServersFileItem
          v-for="item in data.items"
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
        />
      </div>
      <div
        v-else-if="status === 'error'"
        class="flex h-full w-full items-center justify-center gap-6 p-20"
      >
        <FileIcon class="text-red-500 h-16 w-16" />
        <div class="flex flex-col gap-2">
          <h3 class="text-red-500 m-0 text-2xl font-bold">Invalid path</h3>
          <p class="m-0 text-sm text-secondary">The path provided is invalid or does not exist.</p>
        </div>
      </div>
      <UiServersPyroLoading v-else />
    </div>
    <div v-if="data && data.total >= 1" class="mt-4 flex w-full flex-col items-center">
      <Pagination
        :page="currentPage"
        :count="data.total"
        :link-function="(page) => `?page=${page}`"
        class="pagination-after"
        @switch-page="navigateToPage"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { BoxIcon, FileIcon, PlusIcon, UploadIcon, XIcon } from "@modrinth/assets";
import { Button, Pagination, Modal } from "@modrinth/ui";

const route = useNativeRoute();
const router = useRouter();
const serverId = route.params.id.toString();
const serverStore = useServerStore();

const maxResults = 10;
const currentPage = ref(1);
const currentPath = ref(route.query.path || "");

useHead({
  title: `Files - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

const auth = await serverStore.getFileApiInfo(serverId);

const { data, status } = await useLazyAsyncData("filesData", async () => {
  const data = await serverStore.listDirContents(
    await auth,
    Array.isArray(currentPath.value) ? currentPath.value.join("") : currentPath.value,
    currentPage.value,
    maxResults,
  );
  data.items.sort((a, b) => {
    if (a.type === "directory" && b.type !== "directory") return -1;
    if (a.type !== "directory" && b.type === "directory") return 1;
    if (a.count > b.count) return -1;
    if (a.count < b.count) return 1;
    if (a.name > b.name) return 1;
    if (a.name < b.name) return -1;
    return 0;
  });
  return data;
});

watch(
  () => route.query.path,
  async (newPath) => {
    currentPath.value = newPath || "/";
    await refreshNuxtData("filesData");
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
  currentPage.value = page;
  refreshNuxtData("filesData");
};

// Modal refs and data
const createItemModal = ref<typeof Modal>();
const deleteItemModal = ref<typeof Modal>();
const renameItemModal = ref<typeof Modal>();
const moveItemModal = ref<typeof Modal>();
const newItemType = ref<"file" | "directory">("file");
const newItemName = ref("");
const selectedItem = ref(null as any);
const destinationFolder = ref("");

const showCreateModal = (type: "file" | "directory") => {
  newItemType.value = type;
  newItemName.value = "";
  createItemModal.value?.show();
};

const createNewItem = async () => {
  try {
    const path =
      typeof currentPath.value === "string" ? currentPath.value : currentPath.value.join("");
    await serverStore.createFileOrFolder(auth, path, newItemName.value, newItemType.value);
    await refreshNuxtData("filesData");
    createItemModal.value?.hide();
  } catch (error) {
    console.error("Error creating item:", error);
  }
};

const showDeleteModal = (item: any) => {
  selectedItem.value = item;
  deleteItemModal.value?.show();
};

const deleteItem = async () => {
  try {
    await serverStore.deleteFileOrFolder(
      auth,
      `${currentPath.value}/${selectedItem.value.name}`,
      selectedItem.value.type === "directory",
    );
    await refreshNuxtData("filesData");
    deleteItemModal.value?.hide();
  } catch (error) {
    console.error("Error deleting item:", error);
  }
};

const downloadFile = async (item: any) => {
  if (item.type === "file") {
    try {
      const downloadUrl = await serverStore.downloadFile(auth, `${currentPath.value}/${item.name}`);
      if (typeof downloadUrl === "string" || downloadUrl instanceof URL) {
        window.open(downloadUrl.toString(), "_blank");
      } else {
        throw new TypeError("Invalid download URL");
      }
    } catch (error) {
      console.error("Error downloading file:", error);
    }
  }
};

const renameItem = async () => {
  try {
    await serverStore.renameFileOrFolder(
      auth,
      `${currentPath.value}/${selectedItem.value.name}`,
      newItemName.value,
    );
    await refreshNuxtData("filesData");
    renameItemModal.value?.hide();
  } catch (error) {
    console.error("Error renaming item:", error);
  }
};

const showUploadModal = () => {};

const showRenameModal = (item: any) => {
  selectedItem.value = item;
  renameItemModal.value?.show();
};

const showMoveModal = (item: any) => {
  selectedItem.value = item;
  destinationFolder.value = Array.isArray(currentPath.value)
    ? currentPath.value.join("")
    : currentPath.value;
  moveItemModal.value?.show();
};

const moveItem = async () => {
  if (!selectedItem.value || !destinationFolder.value) return;

  try {
    await serverStore.moveFileOrFolder(
      auth,
      `${currentPath.value}/${selectedItem.value.name}`,
      `${destinationFolder.value}/${selectedItem.value.name}`,
    );
    await refreshNuxtData("filesData");
    moveItemModal.value?.hide();
  } catch (error) {
    console.error("Error moving item:", error);
    // Handle error (show error message to user)
  }
};
</script>
