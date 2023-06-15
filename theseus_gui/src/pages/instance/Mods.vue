<template>
  <Card class="mod-card">
    <div class="card-row">
      <div class="iconified-input">
        <SearchIcon />
        <input
          v-model="searchFilter"
          type="text"
          :placeholder="`Search ${search.length} ${(['All', 'Other'].includes(selectedProjectType)
            ? 'projects'
            : selectedProjectType.toLowerCase()
          ).slice(0, search.length === 1 ? -1 : 64)}...`"
          class="text-input"
          autocomplete="off"
        />
        <Button @click="() => (searchFilter = '')">
          <XIcon />
        </Button>
      </div>
      <span class="manage">
        <span class="text-combo">
          <span class="no-wrap sort"> Sort by </span>
          <DropdownSelect
            v-model="sortFilter"
            name="sort-by"
            :options="['Name', 'Version', 'Author', 'Enabled']"
            default-value="Name"
            class="dropdown"
          />
        </span>
        <DropdownButton
          :options="['search', 'from_file']"
          default-value="search"
          name="add-content-dropdown"
          color="primary"
          @option-click="handleContentOptionClick"
        >
          <template #search>
            <SearchIcon />
            <span class="no-wrap"> Search addons </span>
          </template>
          <template #from_file>
            <FolderOpenIcon />
            <span class="no-wrap"> Add from file </span>
          </template>
        </DropdownButton>
      </span>
    </div>
    <div class="second-row">
      <Chips
        v-if="Object.keys(selectableProjectTypes).length > 1"
        v-model="selectedProjectType"
        :items="Object.keys(selectableProjectTypes)"
      />
      <Button :disabled="!projects.some((x) => x.outdated)" class="no-wrap" @click="updateAll">
        <UpdatedIcon />
        Update {{ selected.length > 0 ? 'selected' : 'all' }}
      </Button>
      <Button v-if="selected.length > 0" class="no-wrap" @click="deleteWarning.show()">
        <TrashIcon />
        Remove selected
      </Button>
      <DropdownButton
        v-if="selected.length > 0"
        :options="['toggle', 'disable', 'enable']"
        default-value="toggle"
        @option-click="toggleSelected"
      >
        <template #toggle>
          <GlobeIcon />
          Toggle selected
        </template>
        <template #disable>
          <EditIcon />
          Disable selected
        </template>
        <template #enable>
          <HashIcon />
          Enable selected
        </template>
      </DropdownButton>
      <DropdownButton
        v-if="selected.length > 0"
        :options="['copy_name', 'copy_url', 'copy_slug']"
        default-value="copy_name"
        @option-click="copySelected"
      >
        <template #copy_name>
          <EditIcon />
          Copy names
        </template>
        <template #copy_slug>
          <HashIcon />
          Copy slugs
        </template>
        <template #copy_url>
          <GlobeIcon />
          Copy URLs
        </template>
      </DropdownButton>
    </div>
    <div class="table">
      <div class="table-row table-head">
        <div class="table-cell table-text">
          <Checkbox v-model="selectAll" class="select-checkbox" />
        </div>
        <div class="table-cell table-text name-cell">Name</div>
        <div class="table-cell table-text">Version</div>
        <div class="table-cell table-text">Author</div>
        <div class="table-cell table-text">Actions</div>
      </div>
      <div
        v-for="mod in search"
        :key="mod.file_name"
        class="table-row"
        @contextmenu.prevent.stop="(c) => handleRightClick(c, mod)"
      >
        <div class="table-cell table-text">
          <Checkbox v-model="mod.selected" class="select-checkbox" />
        </div>
        <div class="table-cell table-text name-cell">
          <router-link
            v-if="mod.slug"
            :to="{ path: `/project/${mod.slug}/`, query: { i: props.instance.path } }"
            class="mod-text"
          >
            <Avatar :src="mod.icon" />
            {{ mod.name }}
          </router-link>
          <div v-else class="mod-text">
            <Avatar :src="mod.icon" />
            {{ mod.name }}
          </div>
        </div>
        <div class="table-cell table-text">{{ mod.version }}</div>
        <div class="table-cell table-text">{{ mod.author }}</div>
        <div class="table-cell table-text manage">
          <Button v-tooltip="'Remove project'" icon-only @click="removeMod(mod)">
            <TrashIcon />
          </Button>
          <AnimatedLogo v-if="mod.updating" class="btn icon-only updating-indicator"></AnimatedLogo>
          <Button
            v-else
            v-tooltip="'Update project'"
            :disabled="!mod.outdated"
            icon-only
            @click="updateProject(mod)"
          >
            <UpdatedIcon v-if="mod.outdated" />
            <CheckIcon v-else />
          </Button>
          <input
            id="switch-1"
            autocomplete="off"
            type="checkbox"
            class="switch stylized-toggle"
            :checked="!mod.disabled"
            @change="toggleDisableMod(mod)"
          />
        </div>
      </div>
    </div>
  </Card>
  <Modal ref="deleteWarning" header="Are you sure?">
    <div class="modal-body">
      <div class="markdown-body">
        <p>
          Are you sure you want to remove <strong>{{ selected.length }} projects</strong> from
          {{ instance.metadata.name }}?
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
  </Modal>
</template>
<script setup>
import {
  Avatar,
  Button,
  TrashIcon,
  Card,
  CheckIcon,
  SearchIcon,
  UpdatedIcon,
  DropdownSelect,
  AnimatedLogo,
  Chips,
  FolderOpenIcon,
  Checkbox,
  formatProjectType,
  DropdownButton,
  EditIcon,
  GlobeIcon,
  HashIcon,
  Modal,
  XIcon,
} from 'omorphia'
import { computed, ref, watch } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { useRouter } from 'vue-router'
import {
  add_project_from_path,
  get,
  remove_project,
  toggle_disable_project,
  update_all,
  update_project,
} from '@/helpers/profile.js'
import { handleError } from '@/store/notifications.js'
import { open } from '@tauri-apps/api/dialog'
import { listen } from '@tauri-apps/api/event'

const router = useRouter()

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
})

const projects = ref([])

const initProjects = (initInstance) => {
  projects.value = []
  for (const [path, project] of Object.entries(initInstance.projects)) {
    if (project.metadata.type === 'modrinth') {
      let owner = project.metadata.members.find((x) => x.role === 'Owner')
      projects.value.push({
        path,
        name: project.metadata.project.title,
        slug: project.metadata.project.slug,
        author: owner ? owner.user.username : null,
        version: project.metadata.version.version_number,
        file_name: project.file_name,
        icon: project.metadata.project.icon_url,
        disabled: project.disabled,
        updateVersion: project.metadata.update_version,
        outdated: !!project.metadata.update_version,
        project_type: project.metadata.project.project_type,
      })
    } else if (project.metadata.type === 'inferred') {
      projects.value.push({
        path,
        name: project.metadata.title ?? project.file_name,
        author: project.metadata.authors[0],
        version: project.metadata.version,
        file_name: project.file_name,
        icon: project.metadata.icon ? convertFileSrc(project.metadata.icon) : null,
        disabled: project.disabled,
        outdated: false,
        project_type: project.metadata.project_type,
      })
    } else {
      projects.value.push({
        path,
        name: project.file_name,
        author: '',
        version: null,
        file_name: project.file_name,
        icon: null,
        disabled: project.disabled,
        outdated: false,
        project_type: null,
      })
    }
  }
}

initProjects(props.instance)

const searchFilter = ref('')
const selectAll = ref(false)
const sortFilter = ref('')
const selectedProjectType = ref('All')
const selected = computed(() => projects.value.filter((mod) => mod.selected))
const deleteWarning = ref(null)

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
  const filtered = projects.value.filter((mod) => {
    return (
      mod.name.toLowerCase().includes(searchFilter.value.toLowerCase()) &&
      (projectType === 'all' || mod.project_type === projectType)
    )
  })

  return updateSort(filtered, sortFilter.value)
})

function updateSort(projects, sort) {
  switch (sort) {
    case 'Version':
      return projects.slice().sort((a, b) => {
        if (a.version < b.version) {
          return -1
        }
        if (a.version > b.version) {
          return 1
        }
        return 0
      })
    case 'Author':
      return projects.slice().sort((a, b) => {
        if (a.author < b.author) {
          return -1
        }
        if (a.author > b.author) {
          return 1
        }
        return 0
      })
    case 'Enabled':
      return projects.slice().sort((a, b) => {
        if (a.disabled && !b.disabled) {
          return 1
        }
        if (!a.disabled && b.disabled) {
          return -1
        }
        return 0
      })
    default:
      return projects.slice().sort((a, b) => {
        if (a.name < b.name) {
          return -1
        }
        if (a.name > b.name) {
          return 1
        }
        return 0
      })
  }
}

async function updateAll() {
  const setProjects = []
  for (const [i, project] of selected.value ?? projects.value.entries()) {
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
}

async function updateProject(mod) {
  mod.updating = true
  mod.path = await update_project(props.instance.path, mod.path).catch(handleError)
  mod.updating = false

  mod.outdated = false
  mod.version = mod.updateVersion.version_number
  mod.updateVersion = null
}

async function toggleDisableMod(mod) {
  mod.path = await toggle_disable_project(props.instance.path, mod.path).catch(handleError)
  mod.disabled = !mod.disabled
}

async function removeMod(mod) {
  await remove_project(props.instance.path, mod.path).catch(handleError)
  projects.value = projects.value.filter((x) => mod.path !== x.path)
}

const handleContentOptionClick = async (args) => {
  if (args.option === 'search') {
    await router.push({
      path: `/browse/${props.instance.metadata.loader === 'vanilla' ? 'datapack' : 'mod'}`,
    })
  } else if (args.option === 'from_file') {
    const newProject = await open({ multiple: true })
    console.log(newProject)
    if (!newProject) return

    for (const project of newProject) {
      console.log(project)
      await add_project_from_path(props.instance.path, project, 'mod').catch(handleError)
      initProjects(await get(props.instance.path).catch(handleError))
    }
  }
}

listen('tauri://file-drop', async (event) => {
  for (const file of event.payload) {
    await add_project_from_path(props.instance.path, file, 'mod').catch(handleError)
    initProjects(await get(props.instance.path).catch(handleError))
  }
})

async function deleteSelected() {
  for (const project of selected.value) {
    await remove_project(props.instance.path, project.path).catch(handleError)
  }
  projects.value = projects.value.filter((x) => !x.selected)
  deleteWarning.value.hide()
}

async function copySelected(args) {
  switch (args.option) {
    case 'copy_name':
      await navigator.clipboard.writeText(selected.value.map((x) => x.name).join('\n'))
      break
    case 'copy_slug':
      await navigator.clipboard.writeText(
        selected.value
          .filter((x) => x.slug)
          .map((x) => x.slug)
          .join('\n')
      )
      break
    case 'copy_url':
      await navigator.clipboard.writeText(
        selected.value
          .filter((x) => x.slug)
          .map((x) => `https://modrinth.com/${x.project_type}/${x.slug}`)
          .join('\n')
      )
      break
  }
}

async function toggleSelected(args) {
  switch (args.option) {
    case 'toggle':
      for (const project of selected.value) {
        await toggleDisableMod(project)
      }
      break
    case 'enable':
      for (const project of selected.value) {
        if (project.disabled) {
          await toggleDisableMod(project)
        }
      }
      break
    case 'disable':
      for (const project of selected.value) {
        if (!project.disabled) {
          await toggleDisableMod(project)
        }
      }
      break
  }
}

watch(selectAll, () => {
  for (const project of projects.value) {
    project.selected = selectAll.value
  }
})

const handleRightClick = (event, mod) => {
  if (mod.slug && mod.project_type) {
    props.options.showMenu(
      event,
      {
        link: `https://modrinth.com/${mod.project_type}/${mod.slug}`,
      },
      [{ name: 'open_link' }, { name: 'copy_link' }]
    )
  }
}
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
}

.table-row {
  grid-template-columns: min-content 2fr 1fr 1fr 11rem;
}

.table-cell {
  align-items: center;
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
}

.text-combo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.name-cell {
  padding-left: 0;
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
</style>
<style lang="scss">
.updating-indicator {
  svg {
    margin-left: 0.5rem !important;
  }
}

.v-popper--theme-tooltip .v-popper__inner {
  background: #fff !important;
}

.select-checkbox {
  button.checkbox {
    border: none;
  }
}
</style>
