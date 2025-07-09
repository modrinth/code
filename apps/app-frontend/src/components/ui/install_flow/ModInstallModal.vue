<script setup>
import {
  DownloadIcon,
  PlusIcon,
  UploadIcon,
  XIcon,
  RightArrowIcon,
  CheckIcon,
} from '@modrinth/assets'
import { Avatar, Button, Card } from '@modrinth/ui'
import { computed, ref } from 'vue'
import {
  add_project_from_version as installMod,
  check_installed,
  get,
  list,
  create,
} from '@/helpers/profile'
import { open } from '@tauri-apps/plugin-dialog'
import { installVersionDependencies } from '@/store/install.js'
import { handleError } from '@/store/notifications.js'
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import { trackEvent } from '@/helpers/analytics'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

const router = useRouter()

const versions = ref()
const project = ref()

const installModal = ref()
const searchFilter = ref('')

const showCreation = ref(false)
const icon = ref(null)
const name = ref(null)
const display_icon = ref(null)
const loader = ref(null)
const gameVersion = ref(null)
const creatingInstance = ref(false)

const profiles = ref([])

const shownProfiles = computed(() =>
  profiles.value
    .filter((profile) => {
      return profile.name.toLowerCase().includes(searchFilter.value.toLowerCase())
    })
    .filter((profile) => {
      const loaders = versions.value.flatMap((v) => v.loaders)

      return (
        versions.value.flatMap((v) => v.game_versions).includes(profile.game_version) &&
        (project.value.project_type === 'mod'
          ? loaders.includes(profile.loader) || loaders.includes('minecraft')
          : true)
      )
    }),
)

const onInstall = ref(() => {})

defineExpose({
  show: async (projectVal, versionsVal, callback) => {
    project.value = projectVal
    versions.value = versionsVal
    searchFilter.value = ''

    showCreation.value = false
    name.value = null
    icon.value = null
    display_icon.value = null
    gameVersion.value = null
    loader.value = null

    onInstall.value = callback

    const profilesVal = await list().catch(handleError)
    for (const profile of profilesVal) {
      profile.installing = false
      profile.installedMod = await check_installed(profile.path, project.value.id).catch(
        handleError,
      )
    }
    profiles.value = profilesVal

    installModal.value.show()

    trackEvent('ProjectInstallStart', { source: 'ProjectInstallModal' })
  },
})

async function install(instance) {
  instance.installing = true
  const version = versions.value.find((v) => {
    return (
      v.game_versions.includes(instance.game_version) &&
      (project.value.project_type === 'mod'
        ? v.loaders.includes(instance.loader) || v.loaders.includes('minecraft')
        : true)
    )
  })

  if (!version) {
    instance.installing = false
    handleError('No compatible version found')
    return
  }

  await installMod(instance.path, version.id).catch(handleError)
  await installVersionDependencies(instance, version)

  instance.installedMod = true
  instance.installing = false

  trackEvent('ProjectInstall', {
    loader: instance.loader,
    game_version: instance.game_version,
    id: project.value.id,
    version_id: version.id,
    project_type: project.value.project_type,
    title: project.value.title,
    source: 'ProjectInstallModal',
  })

  onInstall.value(version.id)
}

const toggleCreation = () => {
  showCreation.value = !showCreation.value
  name.value = null
  icon.value = null
  display_icon.value = null
  gameVersion.value = null
  loader.value = null

  if (showCreation.value) {
    trackEvent('InstanceCreateStart', { source: 'ProjectInstallModal' })
  }
}

const upload_icon = async () => {
  const res = await open({
    multiple: false,
    filters: [
      {
        name: 'Image',
        extensions: ['png', 'jpeg'],
      },
    ],
  })
  icon.value = res.path ?? res

  if (!icon.value) return
  display_icon.value = convertFileSrc(icon.value)
}

const reset_icon = () => {
  icon.value = null
  display_icon.value = null
}

const createInstance = async () => {
  creatingInstance.value = true

  const loader =
    versions.value[0].loaders[0] !== 'forge' &&
    versions.value[0].loaders[0] !== 'fabric' &&
    versions.value[0].loaders[0] !== 'quilt'
      ? 'vanilla'
      : versions.value[0].loaders[0]

  const id = await create(
    name.value,
    versions.value[0].game_versions[0],
    loader,
    'latest',
    icon.value,
  ).catch(handleError)

  await installMod(id, versions.value[0].id).catch(handleError)

  await router.push(`/instance/${encodeURIComponent(id)}/`)

  const instance = await get(id, true)
  await installVersionDependencies(instance, versions.value[0])

  trackEvent('InstanceCreate', {
    profile_name: name.value,
    game_version: versions.value[0].game_versions[0],
    loader: loader,
    loader_version: 'latest',
    has_icon: !!icon.value,
    source: 'ProjectInstallModal',
  })

  trackEvent('ProjectInstall', {
    loader: loader,
    game_version: versions.value[0].game_versions[0],
    id: project.value,
    version_id: versions.value[0].id,
    project_type: project.value.project_type,
    title: project.value.title,
    source: 'ProjectInstallModal',
  })

  onInstall.value(versions.value[0].id)

  if (installModal.value) installModal.value.hide()
  creatingInstance.value = false
}
</script>

<template>
  <ModalWrapper ref="installModal" header="Install project to instance" :on-hide="onInstall">
    <div class="modal-body">
      <input
        v-model="searchFilter"
        autocomplete="off"
        type="text"
        class="search"
        placeholder="Search for an instance"
      />
      <div class="profiles" :class="{ 'hide-creation': !showCreation }">
        <div v-for="profile in shownProfiles" :key="profile.name" class="option">
          <router-link
            class="btn btn-transparent profile-button"
            :to="`/instance/${encodeURIComponent(profile.path)}`"
            @click="installModal.hide()"
          >
            <Avatar
              :src="profile.icon_path ? convertFileSrc(profile.icon_path) : null"
              class="profile-image"
            />
            {{ profile.name }}
          </router-link>
          <div
            v-tooltip="
              profile.linked_data?.locked && !profile.installedMod
                ? 'Unpair or unlock an instance to add mods.'
                : ''
            "
          >
            <Button
              :disabled="profile.installedMod || profile.installing"
              @click="install(profile)"
            >
              <DownloadIcon v-if="!profile.installedMod && !profile.installing" />
              <CheckIcon v-else-if="profile.installedMod" />
              {{
                profile.installing
                  ? 'Installing...'
                  : profile.installedMod
                    ? 'Installed'
                    : 'Install'
              }}
            </Button>
          </div>
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
              <Button :disabled="!display_icon" @click="reset_icon()">
                <XIcon />
                <span class="no-wrap"> Remove icon </span>
              </Button>
            </div>
          </div>
          <div class="creation-settings">
            <input
              v-model="name"
              autocomplete="off"
              type="text"
              placeholder="Name"
              class="creation-input"
            />
            <Button :disabled="creatingInstance === true || !name" @click="createInstance()">
              <RightArrowIcon />
              {{ creatingInstance ? 'Creating...' : 'Create' }}
            </Button>
          </div>
        </div>
      </Card>
      <div class="input-group push-right">
        <Button :color="showCreation ? '' : 'primary'" @click="toggleCreation()">
          <PlusIcon />
          {{ showCreation ? 'Hide New Instance' : 'Create new instance' }}
        </Button>
        <Button @click="installModal.hide()">Cancel</Button>
      </div>
    </div>
  </ModalWrapper>
</template>

<style scoped lang="scss">
.creation-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin: 0;
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
  min-width: 350px;
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
</style>
