<template>
  <Card v-if="instance.metadata.linked_data" class="modpack-header card">
    <div class="label">
      <h3>
        <span class="label__title size-card-header"
          >{{ instance.metadata.name }} -
          {{
            installedVersionData.name.charAt(0).toUpperCase() + installedVersionData.name.slice(1)
          }}</span
        >
      </h3>
    </div>
    <div v-if="!instance.locked" class="adjacent-input">
      <Card class="unlocked-instance">
        This is an unlocked instance. There may be unexpected behaviour unintended by the modpack
        creator.
      </Card>
    </div>
    <div v-else class="adjacent-input">
      <label for="repair-profile">
        <span class="label__title">Unlock instance</span>
        <span class="label__description">
          Allows modifications to the instance, which allows you to add projects to the modpack. The
          pack will remain linked, and you can still change versions. Only mods listed in the
          modpack will be modified on version changes.
        </span>
      </label>
      <Button id="repair-profile" @click="unlockProfile"> <XIcon /> Unlock </Button>
    </div>

    <div class="adjacent-input">
      <label for="repair-profile">
        <span class="label__title">Unpair instance</span>
        <span class="label__description">
          Removes the link to an external Modrinth modpack on the instance. This allows you to edit
          modpacks you download through the browse page but you will not be able to update the
          instance from a new version of a modpack if you do this.
        </span>
      </label>
      <Button id="repair-profile" @click="unpairProfile"> <XIcon /> Unpair </Button>
    </div>

    <div class="adjacent-input">
      <label for="repair-profile">
        <span class="label__title">Reinstall modpack</span>
        <span class="label__description">
          Removes all projects and reinstalls Modrinth modpack. Use this to fix unexpected behaviour
          if your instance is diverging from the Modrinth modpack. This also re-locks the instance.
        </span>
      </label>
      <Button
        id="repair-profile"
        color="highlight"
        :disabled="installing || inProgress || offline"
        @click="repairModpack"
      >
        <DownloadIcon /> Reinstall
      </Button>
    </div>
  </Card>
  <Card v-else>
    <div class="label">
      <h3>
        <span class="label__title size-card-header">Export modpack </span>
      </h3>
    </div>
    <div class="adjacent-input">
      <div class="labeled_input">
        <p>Modpack Name</p>
        <div class="iconified-input">
          <PackageIcon />
          <input
            v-model="nameInput"
            type="text"
            placeholder="Modpack name"
            class="input input-widen"
          />
          <Button @click="nameInput = ''">
            <XIcon />
          </Button>
        </div>
      </div>
      <div class="labeled_input">
        <p>Version number</p>
        <div class="iconified-input">
          <VersionIcon />
          <input v-model="versionInput" type="text" placeholder="1.0.0" class="input" />
          <Button @click="versionInput = ''">
            <XIcon />
          </Button>
        </div>
      </div>
    </div>
    <div class="adjacent-input">
      <div class="labeled_input">
        <p>Description</p>

        <div class="textarea-wrapper">
          <textarea
            v-model="exportDescription"
            placeholder="My super-special modpack is the best of them all!"
          />
        </div>
      </div>
    </div>
    <div class="adjacent-input"></div>
    <div class="table adjacent-input">
      <div class="table-head">
        <div class="table-cell row-wise">
          <span>Select files and folders to include in pack </span>
          <Button
            class="sleek-primary collapsed-button"
            icon-only
            @click="() => (showingFiles = !showingFiles)"
          >
            <PlusIcon v-if="!showingFiles" />
            <XIcon v-else />
          </Button>
        </div>
      </div>
      <div v-if="showingFiles" class="table-content">
        <div v-for="[path, children] of folders" :key="path.name" class="table-row">
          <div class="table-cell file-entry">
            <div class="file-primary">
              <Checkbox
                :model-value="children.every((child) => child.selected)"
                :label="path.name"
                class="select-checkbox"
                :disabled="children.every((x) => x.disabled)"
                @update:model-value="
                  (newValue) => children.forEach((child) => (child.selected = newValue))
                "
              />
              <Checkbox
                v-model="path.showingMore"
                class="select-checkbox dropdown"
                collapsing-toggle-style
              />
            </div>
            <div v-if="path.showingMore" class="file-secondary">
              <div v-for="child in children" :key="child.path" class="file-secondary-row">
                <Checkbox
                  v-model="child.selected"
                  :label="child.name"
                  class="select-checkbox"
                  :disabled="child.disabled"
                />
              </div>
            </div>
          </div>
        </div>
        <div v-for="file in files" :key="file.path" class="table-row">
          <div class="table-cell file-entry">
            <div class="file-primary">
              <Checkbox
                v-model="file.selected"
                :label="file.name"
                :disabled="file.disabled"
                class="select-checkbox"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="button-row push-right">
      <Button v-tooltip="'Sharing is not yet implemented!'" disabled>
        <SendIcon />
        Share
      </Button>

      <Button color="primary" @click="exportPack">
        <PackageIcon />
        Export modpack
      </Button>
    </div>
  </Card>
  <Card v-if="instance.metadata.linked_data" class="filter-header">
    <div class="manage">
      <multiselect
        v-model="filterLoader"
        :options="
          versions
            .flatMap((value) => value.loaders)
            .filter((value, index, self) => self.indexOf(value) === index)
        "
        :multiple="true"
        :searchable="true"
        :show-no-results="false"
        :close-on-select="false"
        :clear-search-on-select="false"
        :show-labels="false"
        :selectable="() => versions.length <= 6"
        placeholder="Filter loader..."
        :custom-label="(option) => option.charAt(0).toUpperCase() + option.slice(1)"
      />
      <multiselect
        v-model="filterGameVersions"
        :options="
          versions
            .flatMap((value) => value.game_versions)
            .filter((value, index, self) => self.indexOf(value) === index)
        "
        :multiple="true"
        :searchable="true"
        :show-no-results="false"
        :close-on-select="false"
        :clear-search-on-select="false"
        :show-labels="false"
        :selectable="() => versions.length <= 6"
        placeholder="Filter versions..."
        :custom-label="(option) => option.charAt(0).toUpperCase() + option.slice(1)"
      />
      <multiselect
        v-model="filterVersions"
        :options="
          versions
            .map((value) => value.version_type)
            .filter((value, index, self) => self.indexOf(value) === index)
        "
        :multiple="true"
        :searchable="true"
        :show-no-results="false"
        :close-on-select="false"
        :clear-search-on-select="false"
        :show-labels="false"
        :selectable="() => versions.length <= 6"
        placeholder="Filter release channel..."
        :custom-label="(option) => option.charAt(0).toUpperCase() + option.slice(1)"
      />
    </div>
    <Button
      class="no-wrap clear-filters"
      :disabled="
        filterVersions.length === 0 && filterLoader.length === 0 && filterGameVersions.length === 0
      "
      :action="clearFilters"
    >
      <ClearIcon />
      Clear filters
    </Button>
  </Card>
  <Pagination
    v-if="instance.metadata.linked_data"
    :page="currentPage"
    :count="Math.ceil(filteredVersions.length / 20)"
    class="pagination-before"
    :link-function="(page) => `?page=${page}`"
    @switch-page="switchPage"
  />
  <Card v-if="instance.metadata.linked_data" class="mod-card">
    <div class="table">
      <div class="table-row with-columns table-head">
        <div class="table-cell table-text download-cell" />
        <div class="name-cell table-cell table-text">Name</div>
        <div class="table-cell table-text">Supports</div>
        <div class="table-cell table-text">Stats</div>
      </div>
      <div
        v-for="version in filteredVersions.slice((currentPage - 1) * 20, currentPage * 20)"
        :key="version.id"
        class="table-row with-columns selectable"
        @click="$router.push(`/project/${$route.params.id}/version/${version.id}`)"
      >
        <div class="table-cell table-text">
          <Button
            :color="version.id === installedVersion ? '' : 'primary'"
            icon-only
            :disabled="inProgress || installing || version.id === installedVersion"
            @click.stop="() => switchVersion(version.id)"
          >
            <SwapIcon v-if="version.id !== installedVersion" />
            <CheckIcon v-else />
          </Button>
        </div>
        <div class="name-cell table-cell table-text">
          <div class="version-link">
            {{ version.name.charAt(0).toUpperCase() + version.name.slice(1) }}
            <div class="version-badge">
              <div class="channel-indicator">
                <Badge
                  :color="releaseColor(version.version_type)"
                  :type="
                    version.version_type.charAt(0).toUpperCase() + version.version_type.slice(1)
                  "
                />
              </div>
              <div>
                {{ version.version_number }}
              </div>
            </div>
          </div>
        </div>
        <div class="table-cell table-text stacked-text">
          <span>
            {{
              version.loaders.map((str) => str.charAt(0).toUpperCase() + str.slice(1)).join(', ')
            }}
          </span>
          <span>
            {{ version.game_versions.join(', ') }}
          </span>
        </div>
        <div class="table-cell table-text stacked-text">
          <div>
            <span> Published on </span>
            <strong>
              {{
                new Date(version.date_published).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'short',
                  day: 'numeric',
                })
              }}
            </strong>
          </div>
          <div>
            <strong>
              {{ formatNumber(version.downloads) }}
            </strong>
            <span> Downloads </span>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup>
import {
  Card,
  Button,
  CheckIcon,
  PlusIcon,
  ClearIcon,
  XIcon,
  DownloadIcon,
  SendIcon,
  Badge,
  Checkbox,
  Pagination,
  formatNumber,
} from 'omorphia'
import Multiselect from 'vue-multiselect'
import { PackageIcon, VersionIcon } from '@/assets/icons'
import { releaseColor } from '@/helpers/utils'
import { computed, ref, toRef, watch } from 'vue'
import { SwapIcon } from '@/assets/icons/index.js'
import {
  edit,
  update_managed_modrinth_version,
  update_repair_modrinth,
  get_potential_override_folders,
  export_profile_mrpack,
} from '@/helpers/profile'
import { mixpanel_track } from '@/helpers/mixpanel'
import { handleError } from '@/store/notifications'
import { sep } from '@tauri-apps/api/path'
import { open } from '@tauri-apps/api/dialog'

const filterVersions = ref([])
const filterLoader = ref(props.instance ? [props.instance?.metadata?.loader] : [])
const filterGameVersions = ref(props.instance ? [props.instance?.metadata?.game_version] : [])

const currentPage = ref(1)

const clearFilters = () => {
  filterVersions.value = []
  filterLoader.value = []
  filterGameVersions.value = []
}

const props = defineProps({
  versions: {
    type: Array,
    required: true,
  },
  installing: {
    type: Boolean,
    default: false,
  },
  instance: {
    type: Object,
    default: null,
  },
})

const installedVersion = computed(() => props.instance?.metadata?.linked_data?.version_id)
const installedVersionData = computed(() =>
  filteredVersions.value.find((version) => version.id === installedVersion.value)
)
const installing = computed(() => props.instance.install_stage !== 'installed')
const inProgress = ref(false)

const nameInput = ref(props.instance.metadata.name)
const exportDescription = ref('')
const versionInput = ref('1.0.0')
const showingFiles = ref(false)
const files = ref([])
const folders = ref([])

const initFiles = async () => {
  const newFolders = new Map()
  files.value = []
  await get_potential_override_folders(props.instance.path).then((filePaths) =>
    filePaths
      .map((folder) => ({
        path: folder,
        name: folder.split(sep).pop(),
        selected:
          folder.startsWith('mods') ||
          folder.startsWith('datapacks') ||
          folder.startsWith('resourcepacks') ||
          folder.startsWith('shaderpacks') ||
          folder.startsWith('config'),
        disabled:
          folder === 'profile.json' ||
          folder.startsWith('modrinth_logs') ||
          folder.startsWith('.fabric') ||
          folder.includes('.DS_Store'),
      }))
      .forEach((pathData) => {
        const parent = pathData.path.split(sep).slice(0, -1).join(sep)
        if (parent !== '') {
          if (newFolders.has(parent)) {
            newFolders.get(parent).push(pathData)
          } else {
            newFolders.set(parent, [pathData])
          }
        } else {
          files.value.push(pathData)
        }
      })
  )
  folders.value = [...newFolders.entries()].map(([name, value]) => [
    {
      name,
      showingMore: false,
    },
    value,
  ])
}
await initFiles()

if (props.instance.export_cache) {
  const export_cache = toRef(props.instance.export_cache).value
  if (export_cache.name) {
    nameInput.value = export_cache.name
  }
  if (export_cache.version) {
    versionInput.value = export_cache.version
  }
  if (export_cache.description) {
    exportDescription.value = export_cache.description
  }
  for (const file of export_cache.overrides) {
    files.value.forEach((f) => {
      if (f.path === file) f.selected = true
    })
    folders.value.forEach((args) => {
      args[1].forEach((child) => {
        if (child.path === file) {
          child.selected = true
        }
      })
    })
  }
}

const exportPack = async () => {
  const filesToExport = files.value.filter((file) => file.selected).map((file) => file.path)
  folders.value.forEach((args) => {
    args[1].forEach((child) => {
      if (child.selected) {
        filesToExport.push(child.path)
      }
    })
  })
  const outputPath = await open({
    directory: true,
    multiple: false,
  })

  if (outputPath) {
    export_profile_mrpack(
      props.instance.path,
      outputPath + `/${nameInput.value} ${versionInput.value}.mrpack`,
      filesToExport,
      versionInput.value,
      exportDescription.value,
      nameInput.value
    ).catch((err) => handleError(err))
  }
}

const switchVersion = async (versionId) => {
  inProgress.value = true
  console.log('update version', props.instance.path, versionId)
  await update_managed_modrinth_version(props.instance.path, versionId)
  inProgress.value = false
}

async function unpairProfile() {
  const editProfile = props.instance
  editProfile.metadata.linked_data = null
  editProfile.locked = false
  await edit(props.instance.path, editProfile)
  installedVersion.value = null
  installedVersionData.value = null
}

async function unlockProfile() {
  const editProfile = props.instance
  editProfile.locked = false
  await edit(props.instance.path, editProfile)
}

async function repairModpack() {
  inProgress.value = true
  await update_repair_modrinth(props.instance.path).catch(handleError)
  inProgress.value = false

  mixpanel_track('InstanceRepair', {
    loader: props.instance.metadata.loader,
    game_version: props.instance.metadata.game_version,
  })
}

const filteredVersions = computed(() => {
  return props.versions.filter(
    (projectVersion) =>
      (filterGameVersions.value.length === 0 ||
        filterGameVersions.value.some((gameVersion) =>
          projectVersion.game_versions.includes(gameVersion)
        )) &&
      (filterLoader.value.length === 0 ||
        filterLoader.value.some((loader) => projectVersion.loaders.includes(loader))) &&
      (filterVersions.value.length === 0 ||
        filterVersions.value.includes(projectVersion.version_type))
  )
})

function switchPage(page) {
  currentPage.value = page
}

//watch all the filters and if a value changes, reset to page 1
watch([filterVersions, filterLoader, filterGameVersions], () => {
  currentPage.value = 1
})
</script>

<style scoped lang="scss">
.filter-header {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

// .table-row {
//   grid-template-columns: min-content 1fr 1fr 1.5fr;
//   // background-color: red;
//   // color: red;
// }

.with-columns {
  grid-template-columns: min-content 1fr 1fr 1.5fr;
}

.manage {
  display: flex;
  gap: 0.5rem;
  flex-grow: 1;

  .multiselect {
    flex-grow: 1;
  }
}

.card-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--color-raised-bg);
}

.mod-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  overflow: hidden;
  margin-top: 0.5rem;
}

.text-combo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.select {
  width: 100% !important;
  max-width: 20rem;
}

.version-link {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;

  .version-badge {
    display: flex;
    flex-wrap: wrap;

    .channel-indicator {
      margin-right: 0.5rem;
    }
  }
}

.stacked-text {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  align-items: flex-start;
}

.download-cell {
  width: 4rem;
  padding: 1rem;
}

.filter-checkbox {
  :deep(.checkbox) {
    border: none;
  }
}

.unlocked-instance {
  background-color: var(--color-gray);
  color: black;
}

.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: var(--gap-sm);
  align-items: center;
}

.modal-body {
  padding: var(--gap-xl);
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
}

.labeled_input {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);

  p {
    margin: 0;
  }
}

.select-checkbox {
  gap: var(--gap-sm);

  button.checkbox {
    border: none;
  }

  &.dropdown {
    margin-left: auto;
  }
}

.table-content {
  max-height: 18rem;
  overflow-y: auto;
}

.table {
  border: 1px solid var(--color-bg);
}

.file-entry {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
}

.file-primary {
  display: flex;
  align-items: center;
  gap: var(--gap-sm);
}

.file-secondary {
  margin-left: var(--gap-xl);
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
  height: 100%;
  vertical-align: center;
}

.file-secondary-row {
  display: flex;
  align-items: center;
  gap: var(--gap-sm);
}

.push-right {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: flex-end;
}
.row-wise {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
}

.textarea-wrapper {
  // margin-top: 1rem;
  height: 12rem;
  textarea {
    max-height: 12rem;
  }
  .preview {
    overflow-y: auto;
  }
}

.input-widen {
  // stretch across space
  flex-grow: 1;
}
</style>
