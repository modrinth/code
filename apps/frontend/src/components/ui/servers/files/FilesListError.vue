<template>
  <div class="flex h-full w-full items-center justify-center gap-6 p-20">
    <FileIcon class="size-28" />
    <div class="flex flex-col gap-2">
      <h3 class="text-red-500 m-0 text-2xl font-bold">{{ title }}</h3>
      <p class="m-0 text-sm text-secondary">
        {{ message }}
      </p>
      <div class="flex gap-2">
        <ButtonStyled>
          <button size="sm" @click="$emit('refetch')">
            <UiServersIconsLoadingIcon class="h-5 w-5" />
            Try again
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button size="sm" @click="$emit('home')">
            <HomeIcon class="h-5 w-5" />
            Go to home folder
          </button>
        </ButtonStyled>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { FileIcon, HomeIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

defineProps<{
  title: string;
  message: string;
}>();

defineEmits<{
  (e: "refetch"): void;
  (e: "home"): void;
}>();
</script>
