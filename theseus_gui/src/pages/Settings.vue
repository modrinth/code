<script setup>
import { ref, watch } from 'vue'
import {
  Card,
  Slider,
  DropdownSelect,
  Button,
  SearchIcon,
  PlayIcon,
  Modal,
  CheckIcon,
  PlusIcon,
} from 'omorphia'
import { BrowseIcon } from '@/assets/icons'
import { useTheming } from '@/store/state'
import { get, set, deepEqual } from '@/helpers/settings'
import { find_jre_8_jres, find_jre_17_jres } from '@/helpers/jre'
import { open } from '@tauri-apps/api/dialog'

const themeStore = useTheming()

// Original object to track changes
const originalSettings = ref(await get())
// Object to bind
const settings = ref(await get())
// Finding possible Java 8 and 17 installations
const java8InstallOptions = ref(await find_jre_8_jres())
const java17InstallOptions = ref(await find_jre_17_jres())

// Setting java version defaults
if (!settings.value.java_globals.JAVA_8)
  settings.value.java_globals.JAVA_8 = { path: '', version: '' }
if (!settings.value.java_globals.JAVA_17)
  settings.value.java_globals.JAVA_17 = { path: '', version: '' }

// DOM refs
const detectJava17Modal = ref(null)
const detectJava8Modal = ref(null)
const saveButton = ref(null)

watch(settings.value, (newSettings) => {
  // Validate the changed state
  if (newSettings.hooks.pre_launch === '') delete settings.value.hooks.pre_launch
  if (newSettings.hooks.wrapper === '') delete settings.value.hooks.wrapper
  if (newSettings.hooks.post_exit === '') delete settings.value.hooks.post_exit

  settings.value.max_concurrent_downloads = newSettings.max_concurrent_downloads

  if (deepEqual(originalSettings.value, settings.value)) saveButton.value.$el.style.opacity = 0
  else saveButton.value.$el.style.opacity = 1

  if (!settings.value.java_globals.JAVA_8)
    settings.value.java_globals.JAVA_8 = { path: '', version: '' }
  if (!settings.value.java_globals.JAVA_17)
    settings.value.java_globals.JAVA_17 = { path: '', version: '' }
})

const handleTheme = (e) => themeStore.setThemeState(e.option.toLowerCase())
const saveJavaPath = () => {}
const saveSettings = async () => {
  await set(settings.value)
  saveButton.value.$el.style.opacity = 0
}

const handleJava17FileInput = async () => {
  let filePath = await open()
  settings.value.java_globals.JAVA_17 = {
    path: filePath,
    version: '17',
  }
}
const handleJava8FileInput = async () => {
  let filePath = await open()
  settings.value.java_globals.JAVA_8 = {
    path: filePath,
    version: '8',
  }
}

const setJava17Install = (chosenInstall) => {
  settings.value.java_globals.JAVA_17 = chosenInstall
  detectJava17Modal.value.hide()
}
const setJava8Install = (chosenInstall) => {
  settings.value.java_globals.JAVA_8 = chosenInstall
  detectJava8Modal.value.hide()
}
</script>

<template>
  <div>
    <Button ref="saveButton" color="primary" class="save-btn" @click="saveSettings">Save</Button>
    <Modal ref="detectJava17Modal" header="Auto Detect Java Version" class="auto-detect-modal">
      <div class="table-container">
        <div class="table-row table-head">
          <div class="table-cell table-text">Version</div>
          <div class="table-cell table-text">Path</div>
          <div class="table-cell table-text">Actions</div>
        </div>
        <div
          v-for="java17Install in java17InstallOptions"
          :key="java17Install.path"
          class="table-row"
        >
          <div class="table-cell table-text">
            <span>{{ java17Install.version }}</span>
          </div>
          <div class="table-cell table-text">
            <span>{{ java17Install.path }}</span>
          </div>
          <div class="table-cell table-text manage">
            <Button
              :disabled="settings.java_globals.JAVA_17.path === java17Install.path"
              @click="() => setJava17Install(java17Install)"
            >
              <CheckIcon v-if="settings.java_globals.JAVA_17.path === java17Install.path" />
              <span v-else><PlusIcon />Select</span>
            </Button>
          </div>
        </div>
      </div>
    </Modal>
    <Modal ref="detectJava8Modal" header="Auto Detect Java Version" class="auto-detect-modal">
      <div class="table-container">
        <div class="table-row table-head">
          <div class="table-cell table-text">Version</div>
          <div class="table-cell table-text">Path</div>
          <div class="table-cell table-text">Actions</div>
        </div>
        <div v-for="java8Install in java8InstallOptions" :key="java8Install.path" class="table-row">
          <div class="table-cell table-text">
            {{ java8Install.version }}
          </div>
          <div class="table-cell table-text">
            {{ java8Install.path }}
          </div>
          <div class="table-cell table-text manage">
            <Button
              :disabled="settings.java_globals.JAVA_8.path === java8Install.path"
              @click="() => setJava8Install(java8Install)"
            >
              <CheckIcon v-if="settings.java_globals.JAVA_8.path === java8Install.path" />
              <span v-else><PlusIcon />Select</span>
            </Button>
          </div>
        </div>
      </div>
    </Modal>
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
            v-model="settings.java_globals.JAVA_17.path"
            type="text"
            class="input installation-input"
            placeholder="/path/to/java17"
          />
          <span class="installation-buttons">
            <Button @click="() => detectJava17Modal.show()">
              <SearchIcon />
              Auto Detect
            </Button>
            <Button @click="handleJava17FileInput">
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
            v-model="settings.java_globals.JAVA_8.path"
            type="text"
            class="input installation-input"
            placeholder="/path/to/java8"
          />
          <span class="installation-buttons">
            <Button @click="() => detectJava8Modal.show()">
              <SearchIcon />
              Auto Detect
            </Button>
            <Button @click="handleJava8FileInput">
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
      <div class="settings-group">
        <h3>Java Arguments</h3>
        <input
          v-model="settings.custom_java_args"
          type="text"
          class="input installation-input"
          placeholder="-Xms10G -Xmx10G -XX:+UseG1GC -XX:+ParallelRefProcEnabled"
        />
      </div>
      <div class="settings-group">
        <h3>Environment Arguments</h3>
        <input
          v-model="settings.custom_env_args"
          type="text"
          class="input installation-input"
          placeholder="ENV1=TEST ENV2=TEST2"
        />
      </div>
      <hr class="card-divider" />
      <div class="settings-group">
        <div class="sliders">
          <span class="slider">
            Minimum Memory
            <Slider v-model="settings.memory.minimum" :min="1000" :max="8200" :step="10" />
          </span>
          <span class="slider">
            Maximum Memory
            <Slider v-model="settings.memory.maximum" :min="1000" :max="8200" :step="10" />
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
              type="number"
              name="concurrent-downloads"
              class="concurrent-downloads"
            />
          </div>
        </div>
      </div>
    </Card>
    <Card class="settings-card" style="margin-bottom: 3.5rem">
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
.active {
  background: var(--color-brand-highlight) !important;
}

.concurrent-downloads {
  border-radius: var(--radius-md);
  box-sizing: border-box;
  border: 2px solid transparent;
  // safari iOS rounds inputs by default
  // set the appearance to none to prevent this
  appearance: none !important;
  background: var(--color-button-bg);
  color: var(--color-base);
  padding: 0.5rem 1rem;
  font-weight: var(--font-weight-medium);
  outline: 2px solid transparent;
  box-shadow: var(--shadow-inset-sm), 0 0 0 0 transparent;
  transition: box-shadow 0.1s ease-in-out;
  min-height: 40px;
  max-width: 3.6rem !important;

  &::-webkit-inner-spin-button,
  &::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  &:focus,
  &:focus-visible {
    box-shadow: inset 0 0 0 transparent, 0 0 0 0.25rem var(--color-brand-shadow);
    color: var(--color-contrast);
  }

  &:disabled,
  &[disabled] {
    opacity: 0.6;
    pointer-events: none;
    cursor: not-allowed;
  }

  &:focus::placeholder {
    opacity: 0.8;
  }

  &::placeholder {
    color: var(--color-contrast);
    opacity: 0.6;
  }
}

.auto-detect-modal {
  .modal-body {
    width: 45rem !important;
    .content {
      padding: 1rem;

      button {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: center;

        span {
          display: inherit;
          align-items: center;
          justify-content: center;
        }
      }
    }
  }
}
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

.installation-input {
  width: 100% !important;
  flex-grow: 1;
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

.table-container {
  display: grid;
  grid-template-rows: repeat(auto-fill, auto);
  width: 100%;
  border-radius: var(--radius-md);
  overflow: hidden;
}

.table-row {
  display: grid;
  grid-template-columns: 1fr 4fr 1.5fr;
}

.table-head {
  .table-cell {
    background-color: var(--color-accent-contrast);
  }
}

.table-cell {
  padding: 1rem;
  height: 100%;
  align-items: center;
  vertical-align: center;
  display: flex;
}

.table-text {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  display: flex;

  span {
    display: inline-block;
    text-overflow: ellipsis;
    overflow: hidden;
  }
}

.manage {
  display: flex;
  gap: 0.5rem;
}

.table-row:nth-child(even) .table-cell {
  background-color: var(--color-bg);
}
</style>
