<script setup lang="ts">
import { Toggle } from '@modrinth/ui'
import { useTheming } from '@/store/state'
import { ref, watch } from 'vue'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { DEFAULT_FEATURE_FLAGS, type FeatureFlag } from '@/store/theme.ts'

const themeStore = useTheming()

const settings = ref(await getSettings())
const options = ref<FeatureFlag[]>(Object.keys(DEFAULT_FEATURE_FLAGS))

function setFeatureFlag(key: string, value: boolean) {
  themeStore.featureFlags[key] = value
  settings.value.feature_flags[key] = value
}

watch(
  settings,
  async () => {
    await setSettings(settings.value)
  },
  { deep: true },
)
</script>
<template>
  <div v-for="option in options" :key="option" class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast capitalize">
        {{ option.replaceAll('_', ' ') }}
      </h2>
    </div>

    <Toggle
      id="advanced-rendering"
      :model-value="themeStore.getFeatureFlag(option)"
      @update:model-value="() => setFeatureFlag(option, !themeStore.getFeatureFlag(option))"
    />
  </div>
</template>
