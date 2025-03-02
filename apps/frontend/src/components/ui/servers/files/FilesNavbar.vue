<template>
  <div ref="sentinel" class="sentinel" data-pyro-files-sentinel />
  <header
    :class="[
      'duration-20 top-0 flex select-none flex-col justify-between gap-2 bg-table-alternateRow p-3 transition-[border-radius] sm:h-12 sm:flex-row',
      !isStuck ? 'rounded-t-2xl' : 'sticky top-0 z-20',
    ]"
    :data-pyro-files-state="mode"
    :aria-label="mode === 'editing' ? 'File editor navigation' : 'File navigation'"
  >
    <nav
      aria-label="Breadcrumb navigation"
      class="m-0 flex min-w-0 flex-shrink items-center p-0 text-contrast"
    >
      <ol class="m-0 flex min-w-0 flex-shrink list-none items-center p-0">
        <li class="-ml-1 flex-shrink-0">
          <ButtonStyled type="transparent">
            <button
              v-tooltip="'Back to home'"
              type="button"
              class="mr-2 grid h-12 w-10 place-content-center focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
              @click="mode === 'editing' ? $emit('cancel') : $emit('navigate', -1)"
            >
              <span
                class="grid size-8 place-content-center rounded-full bg-button-bg p-[6px] group-hover:bg-brand-highlight group-hover:text-brand"
              >
                <HomeIcon class="h-5 w-5" />
                <span class="sr-only">Home</span>
              </span>
            </button>
          </ButtonStyled>
        </li>
        <li class="m-0 -ml-2 min-w-0 flex-shrink p-0">
          <ol class="m-0 flex min-w-0 flex-shrink items-center overflow-hidden p-0">
            <TransitionGroup
              name="breadcrumb"
              tag="span"
              class="relative flex min-w-0 flex-shrink items-center"
            >
              <li
                v-for="(segment, index) in breadcrumbSegments"
                :key="`${segment || index}-group`"
                class="relative flex min-w-0 flex-shrink items-center text-sm"
              >
                <div class="flex min-w-0 flex-shrink items-center">
                  <ButtonStyled type="transparent">
                    <button
                      class="cursor-pointer truncate focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
                      :aria-current="
                        index === breadcrumbSegments.length - 1 ? 'location' : undefined
                      "
                      :class="{
                        '!text-contrast': index === breadcrumbSegments.length - 1,
                      }"
                      @click="$emit('navigate', index)"
                    >
                      {{ segment || "" }}
                    </button>
                  </ButtonStyled>
                  <ChevronRightIcon
                    v-if="
                      index < breadcrumbSegments.length - 1 ||
                      mode === 'editing' ||
                      mode === 'imageview'
                    "
                    class="size-4 flex-shrink-0 text-secondary"
                    aria-hidden="true"
                  />
                </div>
              </li>
            </TransitionGroup>
            <li v-if="mode !== 'browsing'" class="flex items-center px-3 text-sm">
              <span class="font-semibold !text-contrast" aria-current="location">{{
                fileName
              }}</span>
            </li>
          </ol>
        </li>
      </ol>
    </nav>

    <div v-if="mode === 'editing' || mode === 'imageview'" class="flex items-center gap-2">
      <Button
        v-if="isLogFile"
        v-tooltip="'Share to mclo.gs'"
        icon-only
        transparent
        aria-label="Share to mclo.gs"
        @click="$emit('share')"
      >
        <ShareIcon />
      </Button>
      <ButtonStyled v-if="mode === 'editing'" type="transparent">
        <TeleportOverflowMenu
          position="bottom"
          direction="left"
          aria-label="Save file"
          :options="[
            { id: 'save', action: () => $emit('save') },
            { id: 'save-as', action: () => $emit('save-as') },
            { id: 'save&restart', action: () => $emit('save-restart') },
          ]"
        >
          <SaveIcon aria-hidden="true" />
          <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
          <template #save> <SaveIcon aria-hidden="true" /> Save </template>
          <template #save-as> <SaveIcon aria-hidden="true" /> Save as... </template>
          <template #save&restart>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
              aria-hidden="true"
            >
              <path
                fill-rule="evenodd"
                d="M15.312 11.424a5.5 5.5 0 0 1-9.201 2.466l-.312-.311h2.433a.75.75 0 0 0 0-1.5H3.989a.75.75 0 0 0-.75.75v4.242a.75.75 0 0 0 1.5 0v-2.43l.31.31a7 7 0 0 0 11.712-3.138.75.75 0 0 0-1.449-.39Zm1.23-3.723a.75.75 0 0 0 .219-.53V2.929a.75.75 0 0 0-1.5 0V5.36l-.31-.31A7 7 0 0 0 3.239 8.188a.75.75 0 1 0 1.448.389A5.5 5.5 0 0 1 13.89 6.11l.311.31h-2.432a.75.75 0 0 0 0 1.5h4.243a.75.75 0 0 0 .53-.219Z"
                clip-rule="evenodd"
              />
            </svg>
            Save & restart
          </template>
        </TeleportOverflowMenu>
      </ButtonStyled>
    </div>

    <div v-else class="flex flex-shrink-0 items-center gap-1">
      <div class="flex w-full flex-row-reverse sm:flex-row">
        <ButtonStyled type="transparent">
          <TeleportOverflowMenu
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
              <span class="hidden text-sm font-medium sm:block">{{ filterLabel }}</span>
            </div>
            <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
            <template #all>Show all</template>
            <template #filesOnly>Files only</template>
            <template #foldersOnly>Folders only</template>
          </TeleportOverflowMenu>
        </ButtonStyled>
        <div class="mx-1 w-full text-sm sm:w-48">
          <label for="search-folder" class="sr-only">Search folder</label>
          <div class="relative">
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
              class="h-8 min-h-[unset] w-full border-[1px] border-solid border-divider bg-transparent py-2 pl-9"
              placeholder="Search..."
              @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </div>
      </div>

      <ButtonStyled type="transparent">
        <TeleportOverflowMenu
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
        </TeleportOverflowMenu>
      </ButtonStyled>
    </div>
  </header>
</template>

<script setup lang="ts">
import {
  BoxIcon,
  PlusIcon,
  UploadIcon,
  DropdownIcon,
  FolderOpenIcon,
  SearchIcon,
  HomeIcon,
  ChevronRightIcon,
  FilterIcon,
  ShareIcon,
  SaveIcon,
} from "@modrinth/assets";
import { Button, ButtonStyled } from "@modrinth/ui";
import { ref, computed } from "vue";
import { useIntersectionObserver } from "@vueuse/core";
import TeleportOverflowMenu from "../TeleportOverflowMenu.vue";

const props = defineProps<{
  mode: "browsing" | "editing" | "imageview";
  breadcrumbSegments: string[];
  searchQuery?: string;
  currentFilter?: string;
  fileName?: string;
  filePath?: string;
}>();

defineEmits<{
  (e: "navigate", index: number): void;
  (e: "create", type: "file" | "directory"): void;
  (e: "upload"): void;
  (e: "update:searchQuery", value: string): void;
  (e: "filter", type: string): void;
  (e: "cancel"): void;
  (e: "save"): void;
  (e: "save-as"): void;
  (e: "save-restart"): void;
  (e: "share"): void;
}>();

const sentinel = ref<HTMLElement | null>(null);
const isStuck = ref(false);

useIntersectionObserver(
  sentinel,
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

const isLogFile = computed(() => {
  return props.filePath?.startsWith("logs") || props.filePath?.endsWith(".log");
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
