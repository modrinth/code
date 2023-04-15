<script setup>
import { ref, onBeforeMount } from 'vue'
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
  SaveIcon,
  AnimatedLogo,
} from 'omorphia'
import { BrowseIcon } from '@/assets/icons'
import { useTheming } from '@/store/state'
import { get, set } from '@/helpers/settings'
import { find_jre_8_jres, find_jre_17_jres } from '@/helpers/jre'
import { open } from '@tauri-apps/api/dialog'

const themeStore = useTheming()

const loading = ref(false)
const settings = ref({})
const java8InstallOptions = ref([])
const java17InstallOptions = ref([])

onBeforeMount(async () => {
  loading.value = true
  settings.value = await get()

  // Finding possible Java 8 and 17 installations
  java8InstallOptions.value = await find_jre_8_jres()
  java17InstallOptions.value = await find_jre_17_jres()

  // Setting java version defaults. These can come as NULL from Tauri.
  if (!settings.value.java_globals?.JAVA_8)
    settings.value.java_globals.JAVA_8 = { path: '', version: '' }
  if (!settings.value.java_globals?.JAVA_17)
    settings.value.java_globals.JAVA_17 = { path: '', version: '' }
  loading.value = false
})

// DOM refs
const detectJava17Modal = ref(null)
const detectJava8Modal = ref(null)

const handleTheme = (e) => themeStore.setThemeState(e.option.toLowerCase())
const saveJavaPath = () => {}
const saveSettings = async () => await set(settings.value)

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
  <AnimatedLogo v-if="loading" class="loading" />
  <div v-else style="margin-bottom: 3.5rem">
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
              <span v-if="settings.java_globals.JAVA_17.path === java17Install.path"
                ><CheckIcon />Selected</span
              >

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
              <span v-if="settings.java_globals.JAVA_8.path === java8Install.path"
                ><CheckIcon />Selected</span
              >
              <span v-else><PlusIcon />Select</span>
            </Button>
          </div>
        </div>
      </div>
    </Modal>
    <Card class="theming">
      <h2>Themes</h2>
      <div class="toggle-setting" style="display: flex">
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
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Launcher Settings</h2>
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
    <Button color="primary" class="save-btn" @click="saveSettings"><SaveIcon />Save changes</Button>
  </div>
</template>

<style lang="scss">
.active {
  background: var(--color-brand-highlight) !important;
}

.concurrent-downloads {
  min-height: 40px;
  max-width: 4.2rem !important;
  text-align: center;
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
  margin: 1rem;
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

.theme-dropdown {
  text-transform: capitalize;
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

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100% !important;
}
</style>
