<script setup lang="ts">
import { PreviewSelectButton } from '@modrinth/ui'
import { useTheming } from '@/store/state'

const themeStore = useTheming()

const emit = defineEmits(['update:theme'])

function setTheme(theme: string) {
  emit('update:theme', theme);
}
</script>
<template>
  <h2 class="m-0 text-2xl">Color theme</h2>
  <p class="m-0 mt-1">Select your preferred color theme for Modrinth App.</p>

  <PreviewSelectButton v-for="(option, index) in themeStore.themeOptions" :key="`theme-${index}`" :checked="themeStore.selectedTheme === option" class="mt-4" @click="() => setTheme(option)">
    <template #preview> PREVIEW </template>
    {{ option }}
  </PreviewSelectButton>

  <h2 class="m-0 mt-4 text-2xl">Accent color</h2>
  <p class="m-0 mt-1">
    Thank you for supporting Modrinth! You can choose to make the Modrinth App accent color purple
    if you’d like.
  </p>

  <h2 class="m-0 mt-4 text-2xl">Feature flags</h2>
  <p class="m-0 mt-1">Enable or disable certain features.</p>
</template>
