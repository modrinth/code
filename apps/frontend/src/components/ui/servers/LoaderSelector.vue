<template>
  <div class="flex w-full flex-col gap-1 rounded-2xl bg-table-alternateRow p-2">
    <div
      v-for="loader in vanillaLoaders"
      :key="loader.name"
      class="group relative flex items-center justify-between rounded-2xl p-2 pr-2.5 hover:bg-bg"
    >
      <UiServersLoaderSelectorCard
        :loader="loader"
        :is-current="isCurrentLoader(loader.name)"
        :loader-version="data.loader_version"
        :current-loader="data.loader"
        :is-installing="isInstalling"
        @select="selectLoader"
      />
    </div>
  </div>

  <div class="mt-4">
    <h2 class="mb-2 px-2 text-lg font-bold text-contrast">Mod loaders</h2>
    <div class="flex w-full flex-col gap-1 rounded-2xl bg-table-alternateRow p-2">
      <div
        v-for="loader in modLoaders"
        :key="loader.name"
        class="group relative flex items-center justify-between rounded-2xl p-2 pr-2.5 hover:bg-bg"
      >
        <UiServersLoaderSelectorCard
          :loader="loader"
          :is-current="isCurrentLoader(loader.name)"
          :loader-version="data.loader_version"
          :current-loader="data.loader"
          :is-installing="isInstalling"
          @select="selectLoader"
        />
      </div>
    </div>
  </div>

  <div class="mt-4">
    <h2 class="mb-2 px-2 text-lg font-bold text-contrast">Plugin loaders</h2>
    <div class="flex w-full flex-col gap-1 rounded-2xl bg-table-alternateRow p-2">
      <div
        v-for="loader in pluginLoaders"
        :key="loader.name"
        class="group relative flex items-center justify-between rounded-2xl p-2 pr-2.5 hover:bg-bg"
      >
        <UiServersLoaderSelectorCard
          :loader="loader"
          :is-current="isCurrentLoader(loader.name)"
          :loader-version="data.loader_version"
          :current-loader="data.loader"
          :is-installing="isInstalling"
          @select="selectLoader"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  data: {
    loader: string | null;
    loader_version: string | null;
  };
  isInstalling?: boolean;
}>();

const emit = defineEmits<{
  (e: "selectLoader", loader: string): void;
}>();

const vanillaLoaders = [{ name: "Vanilla" as const, displayName: "Vanilla" }];

const modLoaders = [
  { name: "Fabric" as const, displayName: "Fabric" },
  { name: "Quilt" as const, displayName: "Quilt" },
  { name: "Forge" as const, displayName: "Forge" },
  { name: "NeoForge" as const, displayName: "NeoForge" },
];

const pluginLoaders = [
  { name: "Paper" as const, displayName: "Paper" },
  { name: "Purpur" as const, displayName: "Purpur" },
];

const isCurrentLoader = (loaderName: string) => {
  return props.data.loader?.toLowerCase() === loaderName.toLowerCase();
};

const selectLoader = (loader: string) => {
  emit("selectLoader", loader);
};
</script>
