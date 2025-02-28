<script setup lang="ts">
import { ButtonStyled, Toggle } from '@modrinth/ui'
import { useTheming } from '@/store/state'
import { ref, type Ref, watch } from 'vue'
import { get, set } from '@/helpers/settings'
import { DEFAULT_FEATURE_FLAGS } from '@/store/theme'

type FeatureFlag = keyof typeof DEFAULT_FEATURE_FLAGS

const themeStore = useTheming()

const settings = ref(await get())
const options: Ref<FeatureFlag[]> = ref(Object.keys(DEFAULT_FEATURE_FLAGS) as FeatureFlag[])

function getStoreValue(key: FeatureFlag) {
  return themeStore.featureFlags[key] ?? false
}

function setStoreValue(key: FeatureFlag, value: boolean) {
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
  <h2 class="m-0 text-lg font-extrabold text-contrast">
    Feature flags
  </h2>
  <p class="mt-1 mb-0 leading-tight text-secondary">
    These are developer tools that are not intended to be used by end users except for debugging purposes.
  </p>
  <p class="my-3 font-bold">Do not report bugs or issues if you have any feature flags enabled.</p>
  <div v-for="option in options" :key="option" class="mt-2 px-4 py-3 flex items-center justify-between bg-bg rounded-2xl">
    <div>
      <h2 class="m-0 text-base font-bold text-primary capitalize">
        {{ option.replace(new RegExp('_', "g"), ' ') }}
      </h2>
      <p class="m-0 text-sm text-secondary">Default: {{ DEFAULT_FEATURE_FLAGS[option] }}</p>
    </div>

    <div class="flex items-center gap-1">
      <ButtonStyled type="transparent">
        <button class="text-sm" :disabled="getStoreValue(option) === DEFAULT_FEATURE_FLAGS[option]" @click="() => setStoreValue(option, !themeStore.featureFlags[option])">
          Reset to default
        </button>
      </ButtonStyled>
      <Toggle
        id="advanced-rendering"
        :model-value="getStoreValue(option)"

        @update:model-value="() => setStoreValue(option, !themeStore.featureFlags[option])"
      />
    </div>
  </div>
</template>
