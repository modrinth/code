<template>
  <div ref="pyroFilesSentinel" class="sentinel" data-pyro-files-sentinel />
  <nav
    :class="[
      'top-0 flex h-24 select-none flex-col justify-between bg-table-alternateRow p-3 transition-[border-radius] duration-200 sm:h-12 sm:flex-row',
      !isStuck ? 'rounded-t-2xl' : 'sticky top-0 z-20',
    ]"
    data-pyro-files-state="browsing"
  >
    <ul class="m-0 flex list-none items-center p-0 text-contrast">
      <a
        v-tooltip="'Back to home'"
        role="link"
        class="group absolute left-0 top-0 grid h-12 w-14 place-content-center"
        @click="$emit('navigate', -1)"
      >
        <li
          class="grid size-8 place-content-center rounded-full bg-bg-raised p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
        >
          <HomeIcon class="h-5 w-5" />
        </li>
      </a>
      <UiServersIconsSlashIcon class="ml-8 h-5 w-5" />
      <li
        v-for="(segment, index) in breadcrumbSegments"
        :key="index"
        class="breadcrumb-link flex cursor-pointer items-center"
        @click="$emit('navigate', index)"
      >
        {{ segment || "" }}
        <UiServersIconsSlashIcon class="h-5 w-5" />
      </li>
    </ul>
    <div class="flex items-center gap-1">
      <ButtonStyled type="transparent">
        <UiServersTeleportOverflowMenu
          position="bottom"
          direction="left"
          aria-label="Sort files"
          :options="[
            { id: 'normal', action: () => $emit('sort', 'default') },
            { id: 'modified', action: () => $emit('sort', 'modified') },
            { id: 'filesOnly', action: () => $emit('sort', 'filesOnly') },
            { id: 'foldersOnly', action: () => $emit('sort', 'foldersOnly') },
          ]"
        >
          <span class="whitespace-pre text-sm font-medium">
            {{ sortMethodLabel }}
          </span>
          <SortAscendingIcon aria-hidden="true" />
          <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
          <template #normal> Alphabetical </template>
          <template #modified> Modified </template>
          <template #filesOnly> Files </template>
          <template #foldersOnly> Folders </template>
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
          :value="searchQuery"
          type="search"
          name="search"
          autocomplete="off"
          class="h-8 min-h-[unset] border-[1px] border-solid border-button-bg bg-transparent py-2 pl-9"
          placeholder="Search..."
          @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
        />
      </div>

      <ButtonStyled type="transparent">
        <UiServersTeleportOverflowMenu
          position="bottom"
          direction="left"
          aria-label="Create new..."
          :options="[
            { id: 'file', action: () => $emit('create', 'file') },
            { id: 'directory', action: () => $emit('create', 'directory') },
            { id: 'upload', action: () => $emit('upload') },
          ]"
        >
          <PlusIcon aria-hidden="true" />
          <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
          <template #file> <BoxIcon aria-hidden="true" /> New file </template>
          <template #directory> <FolderOpenIcon aria-hidden="true" /> New folder </template>
          <template #upload> <UploadIcon aria-hidden="true" /> Upload file </template>
        </UiServersTeleportOverflowMenu>
      </ButtonStyled>
    </div>
  </nav>
</template>

<script setup lang="ts">
import {
  BoxIcon,
  PlusIcon,
  UploadIcon,
  DropdownIcon,
  FolderOpenIcon,
  SearchIcon,
  SortAscendingIcon,
  HomeIcon,
} from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
import { ref } from "vue";
import { useIntersectionObserver } from "@vueuse/core";

const props = defineProps<{
  breadcrumbSegments: string[];
  searchQuery: string;
  sortMethod: string;
}>();

defineEmits<{
  (e: "navigate", index: number): void;
  (e: "sort", method: string): void;
  (e: "create", type: "file" | "directory"): void;
  (e: "upload"): void;
  (e: "update:searchQuery", value: string): void;
}>();

const pyroFilesSentinel = ref<HTMLElement | null>(null);
const isStuck = ref(false);

useIntersectionObserver(
  pyroFilesSentinel,
  ([{ isIntersecting }]) => {
    isStuck.value = !isIntersecting;
  },
  { threshold: [0, 1] },
);

const sortMethodLabel = computed(() => {
  switch (props.sortMethod) {
    case "modified":
      return "Modified";
    case "filesOnly":
      return "Files";
    case "foldersOnly":
      return "Folders";
    default:
      return "Alphabetical";
  }
});
</script>

<style scoped>
.sentinel {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  visibility: hidden;
}
</style>
