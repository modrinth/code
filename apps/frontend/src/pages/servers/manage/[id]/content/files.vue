<template>
  <div class="flex w-full flex-col">
    <ContentItem
      v-for="mod in mods"
      :key="mod.name"
      :name="mod.name"
      :icon="mod.icon"
      :size="mod.size"
    />
  </div>
</template>

<script setup lang="ts">
import { FileIcon } from "@modrinth/assets";
import ContentItem from "~/components/ui/servers/ContentItem.vue";

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
