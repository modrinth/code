<template>
  <div ref="pyroFilesSentinel" class="sentinel" data-pyro-files-sentinel />
  <header
    :class="[
      'bg-table-alternate-row top-0 flex flex-col justify-between gap-2 p-3 transition-[border-radius] duration-20 select-none sm:h-12 sm:flex-row',
      !isStuck ? 'rounded-t-2xl' : 'sticky top-0 z-20',
    ]"
    data-pyro-files-state="browsing"
    aria-label="File navigation"
  >
    <nav
      aria-label="Breadcrumb navigation"
      class="text-contrast m-0 flex min-w-0 shrink items-center p-0"
    >
      <ol class="m-0 flex min-w-0 shrink list-none items-center p-0">
        <li class="-ml-1 shrink-0">
          <ButtonStyled type="transparent">
            <button
              v-tooltip="'Back to home'"
              type="button"
              class="focus-visible:outline-brand mr-2 grid h-12 w-10 place-content-center focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-solid"
              @click="$emit('navigate', -1)"
            >
              <span
                class="bg-button-bg group-hover:bg-highlight-brand group-hover:text-brand grid size-8 place-content-center rounded-full p-[6px]"
              >
                <HomeIcon class="h-5 w-5" />
                <span class="sr-only">Home</span>
              </span>
            </button>
          </ButtonStyled>
        </li>
        <li class="m-0 -ml-2 min-w-0 shrink p-0">
          <ol class="m-0 flex min-w-0 shrink items-center overflow-hidden p-0">
            <TransitionGroup
              name="breadcrumb"
              tag="span"
              class="relative flex min-w-0 shrink items-center"
            >
              <li
                v-for="(segment, index) in breadcrumbSegments"
                :key="`${segment || index}-group`"
                class="relative flex min-w-0 shrink items-center text-sm"
              >
                <div class="flex min-w-0 shrink items-center">
                  <ButtonStyled type="transparent">
                    <button
                      class="focus-visible:outline-brand cursor-pointer truncate focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-solid"
                      :aria-current="
                        index === breadcrumbSegments.length - 1 ? 'location' : undefined
                      "
                      :class="{
                        'text-contrast!': index === breadcrumbSegments.length - 1,
                      }"
                      @click="$emit('navigate', index)"
                    >
                      {{ segment || "" }}
                    </button>
                  </ButtonStyled>
                  <ChevronRightIcon
                    v-if="index < breadcrumbSegments.length - 1"
                    class="text-secondary size-4 shrink-0"
                    aria-hidden="true"
                  />
                </div>
              </li>
            </TransitionGroup>
          </ol>
        </li>
      </ol>
    </nav>

    <div class="flex shrink-0 items-center gap-1">
      <div class="flex w-full flex-row-reverse sm:flex-row">
        <ButtonStyled type="transparent">
          <UiServersTeleportOverflowMenu
            position="bottom"
            direction="left"
            aria-label="Filter view"
            :options="[
              { id: 'all', action: () => $emit('filter', 'all') },
              { id: 'filesOnly', action: () => $emit('filter', 'filesOnly') },
              { id: 'foldersOnly', action: () => $emit('filter', 'foldersOnly') },
            ]"
          >
            <div class="flex items-center gap-1">
              <FilterIcon aria-hidden="true" class="h-5 w-5" />
              <span class="hidden text-sm font-medium sm:block">
                {{ filterLabel }}
              </span>
            </div>
            <DropdownIcon aria-hidden="true" class="text-secondary h-5 w-5" />
            <template #all>Show all</template>
            <template #filesOnly>Files only</template>
            <template #foldersOnly>Folders only</template>
          </UiServersTeleportOverflowMenu>
        </ButtonStyled>
        <div class="mx-1 w-full text-sm sm:w-48">
          <label for="search-folder" class="sr-only">Search folder</label>
          <div class="relative">
            <SearchIcon
              class="pointer-events-none absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2"
              aria-hidden="true"
            />
            <input
              id="search-folder"
              :value="searchQuery"
              type="search"
              name="search"
              autocomplete="off"
              class="border-divider h-8 min-h-[unset] w-full border bg-transparent py-2 pl-9"
              placeholder="Search..."
              @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </div>
      </div>

      <ButtonStyled type="transparent">
        <OverflowMenu
          :dropdown-id="`create-new-${baseId}`"
          position="bottom"
          direction="left"
          aria-label="Create new..."
          :options="[
            { id: 'file', action: () => $emit('create', 'file') },
            { id: 'directory', action: () => $emit('create', 'directory') },
            { id: 'upload', action: () => $emit('upload') },
            { divider: true },
            { id: 'upload-zip', shown: false, action: () => $emit('upload-zip') },
            { id: 'install-from-url', action: () => $emit('unzip-from-url', false) },
            { id: 'install-cf-pack', action: () => $emit('unzip-from-url', true) },
          ]"
        >
          <PlusIcon aria-hidden="true" />
          <DropdownIcon aria-hidden="true" class="text-secondary h-5 w-5" />
          <template #file> <BoxIcon aria-hidden="true" /> New file </template>
          <template #directory> <FolderOpenIcon aria-hidden="true" /> New folder </template>
          <template #upload> <UploadIcon aria-hidden="true" /> Upload file </template>
          <template #upload-zip>
            <FileArchiveIcon aria-hidden="true" /> Upload from .zip file
          </template>
          <template #install-from-url>
            <LinkIcon aria-hidden="true" /> Upload from .zip URL
          </template>
          <template #install-cf-pack>
            <CurseForgeIcon aria-hidden="true" /> Install CurseForge pack
          </template>
        </OverflowMenu>
      </ButtonStyled>
    </div>
  </header>
</template>

<script setup lang="ts">
import {
  LinkIcon,
  CurseForgeIcon,
  FileArchiveIcon,
  BoxIcon,
  PlusIcon,
  UploadIcon,
  DropdownIcon,
  FolderOpenIcon,
  SearchIcon,
  HomeIcon,
  ChevronRightIcon,
  FilterIcon,
} from "@modrinth/assets";
import { ButtonStyled, OverflowMenu } from "@modrinth/ui";
import { ref, computed } from "vue";
import { useIntersectionObserver } from "@vueuse/core";

const props = defineProps<{
  breadcrumbSegments: string[];
  searchQuery: string;
  currentFilter: string;
  baseId: string;
}>();

defineEmits<{
  (e: "navigate", index: number): void;
  (e: "create", type: "file" | "directory"): void;
  (e: "upload" | "upload-zip"): void;
  (e: "unzip-from-url", cf: boolean): void;
  (e: "update:searchQuery", value: string): void;
  (e: "filter", type: string): void;
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

const filterLabel = computed(() => {
  switch (props.currentFilter) {
    case "filesOnly":
      return "Files only";
    case "foldersOnly":
      return "Folders only";
    default:
      return "Show all";
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

.breadcrumb-move,
.breadcrumb-enter-active,
.breadcrumb-leave-active {
  transition: all 0.2s ease;
}

.breadcrumb-enter-from {
  opacity: 0;
  transform: translateX(-10px) scale(0.9);
}

.breadcrumb-leave-to {
  opacity: 0;
  transform: translateX(-10px) scale(0.8);
  filter: blur(4px);
}

.breadcrumb-leave-active {
  position: relative;
  pointer-events: none;
}

.breadcrumb-move {
  z-index: 1;
}
</style>
