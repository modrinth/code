<script setup>
import { ref, watch } from 'vue'
import { Card, Slider, DropdownSelect, Toggle } from 'omorphia'
import { handleError, useTheming } from '@/store/state'
import { get, set } from '@/helpers/settings'
import { get_max_memory } from '@/helpers/jre'
import JavaSelector from '@/components/ui/JavaSelector.vue'
import mixpanel from 'mixpanel-browser'

const pageOptions = ['Home', 'Library']

const themeStore = useTheming()

const fetchSettings = await get().catch(handleError)

if (!fetchSettings.java_globals.JAVA_8)
  fetchSettings.java_globals.JAVA_8 = { path: '', version: '' }
if (!fetchSettings.java_globals.JAVA_17)
  fetchSettings.java_globals.JAVA_17 = { path: '', version: '' }

fetchSettings.javaArgs = fetchSettings.custom_java_args.join(' ')
fetchSettings.envArgs = fetchSettings.custom_env_args.map((x) => x.join('=')).join(' ')

const settings = ref(fetchSettings)
const maxMemory = ref(Math.floor((await get_max_memory().catch(handleError)) / 1024))

watch(
  settings,
  async (oldSettings, newSettings) => {
    const setSettings = JSON.parse(JSON.stringify(newSettings))

    if (setSettings.opt_out_analytics) {
      mixpanel.opt_out_tracking()
    } else {
      mixpanel.opt_in_tracking()
    }

    if (setSettings.java_globals.JAVA_8?.path === '') {
      setSettings.java_globals.JAVA_8 = undefined
    }
    if (setSettings.java_globals.JAVA_17?.path === '') {
      setSettings.java_globals.JAVA_17 = undefined
    }

    if (setSettings.java_globals.JAVA_8?.path) {
      setSettings.java_globals.JAVA_8.path = setSettings.java_globals.JAVA_8.path.replace(
        'java.exe',
        'javaw.exe'
      )
    }
    if (setSettings.java_globals.JAVA_17?.path) {
      setSettings.java_globals.JAVA_17.path = setSettings.java_globals.JAVA_17?.path.replace(
        'java.exe',
        'javaw.exe'
      )
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
  },
  { deep: true }
)
</script>

<template>
  <div class="settings-page">
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Display</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="theme">
          <span class="label__title">Color theme</span>
          <span class="label__description">Change the global launcher color theme.</span>
        </label>
        <DropdownSelect
          id="theme"
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
      <div class="adjacent-input">
        <label for="advanced-rendering">
          <span class="label__title">Advanced rendering</span>
          <span class="label__description">
            Enables advanced rendering such as blur effects that may cause performance issues
            without hardware-accelerated rendering.
          </span>
        </label>
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
      <div class="adjacent-input">
        <label for="minimize-launcher">
          <span class="label__title">Minimize launcher</span>
          <span class="label__description"
            >Minimize the launcher when a Minecraft process starts.</span
          >
        </label>
        <Toggle
          id="minimize-launcher"
          :model-value="settings.hide_on_process"
          :checked="settings.hide_on_process"
          @update:model-value="
            (e) => {
              settings.hide_on_process = e
            }
          "
        />
      </div>
      <div class="opening-page">
        <label for="opening-page">
          <span class="label__title">Default landing page</span>
          <span class="label__description">Change the page to which the launcher opens on.</span>
        </label>
        <DropdownSelect
          id="opening-page"
          name="Opening page dropdown"
          :options="pageOptions"
          :default-value="settings.default_page"
          :model-value="settings.default_page"
          class="opening-page"
          @change="
            (e) => {
              settings.default_page = e.option
            }
          "
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Resource management</span>
        </h3>
      </div>

      <div class="adjacent-input">
        <label for="max-downloads">
          <span class="label__title">Maximum concurrent downloads</span>
          <span class="label__description"
            >The maximum amount of files the launcher can download at the same time. Set this to a
            lower value if you have a poor internet connection.</span
          >
        </label>
        <Slider
          id="max-downloads"
          v-model="settings.max_concurrent_downloads"
          :min="1"
          :max="10"
          :step="1"
        />
      </div>

      <div class="adjacent-input">
        <label for="max-writes">
          <span class="label__title">Maximum concurrent writes</span>
          <span class="label__description"
            >The maximum amount of files the launcher can write to the disk at once. Set this to a
            lower value if you are frequently getting I/O errors.</span
          >
        </label>
        <Slider
          id="max-writes"
          v-model="settings.max_concurrent_writes"
          :min="1"
          :max="50"
          :step="1"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Privacy</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="opt-out-analytics">
          <span class="label__title">Disable analytics</span>
          <span class="label__description">
            Modrinth collects anonymized analytics and usage data to improve our user experience and
            customize your experience. Opting out will disable this data collection.
          </span>
        </label>
        <Toggle id="opt-out-analytics" v-model="settings.opt_out_analytics" />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Java settings</span>
        </h3>
      </div>
      <label for="java-17">
        <span class="label__title">Java 17 location</span>
      </label>
      <JavaSelector id="java-17" v-model="settings.java_globals.JAVA_17" :version="17" />
      <label for="java-8">
        <span class="label__title">Java 8 location</span>
      </label>
      <JavaSelector id="java-8" v-model="settings.java_globals.JAVA_8" :version="8" />
      <hr class="card-divider" />
      <label for="java-args">
        <span class="label__title">Java arguments</span>
      </label>
      <input
        id="java-args"
        v-model="settings.javaArgs"
        autocomplete="off"
        type="text"
        class="installation-input"
        placeholder="Enter java arguments..."
      />
      <label for="env-vars">
        <span class="label__title">Environmental variables</span>
      </label>
      <input
        id="env-vars"
        v-model="settings.envArgs"
        autocomplete="off"
        type="text"
        class="installation-input"
        placeholder="Enter environmental variables..."
      />
      <hr class="card-divider" />
      <div class="adjacent-input">
        <label for="max-memory">
          <span class="label__title">Java memory</span>
          <span class="label__description">
            The memory allocated to each instance when it is ran.
          </span>
        </label>
        <Slider
          id="max-memory"
          v-model="settings.memory.maximum"
          :min="256"
          :max="maxMemory"
          :step="1"
          unit="mb"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Hooks</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="pre-launch">
          <span class="label__title">Pre launch</span>
          <span class="label__description"> Ran before the instance is launched. </span>
        </label>
        <input
          id="pre-launch"
          v-model="settings.hooks.pre_launch"
          autocomplete="off"
          type="text"
          placeholder="Enter pre-launch command..."
        />
      </div>
      <div class="adjacent-input">
        <label for="wrapper">
          <span class="label__title">Wrapper</span>
          <span class="label__description"> Wrapper command for launching Minecraft. </span>
        </label>
        <input
          id="wrapper"
          v-model="settings.hooks.wrapper"
          autocomplete="off"
          type="text"
          placeholder="Enter wrapper command..."
        />
      </div>
      <div class="adjacent-input">
        <label for="post-exit">
          <span class="label__title">Post exit</span>
          <span class="label__description"> Ran after the game closes. </span>
        </label>
        <input
          id="post-exit"
          v-model="settings.hooks.post_exit"
          autocomplete="off"
          type="text"
          placeholder="Enter post-exit command..."
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Window size</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="width">
          <span class="label__title">Width</span>
          <span class="label__description"> The width of the game window when launched. </span>
        </label>
        <input
          id="width"
          v-model="settings.game_resolution[0]"
          autocomplete="off"
          type="number"
          placeholder="Enter width..."
        />
      </div>
      <div class="adjacent-input">
        <label for="height">
          <span class="label__title">Height</span>
          <span class="label__description"> The height of the game window when launched. </span>
        </label>
        <input
          id="height"
          v-model="settings.game_resolution[1]"
          autocomplete="off"
          type="number"
          class="input"
          placeholder="Enter height..."
        />
      </div>
    </Card>
  </div>
</template>

<style lang="scss" scoped>
.settings-page {
  margin: 1rem;
}

.installation-input {
  width: 100% !important;
  flex-grow: 1;
}

.theme-dropdown {
  text-transform: capitalize;
}

.card-divider {
  margin: 1rem 0;
}
</style>
