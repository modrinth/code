<template>
  <div>
    <div
      class="flex w-full items-center justify-between gap-2 rounded-xl bg-bg-raised p-2"
      @click="toggleExpanded"
    >
      <div class="flex items-center gap-2">
        <FolderOpenIcon />
        <div class="flex flex-col">
          <span class="text-lg font-bold group-hover:text-contrast">{{ props.path || "/" }}</span>
        </div>
      </div>
      <div class="flex gap-2">
        <Button icon-only transparent>
          <ChevronRightIcon />
        </Button>
      </div>
    </div>
    <div v-if="isExpanded">
      <div
        v-for="child in children"
        :key="child.name"
        class="flex w-full items-center justify-between gap-2 rounded-xl bg-bg-raised p-2"
        @click.stop="selectDirectory(child)"
      >
        <div class="flex items-center gap-2">
          <FolderOpenIcon />
          <div class="flex flex-col">
            <span class="text-lg font-bold group-hover:text-contrast">{{ child.name }}</span>
          </div>
        </div>
        <div class="flex gap-2">
          <Button icon-only transparent>
            <ChevronRightIcon />
          </Button>
        </div>
      </div>
      <FileTree
        v-for="child in children"
        :key="child.path"
        :path="child.path"
        :type="child.type"
        :items="child.items"
        @select="selectDirectory"
      />
      <div ref="scrollContainer" class="flex h-full w-full justify-center overflow-y-auto">
        <div class="flex h-full animate-pulse items-center gap-2" v-if="!isLoading">
          <PyroIcon class="h-4 w-4" /> Loading...
        </div>
        <div v-if="loadError">Error loading directories</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { FolderOpenIcon, ChevronRightIcon, PyroIcon } from "@modrinth/assets";
import { useInfiniteScroll } from "@vueuse/core";

const emit = defineEmits(["select"]);

const props = defineProps<{
  path: string;
  type: "file" | "directory";
  items: any[];
}>();

const expanded = ref(false);
const children = ref<any[]>(props.items || []);
const isLoading = ref(false);
const loadError = ref(false);

const route = useRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const scrollContainer = ref<HTMLElement | null>(null);
const currentPage = ref(1);
const pages = ref(1);
const maxResults = 100;

const { reset } = useInfiniteScroll(
  scrollContainer,
  () => {
    if (!isLoading.value && currentPage.value <= pages.value) {
      fetchChildren();
    }
  },
  { distance: 200 },
);

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
      children.value.push(...data.items);
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

const isExpanded = computed(() => expanded.value || children.value.length > 0);

const toggleExpanded = () => {
  expanded.value = !expanded.value;
  if (expanded.value && children.value.length === 0) {
    fetchChildren();
  }
};

const selectDirectory = (directory: any) => {
  if (directory.type === "directory") {
    emit("select", directory.path);
  }
};

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
