<template>
  <Modal ref="modal" header="Create Instance">
    <div v-if="showContent" class="modal-body">
      <div class="image-upload">
        <Avatar :src="display_icon" size="md" :rounded="true" />
        <div class="image-input">
          <Button @click="upload_icon()">
            <UploadIcon />
            Upload Icon
          </Button>
          <Button @click="reset_icon">
            <XIcon />
            Remove Icon
          </Button>
        </div>
      </div>
      <div class="input-row">
        <p class="input-label">Name</p>
        <input v-model="profile_name" class="text-input" type="text" />
      </div>
      <div class="input-row">
        <p class="input-label">Game Version</p>
        <div class="versions">
          <DropdownSelect
            v-model="game_version"
            :options="game_versions"
            :render-up="!showAdvanced"
          />
          <Checkbox
            v-if="showAdvanced"
            v-model="showSnapshots"
            class="filter-checkbox"
            label="Include snapshots"
          />
        </div>
      </div>
      <div class="input-row">
        <p class="input-label">Loader</p>
        <Chips v-model="loader" :items="loaders" />
      </div>
      <div v-if="showAdvanced" class="input-row">
        <p class="input-label">Loader Version</p>
        <Chips v-model="loader_version" :items="['stable', 'latest', 'other']" />
      </div>
      <div v-if="showAdvanced && loader_version === 'other'">
        <div v-if="game_version" class="input-row">
          <p class="input-label">Select Version</p>
          <DropdownSelect
            v-model="specified_loader_version"
            :options="selectable_versions"
            render-up
          />
        </div>
        <div v-else class="input-row">
          <p class="warning">Select a game version before you select a loader version</p>
        </div>
      </div>
      <div class="button-group">
        <Button @click="toggle_advanced">
          <CodeIcon />
          {{ showAdvanced ? 'Hide Advanced' : 'Show Advanced' }}
        </Button>
        <Button @click="$refs.modal.hide()">
          <XIcon />
          Cancel
        </Button>
        <Button color="primary" :disabled="!check_valid || creating" @click="create_instance()">
          <PlusIcon v-if="!creating" />
          {{ creating ? 'Creating...' : 'Create' }}
        </Button>
      </div>
    </div>
  </Modal>
</template>

<script setup>
import {
  Avatar,
  Button,
  Chips,
  DropdownSelect,
  Modal,
  PlusIcon,
  UploadIcon,
  XIcon,
  CodeIcon,
  Checkbox,
} from 'omorphia'
import { computed, ref } from 'vue'
import { get_game_versions, get_loaders } from '@/helpers/tags'
import { create } from '@/helpers/profile'
import { open } from '@tauri-apps/api/dialog'
import { useRouter } from 'vue-router'
import { tauri } from '@tauri-apps/api'
import { get_fabric_versions, get_forge_versions } from '@/helpers/metadata'

const router = useRouter()

const profile_name = ref('')
const game_version = ref('')
const loader = ref('')
const loader_version = ref('stable')
const specified_loader_version = ref('')
const showContent = ref(false)
const icon = ref(null)
const display_icon = ref(null)
const showAdvanced = ref(false)
const creating = ref(false)
const showSnapshots = ref(false)

defineExpose({
  show: () => {
    showContent.value = false
    modal.value.show()
    game_version.value = ''
    specified_loader_version.value = ''
    profile_name.value = ''
    creating.value = false
    showAdvanced.value = false
    showSnapshots.value = false
    loader.value = ''
    loader_version.value = 'stable'
    icon.value = null
    display_icon.value = null

    setTimeout(() => {
      showContent.value = true
    }, 100)
  },
})

const all_game_versions = ref(await get_game_versions())

const game_versions = computed(() => {
  return all_game_versions.value
    .filter((item) => item.version_type === 'release' || showSnapshots.value)
    .map((item) => item.version)
})
const loaders = ref(
  await get_loaders().then((value) =>
    value
      .filter((item) => item.supported_project_types.includes('modpack'))
      .map((item) => item.name.toLowerCase())
  )
)
const modal = ref(null)

const check_valid = computed(() => {
  return profile_name.value && game_version.value
})

const create_instance = async () => {
  try {
    creating.value = true
    const loader_version_value =
      loader_version.value === 'other' ? specified_loader_version.value : loader_version.value

    const id = await create(
      profile_name.value,
      game_version.value,
      loader.value,
      loader_version_value ?? 'stable',
      icon.value
    )

    await router.push({ path: `/instance/${encodeURIComponent(id)}` })
    modal.value.hide()
    creating.value = false
  } catch (e) {
    console.error(e)
    creating.value = false
  }
}

const upload_icon = async () => {
  icon.value = await open({
    multiple: false,
    filters: [
      {
        name: 'Image',
        extensions: ['png', 'jpeg'],
      },
    ],
  })

  if (!icon.value) return
  display_icon.value = tauri.convertFileSrc(icon.value)
}

const reset_icon = () => {
  icon.value = null
  display_icon.value = null
}

const fabric_versions = ref(await get_fabric_versions())
const forge_versions = ref(await get_forge_versions())

const selectable_versions = computed(() => {
  if (game_version.value) {
    if (loader.value === 'fabric') {
      return fabric_versions.value.gameVersions[0].loaders.map((item) => item.id)
    } else if (loader.value === 'forge') {
      return forge_versions.value.gameVersions
        .find((item) => item.id === game_version.value)
        .loaders.map((item) => item.id)
    }
  }
  return []
})

const toggle_advanced = () => {
  showAdvanced.value = !showAdvanced.value
}
</script>

<style scoped>
.modal-body {
  display: flex;
  flex-direction: column;
  padding: 1rem;
  gap: 1rem;
}

.input-label {
  font-size: 1rem;
  font-weight: bolder;
  color: var(--color-contrast);
  margin-bottom: 0.5rem;
}

.text-input {
  width: 20rem;
}

.button-group {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}

.image-upload {
  display: flex;
  gap: 1rem;
}

.image-input {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  justify-content: center;
}

.warning {
  font-style: italic;
}

.versions {
  display: flex;
  flex-direction: row;
  gap: 1rem;
}

:deep(button.checkbox) {
  border: none;
}
</style>
