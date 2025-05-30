<script setup lang="ts">
import { TeleportDropdownMenu, ThemeSelector, Toggle } from '@modrinth/ui'
import { useTheming } from '@/store/state'
import { get, set } from '@/helpers/settings.ts'
import { ref, watch } from 'vue'
import { getOS } from '@/helpers/utils'
import type { ColorTheme } from '@/store/theme.ts'

const themeStore = useTheming()

const os = ref(await getOS())
const settings = ref(await get())

watch(
  settings,
  async () => {
    await set(settings.value)
  },
  { deep: true },
)
</script>
<template>
  <h2 class="m-0 text-lg font-extrabold text-contrast">Color theme</h2>
  <p class="m-0 mt-1">Select your preferred color theme for Modrinth App.</p>

  <ThemeSelector
    :update-color-theme="
      (theme: ColorTheme) => {
        themeStore.setThemeState(theme)
        settings.theme = theme
      }
    "
    :current-theme="settings.theme"
    :theme-options="themeStore.getThemeOptions()"
    system-theme-color="system"
  />

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Advanced rendering</h2>
      <p class="m-0 mt-1">
        Enables advanced rendering such as blur effects that may cause performance issues without
        hardware-accelerated rendering.
      </p>
    </div>

    <Toggle
      id="advanced-rendering"
      :model-value="themeStore.advancedRendering"
      @update:model-value="
        (e) => {
          themeStore.advancedRendering = e
          settings.advanced_rendering = themeStore.advancedRendering
        }
      "
    />
  </div>

  <div v-if="os !== 'MacOS'" class="mt-4 flex items-center justify-between gap-4">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Native Decorations</h2>
      <p class="m-0 mt-1">Use system window frame (app restart required).</p>
    </div>
    <Toggle id="native-decorations" v-model="settings.native_decorations" />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Minimize launcher</h2>
      <p class="m-0 mt-1">Minimize the launcher when a Minecraft process starts.</p>
    </div>
    <Toggle id="minimize-launcher" v-model="settings.hide_on_process_start" />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Default landing page</h2>
      <p class="m-0 mt-1">Change the page to which the launcher opens on.</p>
    </div>
    <TeleportDropdownMenu
      id="opening-page"
      v-model="settings.default_page"
      name="Opening page dropdown"
      class="w-40"
      :options="['Home', 'Library']"
    />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Jump back into worlds</h2>
      <p class="m-0 mt-1">Includes recent worlds in the "Jump back in" section on the Home page.</p>
    </div>
    <Toggle
      :model-value="themeStore.getFeatureFlag('worlds_in_home')"
      @update:model-value="
        () => {
          const newValue = !themeStore.getFeatureFlag('worlds_in_home')
          themeStore.featureFlags['worlds_in_home'] = newValue
          settings.feature_flags['worlds_in_home'] = newValue
        }
      "
    />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Toggle sidebar</h2>
      <p class="m-0 mt-1">Enables the ability to toggle the sidebar.</p>
    </div>
    <Toggle
      id="toggle-sidebar"
      :model-value="settings.toggle_sidebar"
      @update:model-value="
        (e) => {
          settings.toggle_sidebar = e
          themeStore.toggleSidebar = settings.toggle_sidebar
        }
      "
    />
  </div>
</template>
