<script setup>
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  Card,
  Button,
  SearchIcon,
  Slider,
  AnimatedLogo,
  CheckIcon,
  XIcon,
  Modal,
  TrashIcon,
  DropdownSelect,
} from 'omorphia'
import { BrowseIcon, PlayIcon, HammerIcon } from '@/assets/icons'
import { get_jre, get_all_jre } from '@/helpers/jre'
import { remove, install } from '@/helpers/profile'
import { get_game_versions } from '@/helpers/tags'
import { open } from '@tauri-apps/api/dialog'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
})

const router = useRouter()

const settings = ref({
  resolution: props.instance.resolution ?? [400, 800],
  memory: props.instance.memory ?? { minimum: 1000, maximum: 2000 },
  hooks: props.instance.hooks ?? { pre_launch: '', wrapper: '', post_exit: '' },
  javaArgs: props.instance.java.extra_arguments ?? '',
  jreKey: props.instance.java.jre_key ?? '',
  javaPath: '',
  loader: props.instance.metadata.loader,
  gameVersion: props.instance.metadata.game_version,
})

watch(settings.value, (_, newSettings) => {
  console.log(newSettings)

  const newInstance = {
    ...props.instance,
    resolution: [...settings.value.resolution],
    memory: { ...settings.value.memory },
    hooks: { ...settings.value.hooks },
    java: { jre_key: settings.value.jreKey, extra_arguments: settings.value.javaArgs },
    metadata: {
      ...props.instance.metadata,
      loader: settings.value.loader,
      game_version: settings.value.gameVersion,
    },
  }

  // TODO: Save new data to the instance once able.
  console.log('compare')
  console.log('old', props.instance)
  console.log('new', newInstance)
})

const gameVersions = await get_game_versions()

// TODO: Remove or wire up depending on whether we add these as instance settings.
const fullscreen = ref(false)
const consoleSetting = ref(false)

const testingJava = ref(false)
const javaTestSuccess = ref(null)
const possibleJavaOptions = ref([])
const detectJavaModal = ref(null)

const handleJavaFileInput = async () => {
  let filePath = await open()
  settings.value.javaPath = filePath
}

const handleJavaTest = async () => {
  let result
  testingJava.value = true
  setTimeout(async () => {
    result = await get_jre(settings.value.javaPath)
    testingJava.value = false

    javaTestSuccess.value = !!result

    setTimeout(() => {
      javaTestSuccess.value = null
    }, 2000)
  }, 1000)
}

const loadJavaModal = async () => {
  possibleJavaOptions.value = await get_all_jre()
  detectJavaModal.value.show()
}

const setJavaInstall = (javaInstall) => {
  settings.value.javaPath = javaInstall.path
  detectJavaModal.value.hide()
  possibleJavaOptions.value = []
}

const handleRepair = async () => {
  try {
    await install(props.instance.path)
  } catch (err) {
    console.warn('Repair error:', err)
  }
}

const handleRemove = async () => {
  await remove(props.instance.path)
  router.push('/')
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
          <div v-for="javaInstall in possibleJavaOptions" :key="javaInstall.path" class="table-row">
            <div class="table-cell table-text">
              <span>{{ javaInstall.version }}</span>
            </div>
            <div class="table-cell table-text">
              <span>{{ javaInstall.path }}</span>
            </div>
            <div class="table-cell table-text manage">
              <Button
                :disabled="settings.javaPath === javaInstall.path"
                class="select-btn"
                @click="() => setJavaInstall(javaInstall)"
              >
                <span v-if="settings.javaPath === javaInstall.path"> <CheckIcon />Selected </span>
                <span v-else><PlusIcon />Select</span>
              </Button>
            </div>
          </div>
          <div v-if="possibleJavaOptions.length === 0" class="table-row entire-row">
            <div class="table-cell table-text">No JARS Found!</div>
          </div>
        </div>
      </div>
    </Modal>
    <Card class="settings-card">
      <h2 class="settings-title">Java</h2>
      <div class="settings-group">
        <h3>Installation</h3>
        <input
          v-model="settings.javaPath"
          type="text"
          class="input installation-input"
          placeholder="/Library/Java/JavaVirtualMachines/zulu-17.jdk/Contents/Home"
        />
        <span class="installation-buttons">
          <Button @click="loadJavaModal">
            <SearchIcon />
            Auto Detect
          </Button>
          <Button @click="handleJavaFileInput">
            <BrowseIcon />
            Browse
          </Button>
          <Button @click="handleJavaTest">
            <PlayIcon />
            Test
          </Button>
          <AnimatedLogo v-if="testingJava === true" class="testing-loader" />
          <CheckIcon
            v-else-if="javaTestSuccess === true && testingJava === false"
            class="test-success"
          />
          <XIcon v-else-if="javaTestSuccess === false && testingJava === false" class="test-fail" />
        </span>
      </div>
      <hr class="card-divider" />
      <div class="settings-group">
        <h3>Arguments</h3>
        <input
          ref="javaArgs"
          v-model="settings.javaArgs"
          type="text"
          class="input installation-input"
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
      <h2 class="settings-title">Window</h2>
      <div class="settings-group">
        <div class="settings-group">
          <div class="sliders">
            <span class="slider">
              Width
              <Slider v-model="settings.resolution[0]" :min="400" :max="2562" :step="2" />
            </span>
            <span class="slider">
              Height
              <Slider v-model="settings.resolution[1]" :min="400" :max="2562" :step="2" />
            </span>
          </div>
          <div class="toggle-setting">
            Start in Fullscreen
            <input
              id="fullscreen"
              v-model="fullscreen"
              type="checkbox"
              name="fullscreen"
              class="switch stylized-toggle"
            />
          </div>
        </div>
        <hr class="card-divider" />
        <div class="settings-group">
          <h3>Console</h3>
          <div class="toggle-setting">
            Show console while game is running
            <input
              id="fullscreen"
              v-model="consoleSetting"
              type="checkbox"
              name="fullscreen"
              class="switch stylized-toggle"
            />
          </div>
          <div class="toggle-setting">
            Close console when game quits
            <input
              id="fullscreen"
              v-model="consoleSetting"
              type="checkbox"
              name="fullscreen"
              class="switch stylized-toggle"
            />
          </div>
          <div class="toggle-setting">
            Show console when game crashes
            <input
              id="fullscreen"
              v-model="consoleSetting"
              type="checkbox"
              name="fullscreen"
              class="switch stylized-toggle"
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
    <Card class="settings-card">
      <h2 class="settings-title">Profile management</h2>
      <div class="settings-group">
        <div class="toggle-setting">
          Profile loader
          <DropdownSelect
            v-model="settings.loader"
            :options="['forge', 'fabric', 'vanilla']"
            class="loader-dropdown"
          />
        </div>
      </div>
      <div class="settings-group">
        <div class="toggle-setting">
          Game version
          <DropdownSelect
            v-model="settings.gameVersion"
            :options="gameVersions.map((v) => v.version)"
          />
        </div>
      </div>
      <hr class="card-divider" />
      <div class="settings-group">
        <div class="toggle-setting">
          Repair profile
          <Button color="highlight" @click="handleRepair"><HammerIcon /> Repair</Button>
        </div>
        <div class="toggle-setting">
          Delete profile
          <Button color="danger" @click="handleRemove"><TrashIcon /> Delete</Button>
        </div>
      </div>
    </Card>
  </div>
</template>

<style lang="scss">
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
  width: 100% !important;
}

.installation-buttons {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  margin: 0;
}

.loader-dropdown {
  text-transform: capitalize;
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
