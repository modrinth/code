<template>
  <div
    :class="
      `group flex w-full items-center justify-between border-0 border-b border-solid border-bg-raised p-[0.7rem] hover:bg-bg-raised` +
      (disabled ? ' cursor-not-allowed' : '')
    "
  >
    <div class="flex items-center gap-2">
      <UiAvatar :src="icon_url" no-shadow size="sm" alt="Server Icon" />
      <div class="flex flex-col">
        <span class="text-lg font-bold group-hover:text-contrast">{{
          name === null ? "External Mod" : name
        }}</span>
        <span class="text-xs text-secondary group-hover:text-primary">{{ version_number }}</span>
      </div>
    </div>
    <div
      class="flex gap-2 rounded-xl bg-bg-raised p-1 font-semibold text-contrast group-hover:bg-bg"
    >
      <Button icon-only transparent v-if="project">
        <EditIcon />
      </Button>
      <Button icon-only transparent>
        <ArchiveIcon />
      </Button>
      <Button icon-only transparent>
        <TrashIcon />
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";
import { EditIcon, TrashIcon, ArchiveIcon } from "@modrinth/assets";

const serverStore = useServerStore();

const props = defineProps<{
  name: string;
  filename: string;
  project: string;
  version: string;
  disabled: boolean;
  icon_url: string;
  version_number: string;
}>();

const disableMod = () => {
  //serverStore.disableMod(props.project, props.version);
};

const deleteMod = () => {
  //serverStore.removeMod(props.project, props.version);
};
</script>
