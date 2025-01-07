<template>
  <template v-if="projects.length > 0">

    <div class="mb-3">
      <NavTabs
        :tabs="[
        {
          label: formatMessage(ANY_PROJECT_TYPE_MESSAGE),
          action: () => {
            projectType = null
          },
          isSelected: () => projectType === null,
        },
        ...availableProjectTypes.map((type) => ({
          label: formatMessage(getProjectTypeMessage(type)),
          action: () => projectType = type,
          isSelected: () => projectType === type,
        })),
      ]"
        class="mb-3"
      />
    </div>
    <div class="mb-3 flex gap-2">
      <div class="iconified-input flex-1">
        <SearchIcon />
        <input v-model="searchInput" type="text" placeholder="Search" />
        <Button class="r-btn" @click="() => (searchInput = '')">
          <XIcon />
        </Button>
      </div>
      <DropdownSelect
        v-slot="{ selected }"
        v-model="sortBy"
        name="Sort Dropdown"
        class="max-w-[16rem]"
        :options="sortOptions"
        :default-value="sortOptions[0]"
        :display-name="(option) => (option.message ? formatMessage(option.message) : 'unknown')"
      >
        <span class="font-semibold text-primary">Sort by: </span>
        <span class="font-semibold text-secondary">{{ selected }}</span>
      </DropdownSelect>
    </div>
    <div class="flex flex-col gap-3">
      <div v-for="project in filteredProjects" :key="`project-list-${project.id}`">
        <NewProjectCard
          :project="project"
          :link="projectLink(project)"
          :platform-tags="platformTags"
          :experimental-colors="experimentalColors"
          :date-type="sortBy.id === 'published' ? 'published' : 'updated'"
        >
          <template #actions>
            <slot name="project-actions" :project="project" />
          </template>
        </NewProjectCard>
      </div>
      <div v-if="filteredProjects.length === 0" class="flex flex-col gap-2 mt-3 items-center">
        <p class="text-lg text-contrast font-bold m-0">
          {{ formatMessage(messages.noProjectsFound, {
          project_type: formatMessage(getProjectTypeMessage(projectType, 'body'), { count: 0 }),
          search_query: searchInput
        }) }}
        </p>
        <ButtonStyled>
          <button @click="searchInput = ''">
            <XCircleIcon /> {{ formatMessage(messages.clearSearchQuery) }}
          </button>
        </ButtonStyled>
      </div>
    </div>
  </template>
  <p v-else class="text-lg text-center text-contrast font-bold m-0 mt-6">
    {{ formatMessage(messages.noProjects) }}
  </p>
</template>
<script setup lang="ts">
import { useVIntl, defineMessages, defineMessage } from '@vintl/vintl'
import { XCircleIcon, SearchIcon, XIcon } from '@modrinth/assets'
import type { PlatformTag, Project, VirtualProjectType } from '@modrinth/utils'
import NewProjectCard from './NewProjectCard.vue'
import { ref, computed, type Ref } from 'vue'
import { Button, DropdownSelect } from '../index'
import type { Linkish } from '../../utils/link'
import {
  ANY_PROJECT_TYPE_MESSAGE,
  getProjectTypeMessage
} from '../../utils/project-types'
import NavTabs from '../base/NavTabs.vue'
import dayjs from 'dayjs'
import ButtonStyled from '../base/ButtonStyled.vue'

const { formatMessage } = useVIntl()

const sortOptions = [
  {
    id: 'downloads',
    message: defineMessage({ id: 'project.list.sort.downloads', defaultMessage: 'Downloads' }),
    sortFunction: (a: Project, b: Project) => b.downloads - a.downloads,
  },
  {
    id: 'followers',
    message: defineMessage({ id: 'project.list.sort.followers', defaultMessage: 'Followers' }),
    sortFunction: (a: Project, b: Project) => b.followers - a.followers,
  },
  {
    id: 'published',
    message: defineMessage({ id: 'project.list.sort.published', defaultMessage: 'Date published' }),
    sortFunction: (a: Project, b: Project) =>
      dayjs(b.approved).valueOf() - dayjs(a.approved).valueOf(),
  },
  {
    id: 'updated',
    message: defineMessage({ id: 'project.list.sort.updated', defaultMessage: 'Date updated' }),
    sortFunction: (a: Project, b: Project) =>
      dayjs(b.updated).valueOf() - dayjs(a.updated).valueOf(),
  },
]

const sortBy = ref(sortOptions[0])

const props = withDefaults(
  defineProps<{
    projects: Project[]
    projectLink: (project: Project) => Linkish
    platformTags?: PlatformTag[]
    experimentalColors?: boolean
  }>(),
  {
    platformTags: undefined,
    experimentalColors: false,
  },
)

const searchInput = ref('')

const sortedProjects = computed(() => props.projects.slice().sort(sortBy.value.sortFunction))

const projectType: Ref<VirtualProjectType | null> = ref(null)

const availableProjectTypes = computed(() => {
  const types = new Set<VirtualProjectType>()

  for (const project of sortedProjects.value) {
    types.add(project.project_type)
  }

  return [...types]
})

const filteredProjects = computed(() =>
  sortedProjects.value.filter(
    (project) =>
      project.title.toLowerCase().includes(searchInput.value.toLowerCase()) &&
      (projectType.value === null || project.project_type === projectType.value),
  ),
)

const messages = defineMessages({
  noProjectsFound: {
    id: 'project.list.no-projects-found',
    defaultMessage: 'No {project_type} found matching "{search_query}"',
  },
  noProjects: {
    id: 'project.list.no-projects',
    defaultMessage: 'This user has no projects.',
  },
  clearSearchQuery: {
    id: 'project.list.clear-search-query',
    defaultMessage: 'Clear search query',
  },
})
</script>
