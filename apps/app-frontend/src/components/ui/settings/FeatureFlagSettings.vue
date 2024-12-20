<script setup lang="ts">
import { Toggle } from '@modrinth/ui'
import { useTheming } from '@/store/state'
import { computed } from 'vue'
import type { Ref } from 'vue'

const themeStore = useTheming()

type ThemeStoreKeys = keyof typeof themeStore

const options: Ref<ThemeStoreKeys[]> = computed(() => {
  return Object.keys(themeStore).filter((key) => key.startsWith('featureFlag_')) as ThemeStoreKeys[]
})

function getStoreValue<K extends ThemeStoreKeys>(key: K): (typeof themeStore)[K] {
  return themeStore[key]
}

function setStoreValue<K extends ThemeStoreKeys>(key: K, value: (typeof themeStore)[K]) {
  themeStore[key] = value
}

function formatFlagName(name: string) {
  return name.replace('featureFlag_', '')
}
</script>
<template>
  <div v-for="option in options" :key="option" class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast capitalize">
        {{ formatFlagName(option) }}
      </h2>
    </div>

    <Toggle
      id="advanced-rendering"
      :model-value="getStoreValue(option)"
      :checked="getStoreValue(option)"
      @update:model-value="() => setStoreValue(option, !themeStore[option])"
    />
  </div>
</template>
