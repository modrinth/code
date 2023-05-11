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
  XIcon,
  PlusIcon,
  AnimatedLogo,
  Toggle,
} from 'omorphia'
import { BrowseIcon } from '@/assets/icons'
import { useTheming } from '@/store/state'
import { get, set } from '@/helpers/settings'
import { find_jre_8_jres, find_jre_17_jres, get_jre } from '@/helpers/jre'
import { open } from '@tauri-apps/api/dialog'

const themeStore = useTheming()

const fetchSettings = await get()

if (!fetchSettings.java_globals?.JAVA_8)
  fetchSettings.java_globals.JAVA_8 = { path: '', version: '' }
if (!fetchSettings.java_globals?.JAVA_17)
  fetchSettings.java_globals.JAVA_17 = { path: '', version: '' }
const settings = ref(fetchSettings)

const chosenInstallOptions = ref([])
const browsingInstall = ref(0)

const testingJava17 = ref(false)
const java17Success = ref(null)
const testingJava8 = ref(false)
const java8Success = ref(null)

// DOM refs
const detectJavaModal = ref(null)

const handleTheme = async (e) => {
  themeStore.setThemeState(e.option.toLowerCase())
  settings.value.theme = themeStore.selectedTheme
  await set(settings.value)
}

const handleCollapse = async (e) => {
  themeStore.collapsedNavigation = e
  settings.value.collapsed_navigation = themeStore.collapsedNavigation
  await set(settings.value)
}

const loadJavaModal = async (version) => {
  if (version === 17) chosenInstallOptions.value = await find_jre_17_jres()
  else if (version === 8) chosenInstallOptions.value = await find_jre_8_jres()

  browsingInstall.value = version
  detectJavaModal.value.show()
}

watch(settings.value, async (oldSettings, newSettings) => {
  await set(newSettings)
})

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

const handleJava17Test = async () => {
  let result
  testingJava17.value = true
  setTimeout(async () => {
    result = await get_jre(settings.value.java_globals.JAVA_17.path)
    testingJava17.value = false
    if (result) java17Success.value = true
    else java17Success.value = false

    setTimeout(() => {
      java17Success.value = null
    }, 2000)
  }, 1000)
}

const handleJava8Test = async () => {
  let result
  testingJava8.value = true
  setTimeout(async () => {
    result = await get_jre(settings.value.java_globals.JAVA_8.path)
    testingJava8.value = false
    java8Success.value = !!result

    setTimeout(() => {
      java8Success.value = null
    }, 2000)
  }, 1000)
}

const setJavaInstall = (javaInstall) => {
  if (browsingInstall.value === 17) settings.value.java_globals.JAVA_17 = javaInstall
  else if (browsingInstall.value === 8) settings.value.java_globals.JAVA_8 = javaInstall
  detectJavaModal.value.hide()
  chosenInstallOptions.value = []
  browsingInstall.value = 0
}
</script>

<template>
  <div>
    <Modal ref="detectJavaModal" header="Select java version">
      <div class="auto-detect-modal">
        <div class="table">
          <div class="table-row table-head">
            <div class="table-cell table-text">Version</div>
            <div class="table-cell table-text">Path</div>
            <div class="table-cell table-text">Actions</div>
          </div>
          <div
            v-for="javaInstall in chosenInstallOptions"
            :key="javaInstall.path"
            class="table-row"
          >
            <div class="table-cell table-text">
              <span>{{ javaInstall.version }}</span>
            </div>
            <div class="table-cell table-text">
              <span>{{ javaInstall.path }}</span>
            </div>
            <div class="table-cell table-text manage">
              <Button
                :disabled="
                  settings.java_globals.JAVA_17.path === javaInstall.path ||
                  settings.java_globals.JAVA_8.path === javaInstall.path
                "
                class="select-btn"
                @click="() => setJavaInstall(javaInstall)"
              >
                <span
                  v-if="
                    settings.java_globals.JAVA_17.path === javaInstall.path ||
                    settings.java_globals.JAVA_8.path === javaInstall.path
                  "
                >
                  <CheckIcon />Selected
                </span>
                <span v-else><PlusIcon />Select</span>
              </Button>
            </div>
          </div>
          <div v-if="chosenInstallOptions.length === 0" class="table-row entire-row">
            <div class="table-cell table-text">No JARS Found!</div>
          </div>
        </div>
      </div>
    </Modal>
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
          @change="handleTheme"
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
          @update:model-value="(value) => handleCollapse(value)"
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
            <Button @click="() => loadJavaModal(17)">
              <SearchIcon />
              Auto Detect
            </Button>
            <Button @click="handleJava17FileInput">
              <BrowseIcon />
              Browse
            </Button>
            <Button @click="handleJava17Test">
              <PlayIcon />
              Test
            </Button>
            <AnimatedLogo v-if="testingJava17 === true" class="testing-loader" />
            <CheckIcon
              v-else-if="java17Success === true && testingJava17 === false"
              class="test-success"
            />
            <XIcon
              v-else-if="java17Success === false && testingJava17 === false"
              class="test-fail"
            />
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
            <Button @click="() => loadJavaModal(8)">
              <SearchIcon />
              Auto Detect
            </Button>
            <Button @click="handleJava8FileInput">
              <BrowseIcon />
              Browse
            </Button>
            <Button @click="handleJava8Test">
              <PlayIcon />
              Test
            </Button>
            <AnimatedLogo v-if="testingJava8 === true" class="testing-loader" />
            <CheckIcon
              v-else-if="java8Success === true && testingJava8 === false"
              class="test-success"
            />
            <XIcon v-else-if="java8Success === false && testingJava8 === false" class="test-fail" />
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
          placeholder="Enter java arguments..."
        />
      </div>
      <div class="settings-group">
        <h3>Environment Arguments</h3>
        <input
          v-model="settings.custom_env_args"
          type="text"
          class="input installation-input"
          placeholder="Enter environment arguments..."
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
        <h3>Resource Management</h3>
        <div class="toggle-setting">
          <span>Maximum Concurrent Downloads</span>
          <Slider
            v-model="settings.max_concurrent_downloads"
            class="concurrent-downloads"
            :min="1"
            :max="100"
            :step="1"
          />
        </div>
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Commands</h2>
      <div class="settings-group">
        <div class="toggle-setting">
          Pre Launch
          <input v-model="settings.hooks.pre_launch" type="text" class="input" />
        </div>
        <div class="toggle-setting">
          Wrapper
          <input v-model="settings.hooks.wrapper" type="text" class="input" />
        </div>
        <div class="toggle-setting">
          Post Launch
          <input v-model="settings.hooks.post_exit" type="text" class="input" />
        </div>
      </div>
    </Card>
  </div>
</template>

<style lang="scss">
.concurrent-downloads {
  width: 80% !important;
}

.auto-detect-modal {
  padding: 1rem;

  .table {
    .table-row {
      grid-template-columns: 1fr 4fr 1.5fr;
    }

    span {
      display: inherit;
      align-items: center;
      justify-content: center;
    }
  }
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
  gap: 1rem;
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

.manage {
  display: flex;
  gap: 0.5rem;
}

.testing-loader {
  height: 1rem !important;
  width: 1rem !important;

  svg {
    height: inherit !important;
    width: inherit !important;
  }
}

.test-success {
  color: var(--color-green);
}

.test-fail {
  color: var(--color-red);
}
</style>
