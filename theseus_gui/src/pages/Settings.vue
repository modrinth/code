<script setup>
import { ref, watch } from 'vue'
import { Card, Slider, DropdownSelect, Toggle } from 'omorphia'
import { handleError, useTheming } from '@/store/state'
import { get, set } from '@/helpers/settings'
import { get_max_memory } from '@/helpers/jre'
import JavaSelector from '@/components/ui/JavaSelector.vue'

const themeStore = useTheming()

const fetchSettings = await get().catch(handleError)

if (!fetchSettings.java_globals.JAVA_8)
  fetchSettings.java_globals.JAVA_8 = { path: '', version: '' }
if (!fetchSettings.java_globals.JAVA_17)
  fetchSettings.java_globals.JAVA_17 = { path: '', version: '' }

fetchSettings.javaArgs = fetchSettings.custom_java_args.join(' ')
fetchSettings.envArgs = fetchSettings.custom_env_args.map((x) => x.join('=')).join(' ')

const settings = ref(fetchSettings)
const maxMemory = ref((await get_max_memory().catch(handleError)) / 1024)

watch(settings.value, async (oldSettings, newSettings) => {
  const setSettings = JSON.parse(JSON.stringify(newSettings))

  if (setSettings.java_globals.JAVA_8?.path === '') {
    setSettings.java_globals.JAVA_8 = undefined
  }
  if (setSettings.java_globals.JAVA_17?.path === '') {
    setSettings.java_globals.JAVA_17 = undefined
  }

  setSettings.custom_java_args = setSettings.javaArgs.trim().split(/\s+/).filter(Boolean)
  setSettings.custom_env_args = setSettings.envArgs
    .trim()
    .split(/\s+/)
    .filter(Boolean)
    .map((x) => x.split('=').filter(Boolean))

  if (!setSettings.hooks.pre_launch) {
    setSettings.hooks.pre_launch = null
  }
  if (!setSettings.hooks.wrapper) {
    setSettings.hooks.wrapper = null
  }
  if (!setSettings.hooks.post_exit) {
    setSettings.hooks.post_exit = null
  }

  await set(setSettings)
})
</script>

<template>
  <div>
    <Card class="theming">
      <h2>Display</h2>
      <div class="toggle-setting">
        <div class="description">
          <h3>Color theme</h3>
          <p>Change the global launcher color theme.</p>
        </div>
        <DropdownSelect
          name="Theme dropdown"
          :options="themeStore.themeOptions"
          :default-value="settings.theme"
          :model-value="settings.theme"
          class="theme-dropdown"
          @change="
            (e) => {
              themeStore.setThemeState(e.option.toLowerCase())
              settings.theme = themeStore.selectedTheme
            }
          "
        />
      </div>
      <div class="toggle-setting">
        <div class="description">
          <h3>Collapsed navigation mode</h3>
          <p>Change the style of the side navigation bar</p>
        </div>
        <Toggle
          :model-value="themeStore.collapsedNavigation"
          :checked="themeStore.collapsedNavigation"
          @update:model-value="
            (e) => {
              themeStore.collapsedNavigation = e
              settings.collapsed_navigation = themeStore.collapsedNavigation
            }
          "
        />
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Launcher settings</h2>
      <div class="settings-group">
        <h3>Resource management</h3>
        <div class="toggle-setting">
          <span>Maximum concurrent downloads</span>
          <Slider
            v-model="settings.max_concurrent_downloads"
            class="concurrent-downloads"
            :min="1"
            :max="100"
            :step="1"
          />
        </div>
        <div class="toggle-setting">
          <span>Maximum concurrent writes</span>
          <Slider
            v-model="settings.max_concurrent_writes"
            class="concurrent-downloads"
            :min="1"
            :max="100"
            :step="1"
          />
        </div>
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Java</h2>
      <div class="settings-group">
        <h3>Java 17 location</h3>
        <JavaSelector v-model="settings.java_globals.JAVA_17" :version="17" />
      </div>
      <div class="settings-group">
        <h3>Java 8 location</h3>
        <JavaSelector v-model="settings.java_globals.JAVA_8" :version="8" />
      </div>
      <hr class="card-divider" />
      <div class="settings-group">
        <h3>Java arguments</h3>
        <input
          v-model="settings.javaArgs"
          type="text"
          class="input installation-input"
          placeholder="Enter java arguments..."
        />
      </div>
      <div class="settings-group">
        <h3>Environment variables</h3>
        <input
          v-model="settings.envArgs"
          type="text"
          class="input installation-input"
          placeholder="Enter environment variables..."
        />
      </div>
      <hr class="card-divider" />
      <div class="settings-group">
        <div class="sliders">
          <span class="slider">
            Minimum memory
            <Slider v-model="settings.memory.minimum" :min="256" :max="maxMemory" :step="10" />
          </span>
          <span class="slider">
            Maximum memory
            <Slider v-model="settings.memory.maximum" :min="256" :max="maxMemory" :step="10" />
          </span>
        </div>
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Hooks</h2>
      <div class="settings-group">
        <div class="toggle-setting">
          Pre launch
          <input v-model="settings.hooks.pre_launch" type="text" class="input" />
        </div>
        <div class="toggle-setting">
          Wrapper
          <input v-model="settings.hooks.wrapper" type="text" class="input" />
        </div>
        <div class="toggle-setting">
          Post exit
          <input v-model="settings.hooks.post_exit" type="text" class="input" />
        </div>
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Window Size</h2>
      <div class="settings-group">
        <div class="toggle-setting">
          Width
          <input v-model="settings.game_resolution[0]" type="number" class="input" />
        </div>
        <div class="toggle-setting">
          Height
          <input v-model="settings.game_resolution[1]" type="number" class="input" />
        </div>
      </div>
    </Card>
  </div>
</template>

<style lang="scss" scoped>
.concurrent-downloads {
  width: 80% !important;
}

.slider-input {
  width: 5rem !important;
  flex-basis: 5rem !important;
}

.installation-input {
  width: 100% !important;
  flex-grow: 1;
}

.theming,
.settings-card {
  margin: 1rem;
}

.theming {
  .toggle-setting {
    display: flex;
  }
}

.theme-dropdown {
  text-transform: capitalize;
}

.settings-card {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.settings-title {
  color: var(--color-contrast);
}

.settings-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.toggle-setting {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
}

.sliders {
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  gap: 1rem;
  width: 100%;

  .slider {
    flex-grow: 1;
  }
}
</style>
