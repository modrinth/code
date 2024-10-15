<template>
  <div
    class="relative flex w-full items-center justify-between rounded-xl bg-bg-raised"
    :class="data.disabled ? 'bg-table-alternateRow text-secondary' : ''"
  >
    <Checkbox v-model="selected" class="ml-2" :disabled="data.disabled" />
    <NuxtLink
      :to="`/project/${data.project_id}`"
      class="group flex w-full items-center rounded-xl p-2"
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
          <span class="flex items-center gap-2 text-lg font-bold">
            {{ data.name === null ? "External Mod" : data.name }}
            <span
              v-if="data.disabled"
              class="rounded-full bg-button-bg p-1 px-2 text-xs text-contrast"
            >
              Disabled
            </span>
          </span>
          <span v-if="data.name !== null" class="text-xs text-secondary">
            {{ data.version_number }}
          </span>
        </div>
      </div>
    </NuxtLink>
    <div class="absolute right-2 flex gap-2 rounded-xl p-1 font-semibold text-contrast">
      <Button v-if="data.project_id" icon-only transparent @click="emit('edit', data)">
        <EditIcon />
      </Button>
      <Button icon-only transparent @click="emit('delete', data)">
        <TrashIcon />
      </Button>
      <input
        id="property.id"
        class="switch stylized-toggle"
        type="checkbox"
        @change="emit('toggle', data)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, Checkbox } from "@modrinth/ui";
import { EditIcon, TrashIcon } from "@modrinth/assets";

const emit = defineEmits(["toggle", "delete", "edit"]);

const selected = ref(false);

defineProps<{
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
