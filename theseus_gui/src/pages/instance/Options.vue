<template>
  <Card class="settings-card">
    <h2 class="settings-title">Java</h2>
    <div class="settings-group">
      <h3>Installation</h3>
      <Checkbox v-model="overrideJavaInstall" label="Override global java installations" />
      <input
        ref="javaPath"
        v-model="javaPath"
        :disabled="!overrideJavaInstall"
        type="text"
        class="input installation-input"
        :placeholder="optimalJava ? optimalJava.path : 'Enter java path...'"
      />
      <span class="installation-buttons">
        <Button :disabled="!overrideJavaInstall" @click="saveJavaPath">
          <SearchIcon />
          Auto Detect
        </Button>
        <Button :disabled="!overrideJavaInstall" @click="saveJavaPath">
          <BrowseIcon />
          Browse
        </Button>
        <Button :disabled="!overrideJavaInstall" @click="saveJavaPath">
          <PlayIcon />
          Test
        </Button>
      </span>
    </div>
    <hr class="card-divider" />
    <div class="settings-group">
      <h3>Java Arguments</h3>
      <Checkbox v-model="overrideJavaArgs" label="Override global java arguments" />
      <input
        ref="javaArgs"
        v-model="javaArgs"
        :disabled="!overrideJavaArgs"
        type="text"
        class="input installation-input"
        :placeholder="globalSettings.custom_java_args ?? 'Enter java arguments...'"
      />
    </div>
    <div class="settings-group">
      <h3>Environment Variables</h3>
      <Checkbox v-model="overrideEnvVars" label="Override environment variables" />
      <input
        ref="javaArgs"
        v-model="javaArgs"
        :disabled="!overrideEnvVars"
        type="text"
        class="input installation-input"
        :placeholder="globalSettings.custom_env_args ?? 'Enter environment arguments...'"
      />
    </div>
    <hr class="card-divider" />
    <div class="settings-group">
      <Checkbox v-model="overrideMemorySettings" label="Override global memory settings" />
      <div class="sliders">
        <span class="slider">
          Minimum Memory
          <Slider
            v-model="globalSettings.memory.minimum"
            :disabled="!overrideMemorySettings"
            :min="256"
            :max="maxMemory"
            :step="10"
          />
        </span>
        <span class="slider">
          Maximum Memory
          <Slider
            v-model="globalSettings.memory.maximum"
            :disabled="!overrideMemorySettings"
            :min="256"
            :max="maxMemory"
            :step="10"
          />
        </span>
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title">Window</h2>
    <Checkbox v-model="overrideWindowSettings" label="Override global window settings" />
    <div class="settings-group">
      <div class="toggle-setting">
        Width
        <input
          :disabled="!overrideWindowSettings"
          type="number"
          class="input"
          :placeholder="globalSettings.game_resolution[0]"
        />
      </div>
      <div class="toggle-setting">
        Height
        <input
          :disabled="!overrideWindowSettings"
          type="number"
          class="input"
          :placeholder="globalSettings.game_resolution[1]"
        />
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title">Hooks</h2>
    <Checkbox v-model="overrideHooks" label="Override global hooks" />
    <div class="settings-group">
      <div class="toggle-setting">
        Pre Launch
        <input
          v-model="javaArgs"
          :disabled="!overrideHooks"
          type="text"
          class="input"
          :placeholder="globalSettings.hooks.pre_launch"
        />
      </div>
      <div class="toggle-setting">
        Wrapper
        <input
          v-model="javaArgs"
          :disabled="!overrideHooks"
          type="text"
          class="input"
          :placeholder="globalSettings.hooks.wrapper"
        />
      </div>
      <div class="toggle-setting">
        Post Exit
        <input
          v-model="javaArgs"
          :disabled="!overrideHooks"
          type="text"
          class="input"
          :placeholder="globalSettings.hooks.post_exit"
        />
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title">Profile Management</h2>
    <div class="settings-group">
      <div class="toggle-setting">
        Repair profile
        <button class="btn btn-highlight" :disabled="repairing" @click="repairProfile">
          <HammerIcon /> Repair
        </button>
      </div>
      <div class="toggle-setting">
        Delete profile
        <button class="btn btn-danger" :disabled="removing" @click="removeProfile">
          <TrashIcon /> Delete
        </button>
      </div>
    </div>
  </Card>
</template>

<script setup>
import { Card, Button, SearchIcon, Slider, TrashIcon, Checkbox } from 'omorphia'
import { BrowseIcon, PlayIcon } from '@/assets/icons'
import { HammerIcon } from '@/assets/icons'
import { useRouter } from 'vue-router'
import { get_optimal_jre_key, install, remove } from '@/helpers/profile.js'
import { ref } from 'vue'
import { get_max_memory } from '@/helpers/jre.js'
import { get } from '@/helpers/settings.js'

const router = useRouter()

const props = defineProps({
  instance: {
    type: Object,
    required: true,
  },
})

const globalSettings = await get()
const optimalJava = await get_optimal_jre_key(props.instance.path)
const maxMemory = ref((await get_max_memory()) / 1024)

const overrideJavaInstall = ref(false)
const overrideJavaArgs = ref(false)
const overrideEnvVars = ref(false)
const overrideMemorySettings = ref(false)
const overrideWindowSettings = ref(false)
const overrideHooks = ref(false)

const repairing = ref(false)
async function repairProfile() {
  repairing.value = true
  await install(props.instance.path)
  repairing.value = false
}

const removing = ref(false)
async function removeProfile() {
  removing.value = true
  await remove(props.instance.path)
  removing.value = false

  await router.push({ path: '/' })
}
</script>

<style scoped lang="scss">
.settings-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.settings-title {
  color: var(--color-contrast);
}

.settings-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.installation-input {
  width: 100%;
}

.installation-buttons {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  margin: 0;
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

.toggle-setting {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
}

.card-divider {
  background-color: var(--color-button-bg);
  border: none;
  color: var(--color-button-bg);
  height: 1px;
  margin: var(--gap-sm) 0;
}

:deep(button.checkbox) {
  border: none;
}
</style>
