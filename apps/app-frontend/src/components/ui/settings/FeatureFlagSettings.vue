<script setup lang="ts">
import { Toggle } from '@modrinth/ui'
import { useTheming } from '@/store/state'
import { ref, watch } from 'vue'
import { get, set } from '@/helpers/settings'

const themeStore = useTheming()

const settings = ref(await get())
const options = ref(['project_background', 'page_path'])

function getStoreValue(key: string) {
  return themeStore.featureFlags[key] ?? false
}

function setStoreValue(key: string, value: boolean) {
  themeStore.featureFlags[key] = value
  settings.value.feature_flags[key] = value
}

watch(
  settings,
  async () => {
    await set(settings.value)
  },
  { deep: true },
)
</script>
<template>
  <div v-for="option in options" :key="option" class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast capitalize">
        {{ option }}
      </h2>
    </div>

    <Toggle
      id="advanced-rendering"
      :model-value="getStoreValue(option)"
      :checked="getStoreValue(option)"
      @update:model-value="() => setStoreValue(option, !themeStore.featureFlags[option])"
    />
  </div>
</template>
