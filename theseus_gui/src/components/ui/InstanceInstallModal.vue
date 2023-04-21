<script setup>
import { Avatar, Modal, Button, DownloadIcon, PlusIcon, Card, UploadIcon, XIcon, DropdownSelect } from 'omorphia'
import { computed, ref } from 'vue'
import { add_project_from_version as installMod, list } from '@/helpers/profile'
import { tauri } from '@tauri-apps/api'
import { open } from '@tauri-apps/api/dialog'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import {LoginIcon} from "@/assets/icons";
import {get_game_versions} from "@/helpers/tags";
import {useRouter} from "vue-router";
import {create} from "@/helpers/profile";
const router = useRouter()
const version = ref('')
const project = ref('')
const installModal = ref(null)
const searchFilter = ref('')
const showCreation = ref(false)
const icon = ref(null)
const name = ref(null)
const display_icon = ref(null)
const loader = ref(null)
const gameVersion = ref(null)
const creatingInstance = ref(false)

defineExpose({
  show: (projectId, id) => {
    project.value = projectId
    version.value = id
    installModal.value.show()
    searchFilter.value = ''
  },
})

const profiles = ref(await list().then(Object.values))

async function install(instance) {
  instance.installing = true
  await installMod(instance.path, version.value)
  instance.installed = true
  instance.installing = false
}

const filteredVersions = computed(() => {
  const filtered = profiles.value.filter((profile) => {
    return profile.metadata.name.toLowerCase().includes(searchFilter.value.toLowerCase())
  })

  filtered.map((profile) => {
    profile.installing = false
    profile.installed = checkInstalled(profile)
  })

  return filtered
})

const checkInstalled = (profile) => {
  console.log(Object.values(profile.projects))
  console.log(project.value)
  return Object.values(profile.projects).some((p) => p.metadata?.project?.id === project.value)
}

const toggleCreation = () => {
  showCreation.value = !showCreation.value
  name.value = null
  icon.value = null
  display_icon.value = null
  gameVersion.value = null
  loader.value = null
}

const gameVersions = ref(
  (await get_game_versions()).filter((item) => item.version_type === 'release').map((item) => item.version)
)

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

  display_icon.value = tauri.convertFileSrc(icon.value)
}

const reset_icon = () => {
  icon.value = null
  display_icon.value = null
}

const createInstance = async () => {
  const loader_version_value = 'latest'
  creatingInstance.value = true
  const id = await create(
    name.value,
    gameVersion.value,
    loader.value,
    loader_version_value,
    icon.value
  )

  await installMod(id, version.value)

  await router.push({ path: `/instance/${encodeURIComponent(id)}` })
  installModal.value.hide()
  creatingInstance.value = false
}

const check_valid = computed(() => {
  return (
    name.value &&
    gameVersion.value &&
    loader.value
  )
})
</script>

<template>
  <Modal ref="installModal" header="Install mod to instance">
    <div class="modal-body">
      <input v-model="searchFilter" type="text" class="search" placeholder="Search for a profile" />
      <div class="profiles">
        <div
          v-for="profile in filteredVersions"
          :key="profile.metadata.name"
          class="option"
        >
          <Avatar :src="convertFileSrc(profile.metadata.icon)" size="xs" />
          <div class="name">{{ profile.metadata.name }}</div>
          <div class="footer">
            <Button :disabled="profile.installed || profile.installing" @click="install(profile)">
              <DownloadIcon v-if="!profile.installed && !profile.installing"/>
              {{ profile.installing ? 'Installing...' :profile.installed ? 'Installed' : 'Install'}}
            </Button>
            <Button @click="$router.push(`/instance/${encodeURIComponent(profile.path)}`)">
              <LoginIcon />
              View
            </Button>
          </div>
        </div>
      </div>
      <Card v-if="showCreation" class="creation-card">
        <div class="creation-container">
          <div class="creation-icon">
            <Avatar
              size="md"
              class="icon"
              :src="display_icon"
            />
            <div class="creation-icon__description">
              <Button @click="upload_icon()">
                <UploadIcon />
                Upload Icon
              </Button>
              <Button @click="reset_icon()">
                <XIcon />
                Remove Icon
              </Button>
            </div>
          </div>
          <div class="creation-settings">
            <input v-model="name" type="text" placeholder="Name" class="creation-input"/>
            <div class="dropdown-options">
              <DropdownSelect
                v-model="gameVersion"
                :options="gameVersions"
                class="creation-dropdown"
                placeholder="Version"
              />
              <DropdownSelect
                v-model="loader"
                :options="['fabric', 'forge', 'quilt']"
                class="creation-dropdown"
                placeholder="Loader"
              />
            </div>
          </div>
        </div>
        <div class="footer">
          <Button :disabled="creatingInstance === true || !check_valid" @click="createInstance()">
            <PlusIcon />
            {{ creatingInstance ? 'Creating...' : 'Create'}}
          </Button>
        </div>
      </Card>
      <div class="footer">
        <Button color="primary" @click="toggleCreation()">
          <PlusIcon />
          Create new instance
        </Button>
        <Button @click="installModal.hide()">Cancel</Button>
      </div>
    </div>
  </Modal>
</template>

<style scoped lang="scss">
.creation-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-top: 1rem;
  padding: 1rem;
  background-color: var(--color-bg);
}

.creation-container {
  display: flex;
  flex-direction: row;
  gap: 1rem;
}

.creation-icon {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  align-items: center;
  flex-grow: 1;

  .creation-icon__description {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
}

.creation-input {
  width: 100%;
}

.creation-dropdown {
  width: min-content !important;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.creation-settings {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 0.5rem;
  align-items: center;
  width: min-content;

  .dropdown-options {
    display: flex;
    flex-direction: row;
    gap: 0.5rem;
  }
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
}

.profiles {
  max-height: 18rem;
  overflow-y: auto;
}

.option {
  width: calc(100%);
  background: var(--color-raised-bg);
  color: var(--color-base);
  box-shadow: none;
  display: flex;
  flex-direction: row;
  padding: 0.5rem;

  img {
    margin-right: 0.5rem;
  }

  .name {
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
}

.footer {
  display: flex;
  flex-direction: row;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-left: auto;
}
</style>
