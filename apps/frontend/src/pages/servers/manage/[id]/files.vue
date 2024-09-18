<template>
  <div
    class="flex w-full flex-col rounded-xl border border-solid border-bg-raised"
    v-if="data && status == 'success'"
  >
    <div
      class="flex items-center justify-between gap-2 rounded-t-xl bg-table-alternateRow px-4 py-2"
    >
      <div class="flex items-center gap-2 text-contrast">
        <span @click="navigateToSegment(-1)" class="breadcrumb-link flex items-center gap-2">
          <BoxIcon class="h-6 w-6 text-brand" />
          /
        </span>
        <span
          v-for="(segment, index) in breadcrumbSegments"
          :key="index"
          @click="navigateToSegment(index)"
          class="breadcrumb-link"
        >
          {{ segment || "" }} /
        </span>
      </div>
      <div class="flex items-center gap-2">
        <Button size="sm" icon-only>
          <PlusIcon />
        </Button>
        <Button size="sm" icon-only>
          <UploadIcon />
        </Button>
      </div>
    </div>
    <UiServersFileItem
      v-for="item in data.items"
      :key="item.name"
      :count="item.count"
      :created="item.created"
      :modifoed="item.modified"
      :name="item.name"
      :path="item.path"
      :type="item.type"
      :size="item.size"
    />
  </div>
  <UiServersPyroLoading v-else />
</template>

<script setup lang="ts">
import { BoxIcon, FileIcon, PlusIcon, UploadIcon } from "@modrinth/assets";
import { Button, Pagination } from "@modrinth/ui";

const route = useNativeRoute();
const router = useRouter();
const serverId = route.params.id.toString();
const serverStore = useServerStore();

const maxResults = 2;
const currentPage = ref(0);
const currentPath = ref(route.query.path || "/");

useHead({
  title: `Files - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

const auth = await serverStore.getFileApiInfo(serverId);

const { data, status } = await useAsyncData("filesData", async () => {
  const data = await serverStore.listDirContents(
    await auth,
    currentPath.value,
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

const breadcrumbSegments = computed(() => currentPath.value.split("/").filter(Boolean));

const navigateToSegment = (index: number) => {
  const newPath = breadcrumbSegments.value.slice(0, index + 1).join("/");
  router.push({ query: { path: newPath } });
};
</script>
