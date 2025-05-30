<script setup lang="ts">
import { get, set } from '@/helpers/settings.ts'
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
  <div>
    <h2 class="m-0 text-lg font-extrabold text-contrast">Window size</h2>

    <div class="flex items-center justify-between gap-4">
      <div>
        <h3 class="mt-2 m-0 text-base font-extrabold text-primary">Fullscreen</h3>
        <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
          Overwrites the options.txt file to start in full screen when launched.
        </p>
      </div>

      <Toggle id="fullscreen" v-model="settings.force_fullscreen" />
    </div>

    <div class="flex items-center justify-between gap-4">
      <div>
        <h3 class="mt-2 m-0 text-base font-extrabold text-primary">Width</h3>
        <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
          The width of the game window when launched.
        </p>
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
        <h3 class="mt-2 m-0 text-base font-extrabold text-primary">Height</h3>
        <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
          The height of the game window when launched.
        </p>
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

    <hr class="mt-4 bg-button-border border-none h-[1px]" />

    <h2 class="mt-4 m-0 text-lg font-extrabold text-contrast">Memory allocated</h2>
    <p class="m-0 mt-1 leading-tight">The memory allocated to each instance when it is ran.</p>
    <Slider
      id="max-memory"
      v-model="settings.memory.maximum"
      :min="512"
      :max="maxMemory"
      :step="64"
      unit="MB"
    />

    <h2 class="mt-4 mb-2 text-lg font-extrabold text-contrast">Java arguments</h2>
    <input
      id="java-args"
      v-model="settings.launchArgs"
      autocomplete="off"
      type="text"
      placeholder="Enter java arguments..."
      class="w-full"
    />

    <h2 class="mt-4 mb-2 text-lg font-extrabold text-contrast">Environmental variables</h2>
    <input
      id="env-vars"
      v-model="settings.envVars"
      autocomplete="off"
      type="text"
      placeholder="Enter environmental variables..."
      class="w-full"
    />

    <hr class="mt-4 bg-button-border border-none h-[1px]" />

    <h2 class="mt-4 m-0 text-lg font-extrabold text-contrast">Hooks</h2>

    <h3 class="mt-2 m-0 text-base font-extrabold text-primary">Pre launch</h3>
    <p class="m-0 mt-1 mb-2 leading-tight text-secondary">Ran before the instance is launched.</p>
    <input
      id="pre-launch"
      v-model="settings.hooks.pre_launch"
      autocomplete="off"
      type="text"
      placeholder="Enter pre-launch command..."
      class="w-full"
    />

    <h3 class="mt-2 m-0 text-base font-extrabold text-primary">Wrapper</h3>
    <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
      Wrapper command for launching Minecraft.
    </p>
    <input
      id="wrapper"
      v-model="settings.hooks.wrapper"
      autocomplete="off"
      type="text"
      placeholder="Enter wrapper command..."
      class="w-full"
    />

    <h3 class="mt-2 m-0 text-base font-extrabold text-primary">Post exit</h3>
    <p class="m-0 mt-1 mb-2 leading-tight text-secondary">Ran after the game closes.</p>
    <input
      id="post-exit"
      v-model="settings.hooks.post_exit"
      autocomplete="off"
      type="text"
      placeholder="Enter post-exit command..."
      class="w-full"
    />
  </div>
</template>
