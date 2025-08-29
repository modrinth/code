<template>
  <div class="flex w-full items-center justify-between">
    <div class="flex items-center gap-4">
      <div
        class="border-button-border bg-button-bg grid size-10 place-content-center rounded-xl border shadow-xs"
        :class="isCurrentLoader ? '[&&]:bg-bg-green' : ''"
      >
        <UiServersIconsLoaderIcon
          :loader="loader.name"
          class="[&&]:size-6"
          :class="isCurrentLoader ? 'text-brand' : ''"
        />
      </div>
      <div class="flex flex-col gap-0.5">
        <div class="flex flex-row items-center gap-2">
          <h1 class="text-contrast m-0 text-xl leading-none font-bold">
            {{ loader.displayName }}
          </h1>
          <span
            v-if="isCurrentLoader"
            class="bg-bg-green text-brand hidden items-center gap-1 rounded-full p-1 px-1.5 text-xs font-semibold sm:flex"
          >
            <CheckIcon class="h-4 w-4" />
            Current
          </span>
        </div>
        <p v-if="isCurrentLoader" class="text-secondary m-0 text-xs">
          {{ loaderVersion }}
        </p>
      </div>
    </div>

    <ButtonStyled>
      <button :disabled="isInstalling" @click="onSelect">
        <DownloadIcon class="h-5 w-5" />
        {{ isCurrentLoader ? "Reinstall" : "Install" }}
      </button>
    </ButtonStyled>
  </div>
</template>

<script setup lang="ts">
import { CheckIcon, DownloadIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

interface LoaderInfo {
  name: "Vanilla" | "Fabric" | "Forge" | "Quilt" | "Paper" | "NeoForge" | "Purpur";
  displayName: string;
}

interface Props {
  loader: LoaderInfo;
  currentLoader: string | null;
  loaderVersion: string | null;
  isInstalling?: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "select", loader: string): void;
}>();

const isCurrentLoader = computed(() => {
  return props.currentLoader?.toLowerCase() === props.loader.name.toLowerCase();
});

const onSelect = () => {
  emit("select", props.loader.name);
};
</script>
