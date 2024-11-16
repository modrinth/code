<template>
  <div
    v-if="isPackLocked"
    class="border-2 border-solid rounded-2xl border-bg-raised p-4 flex flex-col mb-4 gap-2"
  >
    <div class="flex items-center text-lg font-semibold text-contrast">
      <LockIcon class="mr-2" />
      Content locked
    </div>
    <p class="m-0">
      This instance is linked to the modpack
      <router-link
        :to="{
          path: `/project/${instance.linked_data.project_id}/`,
          query: { i: props.instance.path },
        }"
        >{{ instance.name }}</router-link
      >. Adding or removing content may break the modpack or cause unexpected behavior.
    </p>
    <p class="m-0 text-sm text-secondary">
      You can unlock content in this instance's settings if you know what you're doing.
    </p>
  </div>
  <template v-if="projects.length > 0">
    <div class="flex items-center gap-2 mb-4">
      <div class="iconified-input flex-grow">
        <SearchIcon />
        <input
          v-model="searchFilter"
          type="text"
          :placeholder="`Search content...`"
          class="text-input search-input"
          autocomplete="off"
        />
        <Button class="r-btn" @click="() => (searchFilter = '')">
          <XIcon />
        </Button>
      </div>
      <AddContentButton v-if="!isPackLocked" :instance="instance" />
    </div>
    <div v-if="filterOptions.length > 1" class="flex flex-wrap gap-1 items-center pb-4">
      <FilterIcon class="text-secondary h-5 w-5 mr-1" />
      <button
        v-for="filter in filterOptions"
        :key="filter"
        :class="`px-2 py-1 rounded-full font-semibold leading-none border-none cursor-pointer active:scale-[0.97] duration-100 transition-all ${selectedFilters.includes(filter.id) ? 'bg-brand-highlight text-brand' : 'bg-bg-raised text-secondary'}`"
        @click="toggleArray(selectedFilters, filter.id)"
      >
        {{ filter.formattedName }}
      </button>
    </div>
    <ContentListPanel
      v-model="selectedFiles"
      :locked="isPackLocked"
      :items="
        search.map((x) => ({
          disabled: x.disabled,
          filename: x.file_name,
          icon: x.icon,
          title: x.name,
          creator: {
            name: x.author,
            type: 'user',
            id: x.author,
            link: 'https://modrinth.com/user/' + x.author,
            linkProps: { target: '_blank' },
          },
          project: {
            id: x.id,
            link: { path: `/project/${x.id}/`, query: { i: props.instance.path } },
            linkProps: {},
          },
          version: x.version,
          versionId: x.version,
          data: x,
        }))
      "
    >
      <template v-if="selectedProjects.length > 0" #headers>
        <div class="flex gap-2">
          <ButtonStyled
            v-if="selectedProjects.some((m) => m.outdated)"
            color="brand"
            color-fill="text"
            hover-color-fill="text"
          >
            <button @click="updateSelected()"><DownloadIcon /> Update</button>
          </ButtonStyled>
          <ButtonStyled>
            <OverflowMenu
              :options="[
                {
                  id: 'share-names',
                  action: () => shareNames(),
                },
                {
                  id: 'share-file-names',
                  action: () => shareFileNames(),
                },
                {
                  id: 'share-urls',
                  action: () => shareUrls(),
                },
                {
                  id: 'share-markdown',
                  action: () => shareMarkdown(),
                },
              ]"
            >
              <ShareIcon /> Share <DropdownIcon />
              <template #share-names> <TextInputIcon /> Project names </template>
              <template #share-file-names> <FileIcon /> File names </template>
              <template #share-urls> <LinkIcon /> Project links </template>
              <template #share-markdown> <CodeIcon /> Markdown links </template>
            </OverflowMenu>
          </ButtonStyled>
          <ButtonStyled v-if="selectedProjects.some((m) => m.disabled)">
            <button @click="enableAll()"><CheckCircleIcon /> Enable</button>
          </ButtonStyled>
          <ButtonStyled v-if="selectedProjects.some((m) => !m.disabled)">
            <button @click="disableAll()"><SlashIcon /> Disable</button>
          </ButtonStyled>
          <ButtonStyled color="red">
            <button @click="deleteSelected()"><TrashIcon /> Remove</button>
          </ButtonStyled>
        </div>
      </template>
      <template #header-actions>
        <ButtonStyled
          v-if="!isPackLocked && projects.some((m) => (m as any).outdated)"
          type="transparent"
          color="brand"
          color-fill="text"
          hover-color-fill="text"
        >
          <button><DownloadIcon /> Update all</button>
        </ButtonStyled>
      </template>
      <template #actions="{ item }">
        <ButtonStyled
          v-if="!isPackLocked && (item.data as any).outdated"
          type="transparent"
          color="brand"
          circular
        >
          <button v-tooltip="`Update`" @click="updateProject(item.data)">
            <DownloadIcon />
          </button>
        </ButtonStyled>
        <div v-else class="w-[36px]"></div>
        <ButtonStyled v-if="!isPackLocked" type="transparent" circular>
          <button
            v-tooltip="item.disabled ? `Enable` : `Disable`"
            @click="toggleDisableMod(item.data)"
          >
            <CheckCircleIcon v-if="item.disabled" />
            <SlashIcon v-else />
          </button>
        </ButtonStyled>
        <ButtonStyled type="transparent" circular>
          <OverflowMenu
            :options="[
              {
                id: 'show-file',
                action: () => {},
              },
              {
                id: 'copy-link',
                shown: item.project !== undefined,
                action: () => toggleDisableMod(item.data),
              },
              {
                divider: true,
                shown: !isPackLocked,
              },
              {
                id: 'remove',
                color: 'red',
                shown: !isPackLocked,
                action: () => removeMod(item),
              },
            ]"
            direction="left"
          >
            <MoreVerticalIcon />
            <template #show-file> <ExternalIcon /> Show file </template>
            <template #copy-link> <ClipboardCopyIcon /> Copy link </template>
            <template v-if="item.disabled" #toggle> <CheckCircleIcon /> Enable </template>
            <template v-else #toggle> <SlashIcon /> Disable </template>
            <template #remove> <TrashIcon /> Remove </template>
          </OverflowMenu>
        </ButtonStyled>
      </template>
    </ContentListPanel>
  </template>
  <div v-else class="w-full flex flex-col items-center justify-center mt-6 max-w-[48rem] mx-auto">
    <div class="top-box w-full">
      <div class="flex items-center gap-6 w-[32rem] mx-auto">
        <img src="@/assets/sad-modrinth-bot.webp" class="h-24" />
        <span class="text-contrast font-bold text-xl"
          >You haven't added any content to this instance yet.</span
        >
      </div>
    </div>
    <div class="top-box-divider"></div>
    <div class="flex items-center gap-6 py-4">
      <AddContentButton v-if="!isPackLocked" :instance="instance" />
    </div>
  </div>
  <div v-if="false && projects.length > 0" class="table bg-bg-raised">
    <div class="table-row table-head" :class="{ 'show-options': selected.length > 0 }">
      <div class="table-cell table-text">
        <Checkbox v-model="selectAll" class="select-checkbox" />
      </div>
      <div v-if="selected.length === 0" class="table-cell table-text name-cell actions-cell">
        <Button class="transparent" @click="sortProjects('Name')">
          Name
          <DropdownIcon v-if="sortColumn === 'Name'" :class="{ down: ascending }" />
        </Button>
      </div>
      <div v-if="selected.length === 0" class="table-cell table-text version">
        <Button class="transparent" @click="sortProjects('Version')">
          Version
          <DropdownIcon v-if="sortColumn === 'Version'" :class="{ down: ascending }" />
        </Button>
      </div>
      <div v-if="selected.length === 0" class="table-cell table-text actions-cell">
        <Button class="transparent" @click="sortProjects('Enabled')">
          Actions
          <DropdownIcon v-if="sortColumn === 'Enabled'" :class="{ down: ascending }" />
        </Button>
      </div>
      <div v-else class="options table-cell name-cell">
        <div>
          <Button
            class="transparent share"
            @click="() => (showingOptions = !showingOptions)"
            @mouseover="selectedOption = 'Share'"
          >
            <MenuIcon :class="{ open: showingOptions }" />
          </Button>
        </div>
        <Button
          class="transparent share"
          @click="shareNames()"
          @mouseover="selectedOption = 'Share'"
        >
          <ShareIcon />
          Share
        </Button>
        <div v-tooltip="isPackLocked ? 'Unlock this instance to remove mods' : ''">
          <Button
            :disabled="isPackLocked"
            class="transparent trash"
            @click="deleteWarning.show()"
            @mouseover="selectedOption = 'Delete'"
          >
            <TrashIcon />
            Delete
          </Button>
        </div>
        <div v-tooltip="isPackLocked ? 'Unlock this instance to update mods' : ''">
          <Button
            :disabled="isPackLocked || offline"
            class="transparent update"
            @click="updateSelected()"
            @mouseover="selectedOption = 'Update'"
          >
            <UpdatedIcon />
            Update
          </Button>
        </div>
        <div v-tooltip="isPackLocked ? 'Unlock this instance to toggle mods' : ''">
          <Button
            :disabled="isPackLocked"
            class="transparent"
            @click="toggleSelected()"
            @mouseover="selectedOption = 'Toggle'"
          >
            <ToggleIcon />
            Toggle
          </Button>
        </div>
      </div>
    </div>
    <div v-if="showingOptions && selected.length > 0" class="more-box">
      <section v-if="selectedOption === 'Share'" class="options">
        <Button class="transparent" @click="shareNames()">
          <TextInputIcon />
          Share names
        </Button>
        <Button class="transparent" @click="shareUrls()">
          <GlobeIcon />
          Share URLs
        </Button>
        <Button class="transparent" @click="shareFileNames()">
          <FileIcon />
          Share file names
        </Button>
        <Button class="transparent" @click="shareMarkdown()">
          <CodeIcon />
          Share as markdown
        </Button>
      </section>
      <section v-if="selectedOption === 'Delete'" class="options">
        <Button class="transparent" @click="deleteWarning.show()">
          <TrashIcon />
          Delete selected
        </Button>
        <Button class="transparent" @click="deleteDisabledWarning.show()">
          <ToggleIcon />
          Delete disabled
        </Button>
      </section>
      <section v-if="selectedOption === 'Update'" class="options">
        <Button class="transparent" :disabled="offline" @click="updateAll()">
          <UpdatedIcon />
          Update all
        </Button>
        <Button class="transparent" @click="selectUpdatable()">
          <CheckIcon />
          Select updatable
        </Button>
      </section>
      <section v-if="selectedOption === 'Toggle'" class="options">
        <Button class="transparent" @click="enableAll()">
          <CheckIcon />
          Toggle on
        </Button>
        <Button class="transparent" @click="disableAll()">
          <XIcon />
          Toggle off
        </Button>
        <Button class="transparent" @click="hideShowAll()">
          <EyeIcon v-if="hideNonSelected" />
          <EyeOffIcon v-else />
          {{ hideNonSelected ? 'Show' : 'Hide' }} untoggled
        </Button>
      </section>
    </div>
    <div
      v-for="mod in search"
      :key="mod.file_name"
      class="table-row"
      @contextmenu.prevent.stop="(c) => handleRightClick(c, mod)"
    >
      <div class="table-cell table-text checkbox">
        <Checkbox
          :model-value="selectionMap.get(mod.path)"
          class="select-checkbox"
          @update:model-value="(newValue) => selectionMap.set(mod.path, newValue)"
        />
      </div>
      <div class="table-cell table-text name-cell">
        <router-link
          v-if="mod.slug"
          :to="{ path: `/project/${mod.slug}/`, query: { i: props.instance.path } }"
          :disabled="offline"
          class="mod-content"
        >
          <Avatar :src="mod.icon" />
          <div class="mod-text">
            <div class="title">{{ mod.name }}</div>
            <span v-if="mod.author" class="no-wrap">by {{ mod.author }}</span>
          </div>
        </router-link>
        <div v-else class="mod-content">
          <Avatar :src="mod.icon" />
          <span v-tooltip="`${mod.name}`" class="title">{{ mod.name }}</span>
        </div>
      </div>
      <div class="table-cell table-text version">
        <span v-tooltip="`${mod.version}`">{{ mod.version }}</span>
      </div>
      <div class="table-cell table-text manage">
        <div v-tooltip="isPackLocked ? 'Unlock this instance to remove mods.' : 'Remove project'">
          <Button :disabled="isPackLocked" icon-only @click="removeMod(mod)">
            <TrashIcon />
          </Button>
        </div>
        <AnimatedLogo v-if="mod.updating" class="btn icon-only updating-indicator" />
        <div
          v-else
          v-tooltip="isPackLocked ? 'Unlock this instance to update mods.' : 'Update project'"
        >
          <Button
            :disabled="!mod.outdated || offline || isPackLocked"
            icon-only
            @click="updateProject(mod)"
          >
            <UpdatedIcon v-if="mod.outdated" />
            <CheckIcon v-else />
          </Button>
        </div>
        <div v-tooltip="isPackLocked ? 'Unlock this instance to toggle mods.' : ''">
          <input
            id="switch-1"
            :disabled="isPackLocked"
            autocomplete="off"
            type="checkbox"
            class="switch stylized-toggle"
            :checked="!mod.disabled"
            @change="toggleDisableMod(mod)"
          />
        </div>
        <Button
          v-tooltip="`Show ${mod.file_name}`"
          icon-only
          @click="highlightModInProfile(instance.path, mod.path)"
        >
          <FolderOpenIcon />
        </Button>
      </div>
    </div>
  </div>
  <div v-else-if="false" class="empty-prompt">
    <div class="empty-icon">
      <AddProjectImage />
    </div>
    <h3>No projects found</h3>
    <p class="empty-subtitle">Add a project to get started</p>
    <AddContentButton :instance="instance" />
  </div>
  <ModalWrapper ref="deleteWarning" header="Are you sure?">
    <div class="modal-body">
      <div class="markdown-body">
        <p>
          Are you sure you want to remove
          <strong>{{ functionValues.length }} project(s)</strong> from {{ instance.name }}?
          <br />
          This action <strong>cannot</strong> be undone.
        </p>
      </div>
      <div class="button-group push-right">
        <Button @click="deleteWarning.hide()"> Cancel </Button>
        <Button color="danger" @click="deleteSelected">
          <TrashIcon />
          Remove
        </Button>
      </div>
    </div>
  </ModalWrapper>
  <ModalWrapper ref="deleteDisabledWarning" header="Are you sure?">
    <div class="modal-body">
      <div class="markdown-body">
        <p>
          Are you sure you want to remove
          <strong
            >{{ Array.from(projects.values()).filter((x) => x.disabled).length }} disabled
            project(s)</strong
          >
          from {{ instance.name }}?
          <br />
          This action <strong>cannot</strong> be undone.
        </p>
      </div>
      <div class="button-group push-right">
        <Button @click="deleteDisabledWarning.hide()"> Cancel </Button>
        <Button color="danger" @click="deleteDisabled">
          <TrashIcon />
          Remove
        </Button>
      </div>
    </div>
  </ModalWrapper>
  <ShareModalWrapper
    ref="shareModal"
    share-title="Sharing modpack content"
    share-text="Check out the projects I'm using in my modpack!"
    :open-in-new-tab="false"
  />
  <ExportModal v-if="projects.length > 0" ref="exportModal" :instance="instance" />
  <ModpackVersionModal
    v-if="instance.linked_data"
    ref="modpackVersionModal"
    :instance="instance"
    :versions="props.versions"
  />
</template>
<script setup lang="ts">
import {
  PlusIcon,
  ExternalIcon,
  LinkIcon,
  LockIcon,
  ClipboardCopyIcon,
  TrashIcon,
  CheckIcon,
  SearchIcon,
  UpdatedIcon,
  FolderOpenIcon,
  XIcon,
  ShareIcon,
  DropdownIcon,
  GlobeIcon,
  FileIcon,
  EyeIcon,
  EyeOffIcon,
  CodeIcon,
  DownloadIcon,
  FilterIcon,
  MoreVerticalIcon,
  CheckCircleIcon,
  SlashIcon,
} from '@modrinth/assets'
import {
  DropdownSelect,
  Checkbox,
  AnimatedLogo,
  Avatar,
  Button,
  Card,
  ButtonStyled,
  ContentListPanel,
  OverflowMenu,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import type { ComputedRef } from 'vue'
import { computed, onUnmounted, ref, watch } from 'vue'
import { useVIntl, defineMessages } from '@vintl/vintl'
import {
  add_project_from_path,
  get_projects,
  remove_project,
  toggle_disable_project,
  update_all,
  update_project,
} from '@/helpers/profile.js'
import { handleError } from '@/store/notifications.js'
import { trackEvent } from '@/helpers/analytics'
import { highlightModInProfile } from '@/helpers/utils.js'
import { MenuIcon, ToggleIcon, TextInputIcon, AddProjectImage, PackageIcon } from '@/assets/icons'
import ExportModal from '@/components/ui/ExportModal.vue'
import ModpackVersionModal from '@/components/ui/ModpackVersionModal.vue'
import AddContentButton from '@/components/ui/AddContentButton.vue'
import {
  get_organization_many,
  get_project_many,
  get_team_many,
  get_version_many,
} from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events.js'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
  options: {
    type: Object,
    default() {
      return {}
    },
  },
  offline: {
    type: Boolean,
    default() {
      return false
    },
  },
  versions: {
    type: Array,
    required: true,
  },
})

const unlistenProfiles = await profile_listener(async (event) => {
  if (
    event.profile_path_id === props.instance.path &&
    event.event === 'synced' &&
    props.instance.install_stage !== 'pack_installing'
  ) {
    await initProjects()
  }
})

onUnmounted(() => {
  unlistenProfiles()
})

const showingOptions = ref(false)
const isPackLocked = computed(() => {
  return props.instance.linked_data && props.instance.linked_data.locked
})
const canUpdatePack = computed(() => {
  if (!props.instance.linked_data || !props.versions || !props.versions[0]) return false
  return props.instance.linked_data.version_id !== props.versions[0].id
})
const exportModal = ref(null)

const projects = ref([])
const selectedFiles = ref([])
const selectedProjects = computed(() =>
  projects.value.filter((x) => selectedFiles.value.includes(x.file_name)),
)

const selectionMap = ref(new Map())

const initProjects = async (cacheBehaviour) => {
  const newProjects = []

  const profileProjects = await get_projects(props.instance.path, cacheBehaviour)
  const fetchProjects = []
  const fetchVersions = []

  for (const value of Object.values(profileProjects)) {
    if (value.metadata) {
      fetchProjects.push(value.metadata.project_id)
      fetchVersions.push(value.metadata.version_id)
    }
  }

  const [modrinthProjects, modrinthVersions] = await Promise.all([
    await get_project_many(fetchProjects).catch(handleError),
    await get_version_many(fetchVersions).catch(handleError),
  ])

  const [modrinthTeams, modrinthOrganizations] = await Promise.all([
    await get_team_many(modrinthProjects.map((x) => x.team)).catch(handleError),
    await get_organization_many(
      modrinthProjects.map((x) => x.organization).filter((x) => !!x),
    ).catch(handleError),
  ])

  for (const [path, file] of Object.entries(profileProjects)) {
    if (file.metadata) {
      const project = modrinthProjects.find((x) => file.metadata.project_id === x.id)
      const version = modrinthVersions.find((x) => file.metadata.version_id === x.id)

      if (project && version) {
        const org = project.organization
          ? modrinthOrganizations.find((x) => x.id === project.organization)
          : null

        const team = modrinthTeams.find((x) => x[0].team_id === project.team)

        let owner

        if (org) {
          owner = org.name
        } else if (team) {
          owner = team.find((x) => x.is_owner).user.username
        } else {
          owner = null
        }

        newProjects.push({
          path,
          name: project.title,
          slug: project.slug,
          author: owner,
          version: version.version_number,
          file_name: file.file_name,
          icon: project.icon_url,
          disabled: file.file_name.endsWith('.disabled'),
          updateVersion: file.update_version_id,
          outdated: !!file.update_version_id,
          project_type: project.project_type,
          id: project.id,
        })
      }

      continue
    }

    newProjects.push({
      path,
      name: file.file_name.replace('.disabled', ''),
      author: '',
      version: null,
      file_name: file.file_name,
      icon: null,
      disabled: file.file_name.endsWith('.disabled'),
      outdated: false,
      project_type: file.project_type,
    })
  }

  projects.value = newProjects

  const newSelectionMap = new Map()
  for (const project of projects.value) {
    newSelectionMap.set(
      project.path,
      selectionMap.value.get(project.path) ??
        selectionMap.value.get(project.path.slice(0, -9)) ??
        selectionMap.value.get(project.path + '.disabled') ??
        false,
    )
  }
  selectionMap.value = newSelectionMap
}
await initProjects()

const modpackVersionModal = ref(null)
const installing = computed(() => props.instance.install_stage !== 'installed')

const vintl = useVIntl()
const { formatMessage } = vintl

type FilterOption = {
  id: string
  formattedName: string
}

const messages = defineMessages({
  updatesAvailableFilter: {
    id: 'instance.filter.updates-available',
    defaultMessage: 'Updates available',
  },
})

const filterOptions: ComputedRef<FilterOption[]> = computed(() => {
  const options: FilterOption[] = []

  const frequency = projects.value.reduce((map, item) => {
    map[item.project_type] = (map[item.project_type] || 0) + 1
    return map
  }, {})

  const types = Object.keys(frequency).sort((a, b) => frequency[b] - frequency[a])

  types.forEach((type) => {
    options.push({
      id: type,
      formattedName: formatProjectType(type) + 's',
    })
  })

  if (!isPackLocked.value && projects.value.some((m) => m.outdated)) {
    options.push({
      id: 'updates',
      formattedName: formatMessage(messages.updatesAvailableFilter),
    })
  }

  return options
})

const selectedFilters = ref([])
const filteredProjects = computed(() => {
  const updatesFilter = selectedFilters.value.includes('updates')

  const typeFilters = selectedFilters.value.filter((filter) => filter !== 'updates')

  return projects.value.filter((project) => {
    return (
      (typeFilters.length === 0 || typeFilters.includes(project.project_type)) &&
      (!updatesFilter || project.outdated)
    )
  })
})

function toggleArray(array, value) {
  if (array.includes(value)) {
    array.splice(array.indexOf(value), 1)
  } else {
    array.push(value)
  }
}

const searchFilter = ref('')
const selectAll = ref(false)
const selectedProjectType = ref('All')
const deleteWarning = ref(null)
const deleteDisabledWarning = ref(null)
const hideNonSelected = ref(false)
const selectedOption = ref('Share')
const shareModal = ref(null)
const ascending = ref(true)
const sortColumn = ref('Name')

const selected = computed(() =>
  Array.from(selectionMap.value)
    .filter((args) => {
      return args[1]
    })
    .map((args) => {
      return projects.value.find((x) => x.path === args[0])
    }),
)

const functionValues = computed(() =>
  selectedProjects.value.length > 0 ? selectedProjects.value : Array.from(projects.value.values()),
)

const selectableProjectTypes = computed(() => {
  const obj = { All: 'all' }

  for (const project of projects.value) {
    obj[project.project_type ? formatProjectType(project.project_type) + 's' : 'Other'] =
      project.project_type
  }

  return obj
})

const search = computed(() => {
  const projectType = selectableProjectTypes.value[selectedProjectType.value]
  const filtered = filteredProjects.value
    .filter((mod) => {
      return (
        mod.name.toLowerCase().includes(searchFilter.value.toLowerCase()) &&
        (projectType === 'all' || mod.project_type === projectType)
      )
    })
    .filter((mod) => {
      if (hideNonSelected.value) {
        return !mod.disabled
      }
      return true
    })

  return updateSort(filtered)
})

const updateSort = (projects) => {
  switch (sortColumn.value) {
    case 'Version':
      return projects.slice().sort((a, b) => {
        if (a.version < b.version) {
          return ascending.value ? -1 : 1
        }
        if (a.version > b.version) {
          return ascending.value ? 1 : -1
        }
        return 0
      })
    case 'Author':
      return projects.slice().sort((a, b) => {
        if (a.author < b.author) {
          return ascending.value ? -1 : 1
        }
        if (a.author > b.author) {
          return ascending.value ? 1 : -1
        }
        return 0
      })
    case 'Enabled':
      return projects.slice().sort((a, b) => {
        if (a.disabled && !b.disabled) {
          return ascending.value ? 1 : -1
        }
        if (!a.disabled && b.disabled) {
          return ascending.value ? -1 : 1
        }
        return 0
      })
    default:
      return projects.slice().sort((a, b) => {
        if (a.name < b.name) {
          return ascending.value ? -1 : 1
        }
        if (a.name > b.name) {
          return ascending.value ? 1 : -1
        }
        return 0
      })
  }
}

const sortProjects = (filter) => {
  if (sortColumn.value === filter) {
    ascending.value = !ascending.value
  } else {
    sortColumn.value = filter
    ascending.value = true
  }
}

const updateAll = async () => {
  const setProjects = []
  for (const [i, project] of projects.value.entries()) {
    if (project.outdated) {
      project.updating = true
      setProjects.push(i)
    }
  }

  const paths = await update_all(props.instance.path).catch(handleError)

  for (const [oldVal, newVal] of Object.entries(paths)) {
    const index = projects.value.findIndex((x) => x.path === oldVal)
    projects.value[index].path = newVal
    projects.value[index].outdated = false

    if (projects.value[index].updateVersion) {
      projects.value[index].version = projects.value[index].updateVersion.version_number
      projects.value[index].updateVersion = null
    }
  }
  for (const project of setProjects) {
    projects.value[project].updating = false
  }

  trackEvent('InstanceUpdateAll', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    count: setProjects.length,
    selected: selected.value.length > 1,
  })
}

const selectUpdatable = () => {
  for (const project of projects.value) {
    if (project.outdated) {
      selectionMap.value.set(project.path, true)
    }
  }
}

const updateProject = async (mod) => {
  mod.updating = true
  await new Promise((resolve) => setTimeout(resolve, 0))
  mod.path = await update_project(props.instance.path, mod.path).catch(handleError)
  mod.updating = false

  mod.outdated = false
  mod.version = mod.updateVersion.version_number
  mod.updateVersion = null

  trackEvent('InstanceProjectUpdate', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    id: mod.id,
    name: mod.name,
    project_type: mod.project_type,
  })
}

const locks = {}

const toggleDisableMod = async (mod) => {
  // Use mod's id as the key for the lock. If mod doesn't have a unique id, replace `mod.id` with some unique property.
  if (!locks[mod.id]) {
    locks[mod.id] = ref(null)
  }

  const lock = locks[mod.id]

  while (lock.value) {
    await lock.value
  }

  lock.value = toggle_disable_project(props.instance.path, mod.path)
    .then((newPath) => {
      mod.path = newPath
      mod.disabled = !mod.disabled
      trackEvent('InstanceProjectDisable', {
        loader: props.instance.loader,
        game_version: props.instance.game_version,
        id: mod.id,
        name: mod.name,
        project_type: mod.project_type,
        disabled: mod.disabled,
      })
    })
    .catch(handleError)
    .finally(() => {
      lock.value = null
    })

  await lock.value
}

const removeMod = async (mod) => {
  await remove_project(props.instance.path, mod.path).catch(handleError)
  projects.value = projects.value.filter((x) => mod.path !== x.path)

  trackEvent('InstanceProjectRemove', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    id: mod.id,
    name: mod.name,
    project_type: mod.project_type,
  })
}

const deleteSelected = async () => {
  for (const project of functionValues.value) {
    await remove_project(props.instance.path, project.path).catch(handleError)
  }

  projects.value = projects.value.filter((x) => !x.selected)
  deleteWarning.value.hide()
}

const deleteDisabled = async () => {
  for (const project of Array.of(projects.value.values().filter((x) => x.disabled))) {
    await remove_project(props.instance.path, project.path).catch(handleError)
  }

  projects.value = projects.value.filter((x) => !x.selected)
  deleteDisabledWarning.value.hide()
}

const shareNames = async () => {
  console.log(functionValues.value)
  await shareModal.value.show(functionValues.value.map((x) => x.name).join('\n'))
}

const shareFileNames = async () => {
  await shareModal.value.show(functionValues.value.map((x) => x.file_name).join('\n'))
}

const shareUrls = async () => {
  await shareModal.value.show(
    functionValues.value
      .filter((x) => x.slug)
      .map((x) => `https://modrinth.com/${x.project_type}/${x.slug}`)
      .join('\n'),
  )
}

const shareMarkdown = async () => {
  await shareModal.value.show(
    functionValues.value
      .map((x) => {
        if (x.slug) {
          return `[${x.name}](https://modrinth.com/${x.project_type}/${x.slug})`
        }
        return x.name
      })
      .join('\n'),
  )
}

const toggleSelected = async () => {
  for (const project of functionValues.value) {
    await toggleDisableMod(project, !project.disabled)
  }
}

const updateSelected = async () => {
  const promises = []
  for (const project of functionValues.value) {
    if (project.outdated) promises.push(updateProject(project))
  }
  await Promise.all(promises).catch(handleError)
}

const enableAll = async () => {
  for (const project of functionValues.value) {
    if (project.disabled) {
      await toggleDisableMod(project, false)
    }
  }
}

const disableAll = async () => {
  for (const project of functionValues.value) {
    if (!project.disabled) {
      await toggleDisableMod(project, false)
    }
  }
}

const hideShowAll = async () => {
  hideNonSelected.value = !hideNonSelected.value
}

const handleRightClick = (event, mod) => {
  if (mod.slug && mod.project_type) {
    props.options.showMenu(
      event,
      {
        link: `https://modrinth.com/${mod.project_type}/${mod.slug}`,
      },
      [{ name: 'open_link' }, { name: 'copy_link' }],
    )
  }
}

watch(selectAll, () => {
  for (const [key, value] of Array.from(selectionMap.value)) {
    if (value !== selectAll.value) {
      selectionMap.value.set(key, selectAll.value)
    }
  }
})

const refreshingProjects = ref(false)
async function refreshProjects() {
  refreshingProjects.value = true
  await initProjects('bypass')
  refreshingProjects.value = false
}

const unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
  if (event.payload.type !== 'drop') return

  for (const file of event.payload.paths) {
    if (file.endsWith('.mrpack')) continue
    await add_project_from_path(props.instance.path, file).catch(handleError)
  }
  await initProjects()
})

onUnmounted(() => {
  unlisten()
})
</script>

<style scoped lang="scss">
.text-input {
  width: 100%;
}

.manage {
  display: flex;
  gap: 0.5rem;
}

.table {
  margin-block-start: 0;
  border-radius: var(--radius-lg);
  border: 2px solid var(--color-bg);
}

.table-row {
  grid-template-columns: min-content 2fr 1fr 13.25rem;

  &.show-options {
    grid-template-columns: min-content auto;

    .options {
      display: flex;
      flex-direction: row;
      align-items: center;
      gap: var(--gap-md);
    }
  }
}

.static {
  .table-row {
    grid-template-areas: 'manage name version';
    grid-template-columns: 4.25rem 1fr 1fr;
  }

  .name-cell {
    grid-area: name;
  }

  .version {
    grid-area: version;
  }

  .manage {
    justify-content: center;
    grid-area: manage;
  }
}

.table-cell {
  align-items: center;
}

.card-row {
  display: flex;
  align-items: center;
  gap: var(--gap-md);
  justify-content: space-between;
  background-color: var(--color-raised-bg);
}

.mod-card {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: var(--gap-sm);
  justify-content: flex-start;
  margin-bottom: 0.5rem;
  white-space: nowrap;
  align-items: center;

  :deep(.dropdown-row) {
    .btn {
      height: 2.5rem !important;
    }
  }

  :deep(.btn) {
    height: 2.5rem;
  }

  .dropdown-input {
    flex-grow: 1;

    .animated-dropdown {
      width: unset;

      :deep(.selected) {
        border-radius: var(--radius-md) 0 0 var(--radius-md);
      }
    }

    .iconified-input {
      width: 100%;

      input {
        flex-basis: unset;
      }
    }

    :deep(.animated-dropdown) {
      .render-down {
        border-radius: var(--radius-md) 0 0 var(--radius-md) !important;
      }

      .options-wrapper {
        margin-top: 0.25rem;
        width: unset;
        border-radius: var(--radius-md);
      }

      .options {
        border-radius: var(--radius-md);
        border: 1px solid var(--color);
      }
    }
  }
}

.list-card {
  margin-top: 0.5rem;
}

.text-combo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.name-cell {
  padding-left: 0;

  .btn {
    margin-left: var(--gap-sm);
    min-width: unset;
  }
}

.dropdown {
  width: 7rem !important;
}

.sort {
  padding-left: 0.5rem;
}

.second-row {
  display: flex;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: var(--gap-sm);

  .chips {
    flex-grow: 1;
  }
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: var(--gap-lg);

  .button-group {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  strong {
    color: var(--color-contrast);
  }
}

.mod-content {
  display: flex;
  align-items: center;
  gap: 1rem;

  .mod-text {
    display: flex;
    flex-direction: column;
  }

  .title {
    color: var(--color-contrast);
    font-weight: bolder;
  }
}

.actions-cell {
  display: flex;
  align-items: center;
  gap: 0.5rem;

  .btn {
    height: unset;
    width: unset;
    padding: 0;

    &.trash {
      color: var(--color-red);
    }

    &.update {
      color: var(--color-green);
    }

    &.share {
      color: var(--color-blue);
    }
  }
}

.more-box {
  display: flex;
  background-color: var(--color-bg);
  padding: var(--gap-lg);

  .options {
    display: flex;
    flex-wrap: wrap;
    flex-direction: row;
    gap: var(--gap-md);
    flex-grow: 1;
  }
}

.btn {
  &.transparent {
    height: unset;
    width: unset;
    padding: 0;
    color: var(--color-base);
    gap: var(--gap-xs);
    white-space: nowrap;

    svg {
      margin-right: 0 !important;
      transition: transform 0.2s ease-in-out;

      &.open {
        transform: rotate(90deg);
      }

      &.down {
        transform: rotate(180deg);
      }
    }
  }
}
.empty-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--gap-md);
  height: 100%;
  width: 100%;
  margin: auto;

  .empty-icon {
    svg {
      width: 10rem;
      height: 10rem;
      color: var(--color-contrast);
    }
  }

  p,
  h3 {
    margin: 0;
  }
}
</style>

<style lang="scss">
.updating-indicator {
  height: 2.25rem !important;
  width: 2.25rem !important;

  svg {
    height: 1.25rem !important;
    width: 1.25rem !important;
  }
}

.select-checkbox {
  button.checkbox {
    border: none;
    margin: 0;
  }
}

.dropdown-input {
  .selected {
    height: 2.5rem;
  }
}

.pagination-after {
  margin-bottom: 5rem;
}

.search-input {
  min-height: 2.25rem;
  background-color: var(--color-raised-bg);
}

.top-box {
  background-image: radial-gradient(
    50% 100% at 50% 100%,
    var(--color-brand-highlight) 10%,
    #ffffff00 100%
  );
}

.top-box-divider {
  background-image: linear-gradient(90deg, #ffffff00 0%, var(--color-brand) 50%, #ffffff00 100%);
  width: 100%;
  height: 1px;
  opacity: 0.8;
}
</style>
