<template>
  <div>
    <template v-if="projects?.length > 0">
      <div class="flex items-center gap-2 mb-4">
        <div class="iconified-input flex-grow">
          <SearchIcon />
          <input
            v-model="searchFilter"
            type="text"
            :placeholder="`Search ${filteredProjects.length} project${filteredProjects.length === 1 ? '' : 's'}...`"
            class="text-input search-input"
            autocomplete="off"
          />
          <Button class="r-btn" @click="() => (searchFilter = '')">
            <XIcon />
          </Button>
        </div>
        <AddContentButton :instance="instance" />
      </div>
      <div class="flex items-center justify-between">
        <div v-if="filterOptions.length > 1" class="flex flex-wrap gap-1 items-center pb-4">
          <FilterIcon class="text-secondary h-5 w-5 mr-1" />
          <button
            v-for="filter in filterOptions"
            :key="`content-filter-${filter.id}`"
            :class="`px-2 py-1 rounded-full font-semibold leading-none border-none cursor-pointer active:scale-[0.97] duration-100 transition-all ${selectedFilters.includes(filter.id) ? 'bg-brand-highlight text-brand' : 'bg-bg-raised text-secondary'}`"
            @click="toggleArray(selectedFilters, filter.id)"
          >
            {{ filter.formattedName }}
          </button>
        </div>
        <Pagination
          v-if="search.length > 0"
          :page="currentPage"
          :count="Math.ceil(search.length / 20)"
          :link-function="(page) => `?page=${page}`"
          @switch-page="(page) => (currentPage = page)"
        />
      </div>

      <ContentListPanel
        v-model="selectedFiles"
        :locked="isPackLocked"
        :items="
          search.map((x) => {
            const item: ContentItem<any> = {
              path: x.path,
              disabled: x.disabled,
              filename: x.file_name,
              icon: x.icon ?? undefined,
              title: x.name,
              data: x,
            }

            if (x.version) {
              item.version = x.version
              item.versionId = x.version
            }

            if (x.id) {
              item.project = {
                id: x.id,
                link: { path: `/project/${x.id}`, query: { i: props.instance.path } },
                linkProps: {},
              }
            }

            if (x.author) {
              item.creator = {
                name: x.author.name,
                type: x.author.type,
                id: x.author.slug,
                link: `https://modrinth.com/${x.author.type}/${x.author.slug}`,
                linkProps: { target: '_blank' },
              }
            }

            return item
          })
        "
        :sort-column="sortColumn"
        :sort-ascending="ascending"
        :update-sort="sortProjects"
        :current-page="currentPage"
      >
        <template v-if="selectedProjects.length > 0" #headers>
          <div class="flex gap-2">
            <ButtonStyled
              v-if="!isPackLocked && selectedProjects.some((m) => m.outdated)"
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
          <ButtonStyled type="transparent" color-fill="text" hover-color-fill="text">
            <button :disabled="refreshingProjects" class="w-max" @click="refreshProjects">
              <UpdatedIcon />
              Refresh
            </button>
          </ButtonStyled>
          <ButtonStyled
            v-if="!isPackLocked && projects.some((m) => (m as any).outdated)"
            type="transparent"
            color="brand"
            color-fill="text"
            hover-color-fill="text"
            @click="updateAll"
          >
            <button class="w-max"><DownloadIcon /> Update all</button>
          </ButtonStyled>
          <ButtonStyled
            v-if="canUpdatePack"
            type="transparent"
            color="brand"
            color-fill="text"
            hover-color-fill="text"
          >
            <button class="w-max" :disabled="installing" @click="modpackVersionModal?.show()">
              <DownloadIcon /> Update pack
            </button>
          </ButtonStyled>
        </template>
        <template #actions="{ item }">
          <ButtonStyled
            v-if="!isPackLocked && (item.data as any).outdated"
            type="transparent"
            color="brand"
            circular
          >
            <button
              v-tooltip="`Update`"
              :disabled="(item.data as ProjectListEntry).updating"
              @click="updateProject(item.data)"
            >
              <DownloadIcon />
            </button>
          </ButtonStyled>
          <div v-else class="w-[36px]"></div>
          <Toggle
            class="!mx-2"
            :model-value="!item.data.disabled"
            @update:model-value="toggleDisableMod(item.data)"
          />
          <ButtonStyled type="transparent" circular>
            <button v-tooltip="'Remove'" @click="removeMod(item)">
              <TrashIcon />
            </button>
          </ButtonStyled>

          <ButtonStyled type="transparent" circular>
            <OverflowMenu
              :options="[
                {
                  id: 'show-file',
                  action: () => highlightModInProfile(instance.path, item.path),
                },
                {
                  id: 'copy-link',
                  shown: item.data !== undefined && item.data.slug !== undefined,
                  action: () => copyModLink(item),
                },
              ]"
              direction="left"
            >
              <MoreVerticalIcon />
              <template #show-file> <ExternalIcon /> Show file </template>
              <template #copy-link> <ClipboardCopyIcon /> Copy link </template>
            </OverflowMenu>
          </ButtonStyled>
        </template>
      </ContentListPanel>
      <div class="flex justify-end mt-4">
        <Pagination
          v-if="search.length > 0"
          :page="currentPage"
          :count="Math.ceil(search.length / 20)"
          :link-function="(page) => `?page=${page}`"
          @switch-page="(page) => (currentPage = page)"
        />
      </div>
    </template>
    <div v-else class="w-full max-w-[48rem] mx-auto flex flex-col mt-6">
      <RadialHeader class="">
        <div class="flex items-center gap-6 w-[32rem] mx-auto">
          <img src="@/assets/sad-modrinth-bot.webp" class="h-24" />
          <span class="text-contrast font-bold text-xl"
            >You haven't added any content to this instance yet.</span
          >
        </div>
      </RadialHeader>
      <div class="flex mt-4 mx-auto">
        <AddContentButton :instance="instance" />
      </div>
    </div>
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
  </div>
</template>
<script setup lang="ts">
import {
  CheckCircleIcon,
  ClipboardCopyIcon,
  CodeIcon,
  DownloadIcon,
  DropdownIcon,
  ExternalIcon,
  FileIcon,
  FilterIcon,
  LinkIcon,
  MoreVerticalIcon,
  SearchIcon,
  ShareIcon,
  SlashIcon,
  TrashIcon,
  UpdatedIcon,
  XIcon,
} from '@modrinth/assets'
import {
  Button,
  ButtonStyled,
  ContentListPanel,
  OverflowMenu,
  Pagination,
  RadialHeader,
  Toggle,
} from '@modrinth/ui'
import type { Organization, Project, TeamMember, Version } from '@modrinth/utils'
import { formatProjectType } from '@modrinth/utils'
import type { ComputedRef } from 'vue'
import { computed, onUnmounted, ref, watch } from 'vue'
import { defineMessages, useVIntl } from '@vintl/vintl'
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
import { TextInputIcon } from '@/assets/icons'
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
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import dayjs from 'dayjs'
import type { CacheBehaviour, ContentFile, GameInstance } from '@/helpers/types'
import type ContextMenu from '@/components/ui/ContextMenu.vue'
import type { ContentItem } from '@modrinth/ui/src/components/content/ContentListItem.vue'

const props = defineProps<{
  instance: GameInstance
  options: InstanceType<typeof ContextMenu>
  offline: boolean
  playing: boolean
  versions: Version[]
  installed: boolean
}>()

type ProjectListEntryAuthor = {
  name: string
  slug: string
  type: 'user' | 'organization'
}

type ProjectListEntry = {
  path: string
  name: string
  slug?: string
  author: ProjectListEntryAuthor | null
  version: string | null
  file_name: string
  icon: string | undefined
  disabled: boolean
  updateVersion?: string
  outdated: boolean
  updated: dayjs.Dayjs
  project_type: string
  id?: string
  updating?: boolean
  selected?: boolean
}

const isPackLocked = computed(() => {
  return props.instance.linked_data && props.instance.linked_data.locked
})
const canUpdatePack = computed(() => {
  if (!props.instance.linked_data || !props.versions || !props.versions[0]) return false
  return props.instance.linked_data.version_id !== props.versions[0].id
})
const exportModal = ref(null)

const projects = ref<ProjectListEntry[]>([])
const selectedFiles = ref<string[]>([])
const selectedProjects = computed(() =>
  projects.value.filter((x) => selectedFiles.value.includes(x.file_name)),
)

const selectionMap = ref(new Map())

const initProjects = async (cacheBehaviour?: CacheBehaviour) => {
  const newProjects: ProjectListEntry[] = []

  const profileProjects = (await get_projects(props.instance.path, cacheBehaviour)) as Record<
    string,
    ContentFile
  >
  const fetchProjects = []
  const fetchVersions = []

  for (const value of Object.values(profileProjects)) {
    if (value.metadata) {
      fetchProjects.push(value.metadata.project_id)
      fetchVersions.push(value.metadata.version_id)
    }
  }

  const [modrinthProjects, modrinthVersions] = await Promise.all([
    (await get_project_many(fetchProjects).catch(handleError)) as Project[],
    (await get_version_many(fetchVersions).catch(handleError)) as Version[],
  ])

  const [modrinthTeams, modrinthOrganizations] = await Promise.all([
    (await get_team_many(modrinthProjects.map((x) => x.team)).catch(handleError)) as TeamMember[][],
    (await get_organization_many(
      modrinthProjects.map((x) => x.organization).filter((x) => !!x),
    ).catch(handleError)) as Organization[],
  ])

  for (const [path, file] of Object.entries(profileProjects)) {
    if (file.metadata) {
      const project = modrinthProjects.find((x) => file.metadata?.project_id === x.id)
      const version = modrinthVersions.find((x) => file.metadata?.version_id === x.id)

      if (project && version) {
        const org = project.organization
          ? modrinthOrganizations.find((x) => x.id === project.organization)
          : null

        const team = modrinthTeams.find((x) => x[0].team_id === project.team)

        let author: ProjectListEntryAuthor | null = null
        if (org) {
          author = {
            name: org.name,
            slug: org.slug,
            type: 'organization',
          }
        } else if (team) {
          const teamMember = team.find((x) => x.is_owner)
          if (teamMember) {
            author = {
              name: teamMember.user.username,
              slug: teamMember.user.username,
              type: 'user',
            }
          }
        }

        newProjects.push({
          path,
          name: project.title,
          slug: project.slug,
          author,
          version: version.version_number,
          file_name: file.file_name,
          icon: project.icon_url,
          disabled: file.file_name.endsWith('.disabled'),
          updateVersion: file.update_version_id,
          updated: dayjs(version.date_published),
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
      author: null,
      version: null,
      file_name: file.file_name,
      icon: undefined,
      disabled: file.file_name.endsWith('.disabled'),
      outdated: false,
      updated: dayjs(0),
      project_type: file.project_type === 'shaderpack' ? 'shader' : file.project_type,
    })
  }

  projects.value = newProjects ?? []

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

const modpackVersionModal = ref<InstanceType<typeof ModpackVersionModal> | null>()
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
  disabledFilter: {
    id: 'instance.filter.disabled',
    defaultMessage: 'Disabled projects',
  },
})

const filterOptions: ComputedRef<FilterOption[]> = computed(() => {
  const options: FilterOption[] = []

  const frequency = projects.value.reduce((map: Record<string, number>, item) => {
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

  if (projects.value.some((m) => m.disabled)) {
    options.push({
      id: 'disabled',
      formattedName: formatMessage(messages.disabledFilter),
    })
  }

  return options
})

const selectedFilters = ref<string[]>([])
const filteredProjects = computed(() => {
  const updatesFilter = selectedFilters.value.includes('updates')
  const disabledFilter = selectedFilters.value.includes('disabled')

  const typeFilters = selectedFilters.value.filter(
    (filter) => filter !== 'updates' && filter !== 'disabled',
  )

  return projects.value.filter((project) => {
    return (
      (typeFilters.length === 0 || typeFilters.includes(project.project_type)) &&
      (!updatesFilter || project.outdated) &&
      (!disabledFilter || project.disabled)
    )
  })
})

watch(filterOptions, () => {
  for (let i = 0; i < selectedFilters.value.length; i++) {
    const option = selectedFilters.value[i]
    if (!filterOptions.value.some((x) => x.id === option)) {
      selectedFilters.value.splice(i, 1)
    }
  }
})

function toggleArray<T>(array: T[], value: T) {
  if (array.includes(value)) {
    array.splice(array.indexOf(value), 1)
  } else {
    array.push(value)
  }
}

const searchFilter = ref('')
const selectAll = ref(false)
const shareModal = ref<InstanceType<typeof ShareModalWrapper> | null>()
const ascending = ref(true)
const sortColumn = ref('Name')
const currentPage = ref(1)

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

const search = computed(() => {
  const filtered = filteredProjects.value.filter((mod) => {
    return mod.name.toLowerCase().includes(searchFilter.value.toLowerCase())
  })

  switch (sortColumn.value) {
    case 'Updated':
      return filtered.slice().sort((a, b) => {
        const updated = a.updated.isAfter(b.updated) ? 1 : -1
        return ascending.value ? -updated : updated
      })
    default:
      return filtered
        .slice()
        .sort((a, b) =>
          ascending.value ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name),
        )
  }
})

watch([sortColumn, ascending, selectedFilters.value, searchFilter], () => (currentPage.value = 1))

const sortProjects = (filter: string) => {
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

  const paths = (await update_all(props.instance.path).catch(handleError)) as Record<string, string>

  for (const [oldVal, newVal] of Object.entries(paths)) {
    const index = projects.value.findIndex((x) => x.path === oldVal)
    projects.value[index].path = newVal
    projects.value[index].outdated = false

    if (projects.value[index].updateVersion) {
      projects.value[index].version = projects.value[index].updateVersion.version_number
      projects.value[index].updateVersion = undefined
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

const updateProject = async (mod: ProjectListEntry) => {
  mod.updating = true
  await new Promise((resolve) => setTimeout(resolve, 0))
  mod.path = await update_project(props.instance.path, mod.path).catch(handleError)
  mod.updating = false

  mod.outdated = false
  mod.version = mod.updateVersion?.version_number
  mod.updateVersion = undefined

  trackEvent('InstanceProjectUpdate', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    id: mod.id,
    name: mod.name,
    project_type: mod.project_type,
  })
}

const locks: Record<string, string | null> = {}

const toggleDisableMod = async (mod: ProjectListEntry) => {
  // Use mod's id as the key for the lock. If mod doesn't have a unique id, replace `mod.id` with some unique property.
  const lock = locks[mod.file_name]

  while (lock) {
    await new Promise((resolve) => {
      setTimeout((value: unknown) => resolve(value), 100)
    })
  }

  locks[mod.file_name] = 'lock'

  try {
    mod.path = await toggle_disable_project(props.instance.path, mod.path)
    mod.disabled = !mod.disabled

    trackEvent('InstanceProjectDisable', {
      loader: props.instance.loader,
      game_version: props.instance.game_version,
      id: mod.id,
      name: mod.name,
      project_type: mod.project_type,
      disabled: mod.disabled,
    })
  } catch (err) {
    handleError(err)
  }

  locks[mod.file_name] = null
}

const removeMod = async (mod: ContentItem<ProjectListEntry>) => {
  await remove_project(props.instance.path, mod.path).catch(handleError)
  projects.value = projects.value.filter((x) => mod.path !== x.path)

  trackEvent('InstanceProjectRemove', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
    id: mod.data.id,
    name: mod.data.name,
    project_type: mod.data.project_type,
  })
}

const copyModLink = async (mod: ContentItem<ProjectListEntry>) => {
  await navigator.clipboard.writeText(
    `https://modrinth.com/${mod.data.project_type}/${mod.data.slug}`,
  )
}

const deleteSelected = async () => {
  for (const project of functionValues.value) {
    await remove_project(props.instance.path, project.path).catch(handleError)
  }

  projects.value = projects.value.filter((x) => !x.selected)
}

const shareNames = async () => {
  await shareModal.value?.show(functionValues.value.map((x) => x.name).join('\n'))
}

const shareFileNames = async () => {
  await shareModal.value?.show(functionValues.value.map((x) => x.file_name).join('\n'))
}

const shareUrls = async () => {
  await shareModal.value?.show(
    functionValues.value
      .filter((x) => x.slug)
      .map((x) => `https://modrinth.com/${x.project_type}/${x.slug}`)
      .join('\n'),
  )
}

const shareMarkdown = async () => {
  await shareModal.value?.show(
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

const updateSelected = async () => {
  const promises = []
  for (const project of functionValues.value) {
    if (project.outdated) promises.push(updateProject(project))
  }
  await Promise.all(promises).catch(handleError)
}

const enableAll = async () => {
  const promises = []
  for (const project of functionValues.value) {
    if (project.disabled) {
      promises.push(toggleDisableMod(project))
    }
  }
  await Promise.all(promises).catch(handleError)
}

const disableAll = async () => {
  const promises = []
  for (const project of functionValues.value) {
    if (!project.disabled) {
      promises.push(toggleDisableMod(project))
    }
  }
  await Promise.all(promises).catch(handleError)
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

const unlistenProfiles = await profile_listener(
  async (event: { event: string; profile_path_id: string }) => {
    if (
      event.profile_path_id === props.instance.path &&
      event.event === 'synced' &&
      props.instance.install_stage !== 'pack_installing'
    ) {
      await initProjects()
    }
  },
)

onUnmounted(() => {
  unlisten()
  unlistenProfiles()
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
.select-checkbox {
  button.checkbox {
    border: none;
    margin: 0;
  }
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
