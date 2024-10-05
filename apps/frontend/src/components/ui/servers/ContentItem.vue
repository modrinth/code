<template>
  <div
    class="group flex w-full items-center justify-between border-0 border-b border-solid border-bg-raised p-[0.7rem]"
    :class="data.disabled ? 'bg-bg-raised text-secondary' : 'hover:bg-button-bg'"
  >
    <div class="flex items-center gap-2">
      <UiAvatar
        :src="data.icon_url"
        no-shadow
        size="sm"
        alt="Server Icon"
        :class="data.disabled ? 'grayscale' : ''"
      />
      <div class="flex flex-col">
        <span
          class="flex items-center gap-1 text-lg font-bold"
          :class="data.disabled ? 'text-secondary' : 'text-contrast'"
        >
          {{ data.name === null ? "External Mod" : data.name }}
          <span
            v-if="data.disabled"
            class="rounded-full bg-button-bg p-1 px-2 text-xs text-contrast"
            >Disabled</span
          >
        </span>
        <span class="text-xs text-secondary group-hover:text-primary">{{
          data.version_number
        }}</span>
      </div>
    </div>
    <div
      class="flex gap-2 rounded-xl bg-bg-raised p-1 font-semibold text-contrast group-hover:bg-bg"
    >
      <Button icon-only transparent v-if="data.project_id" @click="emit('edit', data)">
        <EditIcon />
      </Button>
      <Button icon-only transparent @click="emit('toggle', data)">
        <ArchiveIcon />
      </Button>
      <Button icon-only transparent @click="emit('delete', data)">
        <TrashIcon />
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";
import { EditIcon, TrashIcon, ArchiveIcon } from "@modrinth/assets";

const serverStore = useServerStore();

const emit = defineEmits(["toggle", "delete", "edit"]);

const props = defineProps<{
  data: {
    name?: string;
    filename: string;
    project_id?: string;
    version_id?: string;
    version_number?: string;
    icon_url?: string;
    disabled: boolean;
  };
}>();
</script>
