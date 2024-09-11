<template>
  <div class="flex w-full flex-col rounded-xl border border-solid border-bg-raised">
    <div
      class="flex items-center justify-between gap-2 rounded-t-xl bg-table-alternateRow px-4 py-2"
    >
      <div class="flex items-center gap-2 text-contrast">
        <BoxIcon class="h-6 w-6 text-brand" />
        / config / jei / folder
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
    <UiServersContentItem
      v-for="mod in mods"
      :key="mod.name"
      :name="mod.name"
      :icon="mod.icon"
      :size="mod.size"
    />
  </div>
</template>

<script setup lang="ts">
import { BoxIcon, FileIcon, PlusIcon, UploadIcon } from "@modrinth/assets";
import { Button } from "@modrinth/ui";

const route = useNativeRoute();
const serverId = route.params.id.toString();
const serverStore = useServerStore();

useHead({
  title: `Files - ${serverStore.serverData[serverId]?.name ?? "Server"} - Modrinth`,
});

const mods = [
  {
    name: "Fabric API",
    icon: FileIcon,
    version: "0.44.1",
    size: 1.42,
  },
  {
    name: "Modrinth API",
    icon: FileIcon,
    version: "0.1.0",
    size: 0.69,
  },
  {
    name: "Modrinth API",
    icon: FileIcon,
    version: "0.1.0",
    size: 0.69,
  },
  {
    name: "Modrinth API",
    icon: FileIcon,
    version: "0.1.0",
    size: 0.69,
  },
  {
    name: "Modrinth API",
    icon: FileIcon,
    version: "0.1.0",
    size: 0.69,
  },
  {
    name: "Modrinth API",
    icon: FileIcon,
    version: "0.1.0",
    size: 0.69,
  },
];
</script>
