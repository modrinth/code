<script setup>
import { ref, watch } from 'vue'
import { Card, Slider, DropdownSelect, Button, SearchIcon, PlayIcon } from 'omorphia'
import { BrowseIcon } from '@/assets/icons'
import { useTheming } from '@/store/state'
import { get, set, deepEqual } from '@/helpers/settings'

// Original object to track changes
const originalSettings = ref(await get())
// Object to bind
const settings = ref(await get())

const saveButton = ref(null)

watch(settings.value, (newSettings) => {
  // Validate the changed state
  if (newSettings.custom_java_args.length === 0) settings.value.custom_java_args = []
  if (newSettings.custom_env_args.length === 0) settings.value.custom_env_args = []
  if (newSettings.java_8_path === '') settings.value.java_8_path = null
  if (newSettings.java_17_path === '') settings.value.java_17_path = null
  if (newSettings.hooks.pre_launch === '') delete settings.value.hooks.pre_launch
  if (newSettings.hooks.wrapper === '') delete settings.value.hooks.wrapper
  if (newSettings.hooks.post_exit === '') delete settings.value.hooks.post_exit

  settings.value.max_concurrent_downloads = parseInt(newSettings.max_concurrent_downloads)

  if (deepEqual(originalSettings.value, settings.value)) saveButton.value.$el.style.opacity = 0
  else saveButton.value.$el.style.opacity = 1
})

const themeStore = useTheming()

const handleTheme = (e) => themeStore.setThemeState(e.option.toLowerCase())
const saveJavaPath = () => {}
const saveSettings = async () => {
  await set(settings.value)
  saveButton.value.$el.style.opacity = 0
}
</script>

<template>
  <div>
    <Button ref="saveButton" color="primary" class="save-btn" @click="saveSettings">Save</Button>
    <Card class="theming">
      <h2>Themes</h2>
      <div class="setting-row">
        <div class="description">
          <h3>Color theme</h3>
          <p>Change the global launcher color theme.</p>
        </div>
        <DropdownSelect
          name="Theme dropdown"
          :options="themeStore.themeOptions"
          :default-value="themeStore.selectedTheme"
          :model-value="themeStore.selectedTheme"
          class="theme-dropdown"
          @change="handleTheme"
        />
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Java</h2>
      <div class="settings-group">
        <h3>Java 17 Location</h3>
        <div class="toggle-setting">
          <input
            v-model="settings.java_17_path"
            type="text"
            class="input installation-input"
            placeholder="/path/to/java17"
          />
          <span class="installation-buttons">
            <Button @click="saveJavaPath">
              <SearchIcon />
              Auto Detect
            </Button>
            <Button @click="saveJavaPath">
              <BrowseIcon />
              Browse
            </Button>
            <Button @click="saveJavaPath">
              <PlayIcon />
              Test
            </Button>
          </span>
        </div>
      </div>
      <div class="settings-group">
        <h3>Java 8 Location</h3>
        <div class="toggle-setting">
          <input
            v-model="settings.java_8_path"
            type="text"
            class="input installation-input"
            placeholder="/path/to/java8"
          />
          <span class="installation-buttons">
            <Button @click="saveJavaPath">
              <SearchIcon />
              Auto Detect
            </Button>
            <Button @click="saveJavaPath">
              <BrowseIcon />
              Browse
            </Button>
            <Button @click="saveJavaPath">
              <PlayIcon />
              Test
            </Button>
          </span>
        </div>
      </div>
      <hr class="card-divider" />
      <div class="toggle-setting">
        <h3>Java Arguments</h3>
        <input v-model="settings.custom_java_args" type="text" class="input installation-input" />
      </div>
      <div class="toggle-setting">
        <h3>Environment Arguments</h3>
        <input v-model="settings.custom_env_args" type="text" class="input installation-input" />
      </div>
      <hr class="card-divider" />
      <div class="settings-group">
        <div class="sliders">
          <span class="slider">
            Minimum Memory
            <Slider v-model="settings.memory.minimum" :min="1024" :max="8192" :step="1024" />
          </span>
          <span class="slider">
            Maximum Memory
            <Slider v-model="settings.memory.maximum" :min="1024" :max="8192" :step="1024" />
          </span>
        </div>
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Window Size</h2>
      <div class="settings-group">
        <div class="settings-group">
          <div class="sliders">
            <span class="slider">
              Width
              <Slider v-model="settings.game_resolution[0]" :min="400" :max="2562" :step="2" />
            </span>
            <span class="slider">
              Height
              <Slider v-model="settings.game_resolution[1]" :min="400" :max="2562" :step="2" />
            </span>
          </div>
        </div>
        <hr class="card-divider" />
        <div class="settings-group">
          <h3>Console</h3>
          <div class="toggle-setting">
            Maximum Concurrent Downloads
            <input
              v-model="settings.max_concurrent_downloads"
              type="text"
              name="concurrent-downloads"
              class="concurrent_downloads"
            />
          </div>
        </div>
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Commands</h2>
      <div class="settings-group">
        <div class="toggle-setting">
          Pre Launch
          <input ref="javaArgs" v-model="settings.hooks.pre_launch" type="text" class="input" />
        </div>
        <div class="toggle-setting">
          Wrapper
          <input ref="javaArgs" v-model="settings.hooks.wrapper" type="text" class="input" />
        </div>
        <div class="toggle-setting">
          Post Launch
          <input ref="javaArgs" v-model="settings.hooks.post_exit" type="text" class="input" />
        </div>
      </div>
    </Card>
  </div>
</template>

<style lang="scss">
.save-btn {
  position: absolute !important;
  z-index: 100;
  top: 4rem;
  right: 4rem;
  opacity: 0;
  transition: 0.2s ease-in-out all;
}

.slider-input {
  width: 5rem !important;
  flex-basis: 5rem !important;
}

.concurrent_downloads {
  width: 4rem !important;
}

.installation-input {
  width: 100%;
}

.theming,
.settings-card {
  margin: 1rem;
  display: flex;
  flex-direction: column;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;

  .slider {
    width: 40%;
  }

  .description {
    line-height: 1.2rem;
    font-size: 1.05rem;
    width: 40%;

    h3 {
      font-size: 1.1rem;
      margin-bottom: 0.5rem;
    }
  }

  .input {
    width: 60%;
    flex-basis: 24rem;
  }

  .theme-dropdown {
    text-transform: capitalize;
  }
}

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
</style>
