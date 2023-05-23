<template>
  <Modal ref="changeVersionsModal" header="Change instance versions">
    <div class="change-versions-modal universal-body">
      <div class="input-row">
        <p class="input-label">Loader</p>
        <Chips v-model="loader" :items="loaders" />
      </div>
      <div class="input-row">
        <p class="input-label">Game Version</p>
        <div class="versions">
          <DropdownSelect v-model="gameVersion" :options="selectableGameVersions" render-up />
          <Checkbox v-model="showSnapshots" class="filter-checkbox" label="Include snapshots" />
        </div>
      </div>
      <div v-if="loader !== 'vanilla'" class="input-row">
        <p class="input-label">Loader Version</p>
        <DropdownSelect
          :model-value="selectableLoaderVersions[loaderVersionIndex]"
          :options="selectableLoaderVersions"
          :display-name="(option) => option?.id"
          render-up
          @change="(value) => (loaderVersionIndex = value.index)"
        />
      </div>
      <div class="button-group">
        <button class="btn" @click="$refs.changeVersionsModal.hide()">
          <XIcon />
          Cancel
        </button>
        <button
          class="btn btn-primary"
          :disabled="!isValid || editing"
          @click="saveGvLoaderEdits()"
        >
          <SaveIcon />
          {{ editing ? 'Saving...' : 'Save changes' }}
        </button>
      </div>
    </div>
  </Modal>
  <section class="card">
    <div class="label">
      <h3>
        <span class="label__title size-card-header">Profile</span>
      </h3>
    </div>
    <label for="instance-icon">
      <span class="label__title">Icon</span>
    </label>
    <div class="input-group">
      <Avatar
        :src="!icon || (icon && icon.startsWith('http')) ? icon : convertFileSrc(icon)"
        size="md"
        class="project__icon"
      />
      <div class="input-stack">
        <button id="instance-icon" class="btn" @click="setIcon">
          <UploadIcon />
          Select icon
        </button>
        <button class="btn" @click="resetIcon">
          <TrashIcon />
          Remove icon
        </button>
      </div>
    </div>

    <label for="project-name">
      <span class="label__title">Name</span>
    </label>
    <input id="profile-name" v-model="title" maxlength="80" type="text" />

    <div class="adjacent-input">
      <label for="edit-versions">
        <span class="label__title">Edit mod loader/game versions</span>
        <span class="label__description">
          Allows you to change the mod loader, loader version, or game version of the profile.
        </span>
      </label>
      <button id="edit-versions" class="btn" @click="$refs.changeVersionsModal.show()">
        <EditIcon />
        Edit versions
      </button>
    </div>
  </section>
  <Card class="settings-card">
    <h2 class="settings-title">Java</h2>
    <div class="settings-group">
      <h3>Installation</h3>
      <Checkbox v-model="overrideJavaInstall" label="Override global java installations" />
      <JavaSelector v-model="javaInstall" :disabled="!overrideJavaInstall" />
    </div>
    <hr class="card-divider" />
    <div class="settings-group">
      <h3>Java arguments</h3>
      <Checkbox v-model="overrideJavaArgs" label="Override global java arguments" />
      <input
        v-model="javaArgs"
        :disabled="!overrideJavaArgs"
        type="text"
        class="input installation-input"
        placeholder="Enter java arguments..."
      />
    </div>
    <div class="settings-group">
      <h3>Environment variables</h3>
      <Checkbox v-model="overrideEnvVars" label="Override global environment variables" />
      <input
        v-model="envVars"
        :disabled="!overrideEnvVars"
        type="text"
        class="input installation-input"
        placeholder="Enter environment variables..."
      />
    </div>
    <hr class="card-divider" />
    <div class="settings-group">
      <Checkbox v-model="overrideMemorySettings" label="Override global memory settings" />
      <div class="sliders">
        <span class="slider">
          Minimum memory
          <Slider
            v-model="memory.minimum"
            :disabled="!overrideMemorySettings"
            :min="256"
            :max="maxMemory"
            :step="10"
          />
        </span>
        <span class="slider">
          Maximum memory
          <Slider
            v-model="memory.maximum"
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
          v-model="resolution[0]"
          :disabled="!overrideWindowSettings"
          type="number"
          class="input"
          @change="updateProfile"
        />
      </div>
      <div class="toggle-setting">
        Height
        <input
          v-model="resolution[1]"
          :disabled="!overrideWindowSettings"
          type="number"
          class="input"
          @change="updateProfile"
        />
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title">Hooks</h2>
    <Checkbox v-model="overrideHooks" label="Override global hooks" />
    <div class="settings-group">
      <div class="toggle-setting">
        Pre launch
        <input v-model="hooks.pre_launch" :disabled="!overrideHooks" type="text" />
      </div>
      <div class="toggle-setting">
        Wrapper
        <input v-model="hooks.wrapper" :disabled="!overrideHooks" type="text" />
      </div>
      <div class="toggle-setting">
        Post exit
        <input v-model="hooks.post_exit" :disabled="!overrideHooks" type="text" />
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title">Profile management</h2>
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
import {
  Card,
  Slider,
  TrashIcon,
  Checkbox,
  UploadIcon,
  Avatar,
  EditIcon,
  Modal,
  Chips,
  DropdownSelect,
  XIcon,
  SaveIcon,
  HammerIcon,
} from 'omorphia'
import { useRouter } from 'vue-router'
import { edit, edit_icon, get_optimal_jre_key, install, remove } from '@/helpers/profile.js'
import { computed, readonly, ref, shallowRef, watch } from 'vue'
import { get_max_memory } from '@/helpers/jre.js'
import { get } from '@/helpers/settings.js'
import JavaSelector from '@/components/ui/JavaSelector.vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { get_fabric_versions, get_forge_versions, get_quilt_versions } from '@/helpers/metadata.js'
import { get_game_versions, get_loaders } from '@/helpers/tags.js'
import { handleError } from '@/store/notifications.js'

const router = useRouter()

const props = defineProps({
  instance: {
    type: Object,
    required: true,
  },
})

const title = ref(props.instance.metadata.name)
const icon = ref(props.instance.metadata.icon)

async function resetIcon() {
  icon.value = null
  await edit_icon(props.instance.path, null).catch(handleError)
}

async function setIcon() {
  const value = await open({
    multiple: false,
    filters: [
      {
        name: 'Image',
        extensions: ['png', 'jpeg', 'svg', 'webp', 'gif', 'jpg'],
      },
    ],
  })

  if (!value) return

  icon.value = value
  await edit_icon(props.instance.path, icon.value).catch(handleError)
}

const globalSettings = await get().catch(handleError)

const javaSettings = props.instance.java ?? {}

const overrideJavaInstall = ref(!!javaSettings.override_version)
const optimalJava = readonly(await get_optimal_jre_key(props.instance.path).catch(handleError))
const javaInstall = ref(optimalJava ?? javaSettings.override_version ?? { path: '', version: '' })

const overrideJavaArgs = ref(!!javaSettings.extra_arguments)
const javaArgs = ref((javaSettings.extra_arguments ?? globalSettings.custom_java_args).join(' '))

const overrideEnvVars = ref(!!javaSettings.custom_env_args)
const envVars = ref(
  (javaSettings.custom_env_args ?? globalSettings.custom_env_args).map((x) => x.join('=')).join(' ')
)

const overrideMemorySettings = ref(!!props.instance.memory)
const memory = ref(props.instance.memory ?? globalSettings.memory)
const maxMemory = (await get_max_memory().catch(handleError)) / 1024

const overrideWindowSettings = ref(!!props.instance.resolution)
const resolution = ref(props.instance.resolution ?? globalSettings.game_resolution)

const overrideHooks = ref(!!props.instance.hooks)
const hooks = ref(props.instance.hooks ?? globalSettings.hooks)

watch(
  [
    title,
    overrideJavaInstall,
    javaInstall,
    overrideJavaArgs,
    javaArgs,
    overrideEnvVars,
    envVars,
    overrideMemorySettings,
    memory.value,
    overrideWindowSettings,
    resolution.value,
    overrideHooks,
    hooks.value,
  ],
  async () => {
    const editProfile = {
      metadata: {
        name: title.value,
      },
      java: {},
    }

    if (overrideJavaInstall.value) {
      if (javaInstall.value.path !== '') {
        editProfile.java.override_version = javaInstall.value
      }
    }

    if (overrideJavaArgs.value) {
      if (javaArgs.value !== '') {
        editProfile.java.extra_arguments = javaArgs.value.trim().split(/\s+/).filter(Boolean)
      }
    }

    if (overrideEnvVars.value) {
      if (envVars.value !== '') {
        editProfile.java.custom_env_args = envVars.value
          .trim()
          .split(/\s+/)
          .filter(Boolean)
          .map((x) => x.split('=').filter(Boolean))
      }
    }

    if (overrideMemorySettings.value) {
      editProfile.memory = memory.value
    }

    if (overrideWindowSettings.value) {
      editProfile.resolution = resolution.value
    }

    if (overrideHooks.value) {
      editProfile.hooks = hooks.value
    }

    await edit(props.instance.path, editProfile)
  }
)

const repairing = ref(false)

async function repairProfile() {
  repairing.value = true
  await install(props.instance.path).catch(handleError)
  repairing.value = false
}

const removing = ref(false)
async function removeProfile() {
  removing.value = true
  await remove(props.instance.path).catch(handleError)
  removing.value = false

  await router.push({ path: '/' })
}

const changeVersionsModal = ref(null)
const showSnapshots = ref(false)

const [fabric_versions, forge_versions, quilt_versions, all_game_versions, loaders] =
  await Promise.all([
    get_fabric_versions().then(shallowRef).catch(handleError),
    get_forge_versions().then(shallowRef).catch(handleError),
    get_quilt_versions().then(shallowRef).catch(handleError),
    get_game_versions().then(shallowRef).catch(handleError),
    get_loaders()
      .then((value) =>
        value
          .filter((item) => item.supported_project_types.includes('modpack'))
          .map((item) => item.name.toLowerCase())
      )
      .then(ref)
      .catch(handleError),
  ])
loaders.value.push('vanilla')

const loader = ref(props.instance.metadata.loader)

const gameVersion = ref(props.instance.metadata.game_version)
const selectableGameVersions = computed(() => {
  return all_game_versions.value
    .filter((item) => {
      let defaultVal = item.version_type === 'release' || showSnapshots.value
      if (loader.value === 'fabric') {
        defaultVal &= fabric_versions.value.gameVersions.some((x) => item.version === x.id)
      } else if (loader.value === 'forge') {
        defaultVal &= forge_versions.value.gameVersions.some((x) => item.version === x.id)
      } else if (loader.value === 'quilt') {
        defaultVal &= quilt_versions.value.gameVersions.some((x) => item.version === x.id)
      }

      return defaultVal
    })
    .map((item) => item.version)
})

const selectableLoaderVersions = computed(() => {
  if (gameVersion.value) {
    if (loader.value === 'fabric') {
      return fabric_versions.value.gameVersions[0].loaders
    } else if (loader.value === 'forge') {
      return forge_versions.value.gameVersions.find((item) => item.id === gameVersion.value).loaders
    } else if (loader.value === 'quilt') {
      return quilt_versions.value.gameVersions[0].loaders
    }
  }
  return []
})
const loaderVersionIndex = ref(
  selectableLoaderVersions.value.findIndex(
    (x) => x.id === props.instance.metadata.loader_version?.id
  )
)

const isValid = computed(() => {
  return (
    selectableGameVersions.value.includes(gameVersion.value) &&
    (loaderVersionIndex.value >= 0 || loader.value === 'vanilla')
  )
})

watch(loader, () => (loaderVersionIndex.value = 0))

const editing = ref(false)
async function saveGvLoaderEdits() {
  editing.value = true

  const editProfile = {
    metadata: {
      game_version: gameVersion.value,
      loader: loader.value,
    },
  }

  if (loader.value !== 'vanilla') {
    editProfile.metadata.loader_version = selectableLoaderVersions.value[loaderVersionIndex.value]
  }
  await edit(props.instance.path, editProfile).catch(handleError)
  await repairProfile()

  editing.value = false
  changeVersionsModal.value.hide()
}
</script>

<style scoped lang="scss">
.change-versions-modal {
  display: flex;
  flex-direction: column;
  padding: 1rem;
  gap: 1rem;

  .input-label {
    font-size: 1rem;
    font-weight: bolder;
    color: var(--color-contrast);
    margin-bottom: 0.5rem;
  }

  .versions {
    display: flex;
    flex-direction: row;
    gap: 1rem;
  }

  .button-group {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }
}

.settings-card {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.input-group {
  display: flex;
  gap: 0.5rem;
}

.input-stack {
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

.installation-input {
  width: 100%;
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
