<script setup lang="ts">
import { Toggle, ThemeSelector, TeleportDropdownMenu } from '@modrinth/ui'
import { useTheming } from '@/store/state'
import { get, set } from '@/helpers/settings'
import { watch, ref } from 'vue'
import { getOS } from '@/helpers/utils'

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
      (theme) => {
        themeStore.setThemeState(theme)
        settings.theme = theme
      }
    "
    :current-theme="settings.theme"
    :theme-options="themeStore.themeOptions"
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
      :checked="themeStore.advancedRendering"
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
    <Toggle
      id="native-decorations"
      :model-value="settings.native_decorations"
      :checked="settings.native_decorations"
      @update:model-value="
        (e) => {
          settings.native_decorations = e
        }
      "
    />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Minimize launcher</h2>
      <p class="m-0 mt-1">Minimize the launcher when a Minecraft process starts.</p>
    </div>
    <Toggle
      id="minimize-launcher"
      :model-value="settings.hide_on_process_start"
      :checked="settings.hide_on_process_start"
      @update:model-value="
        (e) => {
          settings.hide_on_process_start = e
        }
      "
    />
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
      :options="['Home', 'Library']"
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
      :checked="settings.toggle_sidebar"
      @update:model-value="
        (e) => {
          settings.toggle_sidebar = e
          themeStore.toggleSidebar = settings.toggle_sidebar
        }
      "
    />
  </div>
</template>
