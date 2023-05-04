<template>
  <AnimatedLogo v-if="loading" />
  <Card v-else class="mod-card">
    <div class="card-row">
      <div class="iconified-input">
        <SearchIcon />
        <input v-model="searchFilter" type="text" placeholder="Search Mods" />
      </div>
      <span class="manage">
        <span class="text-combo">
          Sort By
          <DropdownSelect
            v-model="sortFilter"
            name="sort-by"
            :options="['Name', 'Version', 'Author']"
            default-value="Name"
            class="dropdown"
          />
        </span>
        <Button color="primary">
          <PlusIcon />
          Add Mods
        </Button>
      </span>
    </div>
    <div class="table">
      <div class="table-row table-head">
        <div class="table-cell table-text">
          <Button color="success" icon-only :disabled="allUpdated" @click="updateAll">
            <UpdatedIcon />
          </Button>
        </div>
        <div class="table-cell table-text name-cell">Name</div>
        <div class="table-cell table-text">Version</div>
        <div class="table-cell table-text">Author</div>
        <div class="table-cell table-text">Actions</div>
      </div>
      <div v-for="mod in search" :key="mod.file_name" class="table-row">
        <div class="table-cell table-text">
          <Button v-if="mod.outdated" icon-only @click="() => update(mod)">
            <UpdatedIcon />
          </Button>
          <Button v-else disabled icon-only>
            <CheckCircleIcon />
          </Button>
        </div>
        <div class="table-cell table-text name-cell">
          <router-link v-if="mod.slug" :to="`/project/${mod.slug}/`" class="mod-text">
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
          <Button icon-only @click="() => deleteMod(mod)">
            <TrashIcon />
          </Button>
          <input
            id="switch-1"
            type="checkbox"
            class="switch stylized-toggle"
            :checked="!mod.disabled"
            @change="() => handleDisable(mod)"
          />
        </div>
      </div>
    </div>
  </Card>
</template>
<script setup>
import {
  Avatar,
  Button,
  TrashIcon,
  PlusIcon,
  Card,
  CheckCircleIcon,
  SearchIcon,
  UpdatedIcon,
  DropdownSelect,
  AnimatedLogo,
} from 'omorphia'
import { computed, ref, shallowRef, onUnmounted, watch } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import {
  toggle_disable_project,
  remove_project,
  update_all,
  update_project,
  add_project_from_path,
  get,
} from '@/helpers/profile'
import { listen } from '@tauri-apps/api/event'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
})

const loading = ref(false)

const projects = shallowRef([])
const formatProjects = (projectsToFormat = []) => {
  projects.value = []

  if (projectsToFormat.length === 0) projectsToFormat = Object.values(props.instance.projects)

  for (const project of projectsToFormat) {
    if (project.metadata.type === 'modrinth') {
      let owner = project.metadata.members.find((x) => x.role === 'Owner')
      projects.value.push({
        name: project.metadata.project.title,
        slug: project.metadata.project.slug,
        author: owner ? owner.user.username : null,
        version: project.metadata.version.version_number,
        file_name: project.file_name,
        icon: project.metadata.project.icon_url,
        disabled: project.file_name.includes('.disabled') ? true : false,
        outdated:
          project.metadata.update_version?.version_number !==
          project.metadata.version?.version_number,
      })
    } else if (project.metadata.type === 'inferred') {
      projects.value.push({
        name: project.metadata.title ?? project.file_name,
        author: project.metadata.authors[0],
        version: project.metadata.version,
        file_name: project.file_name,
        icon: project.metadata.icon ? convertFileSrc(project.metadata.icon) : null,
        disabled: project.file_name.includes('.disabled') ? true : false,
        outdated: false,
      })
    } else {
      projects.value.push({
        name: project.file_name,
        author: '',
        version: null,
        file_name: project.file_name,
        icon: null,
        disabled: project.file_name.includes('.disabled') ? true : false,
        outdated: false,
      })
    }
  }
}
formatProjects()

const searchFilter = ref('')
const sortFilter = ref('')

const search = computed(() => {
  const filtered = projects.value.filter((mod) => {
    return mod.name.toLowerCase().includes(searchFilter.value.toLowerCase())
  })

  return updateSort(filtered, sortFilter.value)
})

const allUpdated = computed(() => projects.value.every((p) => p.outdated === false))

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

const getProject = (mod) =>
  Object.keys(props.instance.projects).find((p) => p.includes(mod.file_name))

const updateAll = async () => {
  console.log('firing')
  loading.value = true
  await update_all(props.instance.path)
  loading.value = false
  props.instance.projects.forEach((p) => (p.outdated = false))
  console.log('finished')
}

const update = async (mod) => await update_project(props.instance.path, getProject(mod))

const handleDisable = async (mod) =>
  await toggle_disable_project(props.instance.path, getProject(mod))

const deleteMod = async (mod) => await remove_project(props.instance.path, getProject(mod))

const dropFileListener = await listen('tauri://file-drop', async (e) => {
  // Get the uploaded file(s) from the payload and add it to profile
  // TODO: Don't do this iteratively when a batch add method is added
  await e.payload.forEach(async (mod) => {
    await add_project_from_path(props.instance.path, mod, 'inferred')
  })

  // Get the profile. A get_proj_list_by_profile may be better here when one is made.
  const profile = await get(props.instance.path)
  formatProjects(Object.values(profile.projects))
})

watch(
  () => props.instance,
  (newProps) => {
    console.log(props.instance)
    console.log(newProps)
    formatProjects()
  }
)

onUnmounted(() => {
  dropFileListener()
})
</script>

<style scoped lang="scss">
.manage {
  display: flex;
  gap: 0.5rem;
}

.table-row {
  grid-template-columns: min-content 2fr 1fr 1fr 8rem;
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
</style>
