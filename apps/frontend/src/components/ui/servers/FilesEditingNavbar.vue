<template>
  <header
    data-pyro-files-state="editing"
    class="flex h-12 select-none items-center justify-between rounded-t-2xl bg-table-alternateRow p-3"
    aria-label="File editor navigation"
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
              @click="goHome"
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
        <li class="m-0 -ml-2 p-0">
          <ol class="m-0 flex items-center p-0">
            <li
              v-for="(segment, index) in breadcrumbSegments"
              :key="index"
              class="flex items-center text-sm"
            >
              <ButtonStyled type="transparent">
                <button
                  class="cursor-pointer focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
                  :class="{ '!text-contrast': index === breadcrumbSegments.length - 1 }"
                  @click="$emit('navigate', index)"
                >
                  {{ segment || "" }}
                </button>
              </ButtonStyled>
              <ChevronRightIcon
                v-if="index < breadcrumbSegments.length"
                class="size-4 text-secondary"
                aria-hidden="true"
              />
            </li>
            <li class="flex items-center px-3 text-sm">
              <span class="font-semibold !text-contrast" aria-current="location">{{
                fileName
              }}</span>
            </li>
          </ol>
        </li>
      </ol>
    </nav>
    <div v-if="!isImage" class="flex gap-2">
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
      <ButtonStyled type="transparent">
        <UiServersTeleportOverflowMenu
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
        </UiServersTeleportOverflowMenu>
      </ButtonStyled>
    </div>
  </header>
</template>

<script setup lang="ts">
import { DropdownIcon, SaveIcon, ShareIcon, HomeIcon, ChevronRightIcon } from "@modrinth/assets";
import { Button, ButtonStyled } from "@modrinth/ui";
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";

const props = defineProps<{
  breadcrumbSegments: string[];
  fileName?: string;
  isImage: boolean;
  filePath?: string;
}>();

const isLogFile = computed(() => {
  return props.filePath?.startsWith("logs") || props.filePath?.endsWith(".log");
});

const route = useRoute();
const router = useRouter();

const emit = defineEmits<{
  (e: "cancel"): void;
  (e: "save"): void;
  (e: "save-as"): void;
  (e: "save-restart"): void;
  (e: "share"): void;
  (e: "navigate", index: number): void;
}>();

const goHome = () => {
  emit("cancel");
  router.push({ path: "/servers/manage/" + route.params.id + "/files" });
};
</script>
