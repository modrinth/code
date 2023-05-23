<script setup>
import {
  Avatar,
  Modal,
  Button,
  DownloadIcon,
  PlusIcon,
  Card,
  UploadIcon,
  XIcon,
  RightArrowIcon,
  CheckIcon,
} from 'omorphia'
import { computed, ref } from 'vue'
import { add_project_from_version as installMod, check_installed, list } from '@/helpers/profile'
import { tauri } from '@tauri-apps/api'
import { open } from '@tauri-apps/api/dialog'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { create } from '@/helpers/profile'
import { installVersionDependencies } from '@/helpers/utils'
import { handleError } from '@/store/notifications.js'

const versions = ref([])
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
  show: async (projectId, selectedVersions) => {
    project.value = projectId
    versions.value = selectedVersions
    installModal.value.show()
    searchFilter.value = ''

    profiles.value = await getData()
  },
})

const profiles = ref([])

async function install(instance) {
  instance.installing = true
  const version = versions.value.find((v) => {
    return (
      v.game_versions.includes(instance.metadata.game_version) &&
      (v.loaders.includes(instance.metadata.loader) || v.loaders.includes('minecraft'))
    )
  })

  await installMod(instance.path, version.id).catch(handleError)
  await installVersionDependencies(instance, version)

  instance.installedMod = true
  instance.installing = false
}

async function getData() {
  const projects = await list(true).then(Object.values).catch(handleError)

  const filtered = projects
    .filter((profile) => {
      return profile.metadata.name.toLowerCase().includes(searchFilter.value.toLowerCase())
    })
    .filter((profile) => {
      return (
        versions.value.flatMap((v) => v.game_versions).includes(profile.metadata.game_version) &&
        versions.value
          .flatMap((v) => v.loaders)
          .some(
            (value) =>
              value === profile.metadata.loader || ['minecraft', 'iris', 'optifine'].includes(value)
          )
      )
    })

  for (let profile of filtered) {
    profile.installing = false
    profile.installedMod = await check_installed(profile.path, project.value).catch(handleError)
  }

  return filtered
}

const toggleCreation = () => {
  showCreation.value = !showCreation.value
  name.value = null
  icon.value = null
  display_icon.value = null
  gameVersion.value = null
  loader.value = null
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

const createInstance = async () => {
  creatingInstance.value = true
  const id = await create(
    name.value,
    versions.value[0].game_versions[0],
    versions.value[0].loaders[0] !== 'forge' ||
      versions.value[0].loaders[0] !== 'fabric' ||
      versions.value[0].loaders[0] !== 'quilt'
      ? versions.value[0].loaders[0]
      : 'vanilla',
    'latest',
    icon.value
  ).catch(handleError)

  await installMod(id, versions.value[0].id).catch(handleError)

  installModal.value.hide()
  creatingInstance.value = false
}

const check_valid = computed(() => {
  return name.value
})
</script>

<template>
  <Modal ref="installModal" header="Install mod to instance">
    <div class="modal-body">
      <input v-model="searchFilter" type="text" class="search" placeholder="Search for a profile" />
      <div class="profiles" :class="{ 'hide-creation': !showCreation }">
        <div v-for="profile in profiles" :key="profile.metadata.name" class="option">
          <Button
            color="raised"
            class="profile-button"
            @click="$router.push(`/instance/${encodeURIComponent(profile.path)}`)"
          >
            <Avatar :src="convertFileSrc(profile.metadata.icon)" class="profile-image" />
            {{ profile.metadata.name }}
          </Button>
          <Button :disabled="profile.installedMod || profile.installing" @click="install(profile)">
            <DownloadIcon v-if="!profile.installedMod && !profile.installing" />
            <CheckIcon v-else-if="profile.installedMod" />
            {{
              profile.installing ? 'Installing...' : profile.installedMod ? 'Installed' : 'Install'
            }}
          </Button>
        </div>
      </div>
      <Card v-if="showCreation" class="creation-card">
        <div class="creation-container">
          <div class="creation-icon">
            <Avatar size="md" class="icon" :src="display_icon" />
            <div class="creation-icon__description">
              <Button @click="upload_icon()">
                <UploadIcon />
                <span class="no-wrap"> Select icon </span>
              </Button>
              <Button @click="reset_icon()">
                <XIcon />
                <span class="no-wrap"> Remove icon </span>
              </Button>
            </div>
          </div>
          <div class="creation-settings">
            <input v-model="name" type="text" placeholder="Name" class="creation-input" />
            <Button :disabled="creatingInstance === true || !check_valid" @click="createInstance()">
              <RightArrowIcon />
              {{ creatingInstance ? 'Creating...' : 'Create' }}
            </Button>
          </div>
        </div>
      </Card>
      <div class="footer">
        <Button :color="showCreation ? '' : 'primary'" @click="toggleCreation()">
          <PlusIcon />
          {{ showCreation ? 'Hide New Instance' : 'Create new instance' }}
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
  margin: 0;
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

.no-wrap {
  white-space: nowrap;
}

.creation-dropdown {
  width: min-content !important;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.creation-settings {
  width: 100%;
  margin-left: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  justify-content: center;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
}

.profiles {
  max-height: 12rem;
  overflow-y: auto;

  &.hide-creation {
    max-height: 21rem;
  }
}

.option {
  width: calc(100%);
  background: var(--color-raised-bg);
  color: var(--color-base);
  box-shadow: none;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: 0 0.5rem;
  gap: 0.5rem;

  img {
    margin-right: 0.5rem;
  }

  .name {
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .profile-button {
    align-content: start;
    padding: 0.5rem;
    text-align: left;
  }
}

.profile-image {
  --size: 2rem !important;
}

.footer {
  display: flex;
  flex-direction: row;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-left: auto;
}
</style>
