<template>
  <div
    v-for="loader in loaders"
    :key="loader.name"
    class="group relative flex items-center justify-between rounded-2xl p-2 pr-2.5 hover:bg-bg"
  >
    <div class="flex items-center gap-4">
      <div
        class="grid size-10 place-content-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
        :class="isCurrentLoader(loader.name) ? '[&&]:bg-bg-green' : ''"
      >
        <UiServersIconsLoaderIcon
          :loader="loader.name"
          class="[&&]:size-6"
          :class="isCurrentLoader(loader.name) ? 'text-brand' : ''"
        />
      </div>
      <div class="flex flex-col gap-0.5">
        <div class="flex flex-row items-center gap-2">
          <h1 class="m-0 text-xl font-bold leading-none text-contrast">
            {{ loader.displayName }}
          </h1>
          <span
            v-if="isCurrentLoader(loader.name)"
            class="hidden items-center gap-1 rounded-full bg-bg-green p-1 px-1.5 text-xs font-semibold text-brand sm:flex"
          >
            <CheckIcon class="h-4 w-4" />
            Current
          </span>
        </div>
        <p v-if="isCurrentLoader(loader.name)" class="m-0 text-xs text-secondary">
          {{ data.loader_version }}
        </p>
      </div>
    </div>

    <ButtonStyled>
      <button @click="selectLoader(loader.name)">
        <DownloadIcon class="h-5 w-5" />
        {{ isCurrentLoader(loader.name) ? "Reinstall" : "Install" }}
      </button>
    </ButtonStyled>
  </div>
</template>

<script setup lang="ts">
import { CheckIcon, DownloadIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

const props = defineProps<{
  data: {
    loader: string | null;
    loader_version: string | null;
  };
}>();

const emit = defineEmits<{
  (e: "selectLoader", loader: string): void;
}>();

const loaders = [
  { name: "Vanilla" as const, displayName: "Vanilla" },
  { name: "Fabric" as const, displayName: "Fabric" },
  { name: "Quilt" as const, displayName: "Quilt" },
  { name: "Forge" as const, displayName: "Forge" },
  { name: "NeoForge" as const, displayName: "NeoForge" },
];

const isCurrentLoader = (loaderName: string) => {
  return props.data.loader?.toLowerCase() === loaderName.toLowerCase();
};

const selectLoader = (loader: string) => {
  emit("selectLoader", loader);
};
</script>
