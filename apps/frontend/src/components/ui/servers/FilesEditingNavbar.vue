<template>
  <nav
    data-pyro-files-state="editing"
    class="flex h-12 select-none items-center justify-between rounded-t-2xl bg-table-alternateRow p-3"
  >
    <div class="flex items-center gap-2 text-contrast">
      <ButtonStyled type="transparent">
        <button @click="$emit('cancel')">
          <LeftArrowIcon class="h-5 w-5" />
        </button>
      </ButtonStyled>

      <div class="">{{ fileName }}</div>
    </div>

    <div v-if="!isImage" class="flex gap-2">
      <Button
        v-if="isLogFile"
        v-tooltip="'Share to mclo.gs'"
        icon-only
        transparent
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
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
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
  </nav>
</template>

<script setup lang="ts">
import { DropdownIcon, SaveIcon, ShareIcon, LeftArrowIcon } from "@modrinth/assets";
import { Button, ButtonStyled } from "@modrinth/ui";

const props = defineProps<{
  fileName?: string;
  isImage: boolean;
  filePath?: string;
}>();

const isLogFile = computed(() => {
  return props.filePath?.startsWith("logs") || props.filePath?.endsWith(".log");
});

defineEmits<{
  (e: "cancel"): void;
  (e: "save"): void;
  (e: "save-as"): void;
  (e: "save-restart"): void;
  (e: "share"): void;
}>();
</script>
