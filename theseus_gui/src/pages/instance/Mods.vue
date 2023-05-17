<template>
  <Card class="mod-card">
    <div class="card-row">
      <div class="iconified-input">
        <SearchIcon />
        <input v-model="searchFilter" type="text" placeholder="Search Mods" class="text-input" />
      </div>
      <span class="manage">
        <span class="text-combo">
          <span class="no-wrap sort"> Sort By </span>
          <DropdownSelect
            v-model="sortFilter"
            name="sort-by"
            :options="['Name', 'Version', 'Author']"
            default-value="Name"
            class="dropdown"
          />
        </span>
        <Button color="primary" @click="searchMod()">
          <PlusIcon />
          <span class="no-wrap"> Add Content </span>
        </Button>
      </span>
    </div>
    <div class="table">
      <div class="table-row table-head">
        <div class="table-cell table-text">
          <Button color="success" icon-only>
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
          <Button v-if="mod.outdated" icon-only>
            <UpdatedIcon />
          </Button>
          <Button v-else disabled icon-only>
            <CheckIcon />
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
          <Button icon-only>
            <TrashIcon />
          </Button>
          <input
            id="switch-1"
            type="checkbox"
            class="switch stylized-toggle"
            :checked="mod.disabled"
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
  CheckIcon,
  SearchIcon,
  UpdatedIcon,
  DropdownSelect,
} from 'omorphia'
import { computed, ref, shallowRef } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { useRouter } from 'vue-router'

const router = useRouter()

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
})

const projects = shallowRef([])
for (const project of Object.values(props.instance.projects)) {
  if (project.metadata.type === 'modrinth') {
    let owner = project.metadata.members.find((x) => x.role === 'Owner')
    projects.value.push({
      name: project.metadata.project.title,
      slug: project.metadata.project.slug,
      author: owner ? owner.user.username : null,
      version: project.metadata.version.version_number,
      file_name: project.file_name,
      icon: project.metadata.project.icon_url,
      disabled: project.disabled,
      outdated: project.metadata.update_version,
    })
  } else if (project.metadata.type === 'inferred') {
    projects.value.push({
      name: project.metadata.title ?? project.file_name,
      author: project.metadata.authors[0],
      version: project.metadata.version,
      file_name: project.file_name,
      icon: project.metadata.icon ? convertFileSrc(project.metadata.icon) : null,
      disabled: project.disabled,
      outdated: false,
    })
  } else {
    projects.value.push({
      name: project.file_name,
      author: '',
      version: null,
      file_name: project.file_name,
      icon: null,
      disabled: project.disabled,
      outdated: false,
    })
  }
}

const searchFilter = ref('')
const sortFilter = ref('')

const search = computed(() => {
  const filtered = projects.value.filter((mod) => {
    return mod.name.toLowerCase().includes(searchFilter.value.toLowerCase())
  })

  return updateSort(filtered, sortFilter.value)
})

const updateSort = (projects, sort) => {
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

const searchMod = () => {
  router.push({ path: '/browse/mod' })
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

.no-wrap {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;

  &.sort {
    padding-left: 0.5rem;
  }
}
</style>
