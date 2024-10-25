<template>
  <div v-for="loader in loaders" :key="loader.name" class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <div
        class="rounded-xl bg-button-bg p-2"
        :class="data.loader?.toLowerCase() === loader.name.toLowerCase() ? '[&&]:bg-bg-green' : ''"
      >
        <UiServersLoaderIcon
          :loader="loader.name"
          class="[&&]:size-10"
          :class="data.loader?.toLowerCase() === loader.name.toLowerCase() ? 'text-brand' : ''"
        />
      </div>
      <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">
        {{ loader.displayName }}
      </h1>
      <span
        v-if="data.loader?.toLowerCase() === loader.name.toLowerCase()"
        class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand"
      >
        Current
      </span>
    </div>

    <ButtonStyled>
      <button @click="selectLoader(loader.name)">
        {{ data.loader?.toLowerCase() === loader.name.toLowerCase() ? "Reinstall" : "Install" }}
      </button>
    </ButtonStyled>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";

defineProps<{
  data: {
    loader: string | null;
  };
}>();

const emit = defineEmits<{
  (e: "selectLoader", loader: string): void;
}>();

const loaders = [
  { name: "Vanilla", displayName: "Vanilla" },
  { name: "Fabric", displayName: "Fabric" },
  { name: "Quilt", displayName: "Quilt" },
  { name: "Forge", displayName: "Forge" },
  { name: "NeoForge", displayName: "NeoForge" },
];

const selectLoader = (loader: string) => {
  emit("selectLoader", loader);
};
</script>
