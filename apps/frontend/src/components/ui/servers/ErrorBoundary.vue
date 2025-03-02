<template>
  <div class="flex w-full flex-col items-center justify-center gap-4 p-4">
    <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
      <div class="flex flex-col items-center text-center">
        <div class="flex flex-col items-center gap-4">
          <div class="grid place-content-center rounded-full bg-bg-orange p-4">
            <IssuesIcon class="size-12 text-orange" />
          </div>
          <h1 class="m-0 mb-2 w-fit text-4xl font-bold">{{ title }}</h1>
        </div>
        <p class="text-lg text-secondary">{{ message }}</p>
        <p v-if="error">
          <span class="break-all font-mono text-xs leading-[190%]">{{
            JSON.stringify(error)
          }}</span>
        </p>
        <ButtonStyled size="large" color="brand" @click="$emit('retry')">
          <button class="mt-6 !w-full">Retry</button>
        </ButtonStyled>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { IssuesIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

defineProps<{
  title?: string;
  message?: string;
  error?: unknown;
}>();

defineEmits<{
  retry: void;
}>();
</script>
