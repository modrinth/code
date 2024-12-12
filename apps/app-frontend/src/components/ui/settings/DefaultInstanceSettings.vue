<script setup lang="ts">
import { get, set } from '@/helpers/settings'
import { ref, watch } from 'vue'
import { get_max_memory } from '@/helpers/jre'
import { handleError } from '@/store/notifications'
import { Slider, Toggle } from '@modrinth/ui'

const fetchSettings = await get()
fetchSettings.launchArgs = fetchSettings.extra_launch_args.join(' ')
fetchSettings.envVars = fetchSettings.custom_env_vars.map((x) => x.join('=')).join(' ')

const settings = ref(fetchSettings)

const maxMemory = ref(Math.floor((await get_max_memory().catch(handleError)) / 1024))

watch(
  settings,
  async () => {
    const setSettings = JSON.parse(JSON.stringify(settings.value))

    setSettings.extra_launch_args = setSettings.launchArgs.trim().split(/\s+/).filter(Boolean)
    setSettings.custom_env_vars = setSettings.envVars
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

    if (!setSettings.custom_dir) {
      setSettings.custom_dir = null
    }

    await set(setSettings)
  },
  { deep: true },
)
</script>

<template>
  <h2 class="m-0 text-2xl">Java arguments</h2>
  <input
    id="java-args"
    v-model="settings.launchArgs"
    autocomplete="off"
    type="text"
    class="installation-input"
    placeholder="Enter java arguments..."
  />

  <h2 class="mt-4 m-0 text-2xl">Environmental variables</h2>
  <input
    id="env-vars"
    v-model="settings.envVars"
    autocomplete="off"
    type="text"
    class="installation-input"
    placeholder="Enter environmental variables..."
  />

  <h2 class="mt-4 m-0 text-2xl">Java memory</h2>
  <p class="m-0 mt-1 leading-tight">The memory allocated to each instance when it is ran.</p>
  <Slider
    id="max-memory"
    v-model="settings.memory.maximum"
    :min="8"
    :max="maxMemory"
    :step="64"
    unit="MB"
  />

  <h2 class="mt-4 m-0 text-2xl">Hooks</h2>

  <h3 class="mt-2 m-0 text-lg">Pre launch</h3>
  <p class="m-0 mt-1 leading-tight">Ran before the instance is launched.</p>
  <input
    id="pre-launch"
    v-model="settings.hooks.pre_launch"
    autocomplete="off"
    type="text"
    placeholder="Enter pre-launch command..."
  />

  <h3 class="mt-2 m-0 text-lg">Wrapper</h3>
  <p class="m-0 mt-1 leading-tight">Wrapper command for launching Minecraft.</p>
  <input
    id="wrapper"
    v-model="settings.hooks.wrapper"
    autocomplete="off"
    type="text"
    placeholder="Enter wrapper command..."
  />

  <h3 class="mt-2 m-0 text-lg">Post exit</h3>
  <p class="m-0 mt-1 leading-tight">Ran after the game closes.</p>
  <input
    id="post-exit"
    v-model="settings.hooks.post_exit"
    autocomplete="off"
    type="text"
    placeholder="Enter post-exit command..."
  />

  <h2 class="mt-4 m-0 text-2xl">Window size</h2>

  <div class="flex items-center justify-between gap-4">
    <div>
      <h3 class="mt-2 m-0 text-lg">Fullscreen</h3>
      <p class="m-0 mt-1 leading-tight">
        Overwrites the options.txt file to start in full screen when launched.
      </p>
    </div>

    <Toggle
      id="fullscreen"
      :model-value="settings.force_fullscreen"
      :checked="settings.force_fullscreen"
      @update:model-value="
        (e) => {
          settings.force_fullscreen = e
        }
      "
    />
  </div>

  <div class="flex items-center justify-between gap-4">
    <div>
      <h3 class="mt-2 m-0 text-lg">Width</h3>
      <p class="m-0 mt-1 leading-tight">The width of the game window when launched.</p>
    </div>

    <input
      id="width"
      v-model="settings.game_resolution[0]"
      :disabled="settings.force_fullscreen"
      autocomplete="off"
      type="number"
      placeholder="Enter width..."
    />
  </div>

  <div class="flex items-center justify-between gap-4">
    <div>
      <h3 class="mt-2 m-0 text-lg">Height</h3>
      <p class="m-0 mt-1 leading-tight">The height of the game window when launched.</p>
    </div>

    <input
      id="height"
      v-model="settings.game_resolution[1]"
      :disabled="settings.force_fullscreen"
      autocomplete="off"
      type="number"
      class="input"
      placeholder="Enter height..."
    />
  </div>
</template>
