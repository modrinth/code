<template>
  <div class="h-[500px]">
    <div
      class="flex w-full cursor-pointer items-center justify-between gap-2 rounded-xl p-1 px-2 hover:bg-button-bg"
      @click="toggleExpanded"
    >
      <div class="flex items-center gap-2">
        <FolderOpenIcon />
        <div class="flex flex-col">
          <span class="text-lg font-bold group-hover:text-contrast">{{ props.path || "/" }}</span>
        </div>
      </div>
      <ChevronRightIcon
        :class="{ 'rotate-90 transform': isExpanded }"
        class="transition-transform duration-200"
      />
    </div>
    <div v-if="isExpanded" class="ml-4">
      <div v-for="child in children" :key="child.path" class="flex flex-col gap-1">
        <div
          class="flex w-full cursor-pointer items-center justify-between gap-2 rounded-xl bg-bg-raised p-1 px-2 hover:bg-button-bg"
          @click="toggleChild(child)"
        >
          <div class="flex items-center gap-2">
            <FolderOpenIcon v-if="child.type === 'directory'" />
            <FileIcon v-else />
            <div class="flex flex-col">
              <span class="text-lg font-bold group-hover:text-contrast">{{ child.name }}</span>
            </div>
          </div>
          <ChevronRightIcon
            v-if="child.type === 'directory'"
            :class="{ 'rotate-90 transform': child.isExpanded }"
            class="transition-transform duration-200"
          />
        </div>
        <!-- Recursively render children if expanded -->
        <div v-if="child.isExpanded" class="ml-4">
          <FileTree
            v-for="subChild in child.children"
            :key="subChild.path"
            :path="subChild.path"
            :type="subChild.type"
            :items="subChild.children"
            @select="emit('select', $event)"
          ></FileTree>
        </div>
      </div>
      <div ref="scrollContainer" class="flex h-full w-full justify-center overflow-y-auto">
        <div class="flex h-full animate-pulse items-center gap-2" v-if="isLoading">
          <PyroIcon class="h-4 w-4" /> Loading...
        </div>
        <div v-if="loadError">Error loading directories</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { FolderOpenIcon, ChevronRightIcon, PyroIcon, FileIcon } from "@modrinth/assets";
import { useInfiniteScroll } from "@vueuse/core";
import FileTree from "./FileTree.vue"; // Import the component itself for recursion

// Constants and variables
const emit = defineEmits(["select"]);
const props = defineProps<{
  path: string;
  type: "file" | "directory";
  items: any[];
}>();

const expanded = ref(false);
const children = ref<any[]>([]);
const isLoading = ref(false);
const loadError = ref(false);
const route = useRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();
const scrollContainer = ref<HTMLElement | null>(null);
const currentPage = ref(1);
const pages = ref(1);
const maxResults = 100;

// Infinite scroll setup
const { reset } = useInfiniteScroll(
  scrollContainer,
  () => {
    if (!isLoading.value && currentPage.value <= pages.value) {
      fetchChildren();
    }
  },
  { distance: 200 },
);

// Fetch children for the current path
const fetchChildren = async () => {
  isLoading.value = true;
  loadError.value = false;
  try {
    const data = await serverStore.listDirContents(
      serverId,
      props.path,
      currentPage.value,
      maxResults,
    );
    if (data && data.items) {
      children.value.push(
        ...data.items.map((item) => ({
          ...item,
          isExpanded: false,
          children: [],
        })),
      );
      pages.value = data.total;
      currentPage.value++;
    } else {
      throw new Error("Invalid data structure received from server.");
    }
  } catch (error) {
    console.error("Error fetching children:", error);
    loadError.value = true;
  } finally {
    isLoading.value = false;
  }
};

// Reactive properties
const isExpanded = computed(() => expanded.value);

// Toggle the expanded state of the root directory
const toggleExpanded = () => {
  expanded.value = !expanded.value;
  if (expanded.value && children.value.length === 0) {
    fetchChildren();
  }
};

// Toggle the expanded state of a child directory
const toggleChild = async (child: any) => {
  if (child.type === "directory") {
    child.isExpanded = !child.isExpanded;
    if (child.isExpanded && child.children.length === 0) {
      // Fetch children for this subdirectory
      isLoading.value = true;
      try {
        const data = await serverStore.listDirContents(serverId, child.path, 1, maxResults);
        if (data && data.items) {
          child.children = data.items.map((item) => ({
            ...item,
            isExpanded: false,
            children: [],
          }));
        }
      } catch (error) {
        console.error("Error fetching subdirectory:", error);
      } finally {
        isLoading.value = false;
      }
    }
    emit("select", child.path);
  }
};

// Watch for changes on the path prop and re-fetch children
watch(
  () => props.path,
  () => {
    children.value = [];
    expanded.value = false;
    currentPage.value = 1;
    fetchChildren();
  },
);
</script>
