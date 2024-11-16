<script setup>
import { computed, nextTick, readonly, ref, shallowRef, Teleport, watch } from 'vue'
import {
  ClearIcon,
  ClientIcon,
  GameIcon,
  LeftArrowIcon,
  SearchIcon,
  ServerIcon,
  XIcon,
} from '@modrinth/assets'
import {
  Avatar,
  BrowseFiltersPanel,
  Button,
  ButtonStyled,
  Checkbox,
  DropdownSelect,
  LoadingIndicator,
  Pagination,
  SearchFilter,
} from '@modrinth/ui'
import { formatCategory, formatCategoryHeader } from '@modrinth/utils'
import Multiselect from 'vue-multiselect'
import { handleError } from '@/store/state'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import { useRoute, useRouter } from 'vue-router'
import SearchCard from '@/components/ui/SearchCard.vue'
import { get as getInstance, get_projects as getInstanceProjects } from '@/helpers/profile.js'
import { convertFileSrc } from '@tauri-apps/api/core'
import { get_search_results } from '@/helpers/cache.js'
import { debounce } from '@/helpers/utils.js'
import NavTabs from '@/components/ui/NavTabs.vue'

const router = useRouter()
const route = useRoute()

const filterAccordions = ref([])

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

const breadcrumbs = useBreadcrumbs()
breadcrumbs.setContext({ name: 'Discover content', link: route.path, query: route.query })

const loading = ref(false)
const query = ref('')
const facets = ref([])
const orFacets = ref([])
const negativeFacets = ref([])
const selectedVersions = ref([])
const onlyOpenSource = ref(false)
const showSnapshots = ref(false)
const hideAlreadyInstalled = ref(false)
const selectedEnvironments = ref([])
const sortTypes = readonly([
  { display: 'Relevance', name: 'relevance' },
  { display: 'Downloads', name: 'downloads' },
  { display: 'Followers', name: 'follows' },
  { display: 'Date published', name: 'newest' },
  { display: 'Date updated', name: 'updated' },
])
const sortType = ref(sortTypes[0])
const maxResults = ref(20)
const currentPage = ref(1)
const projectType = ref(route.params.projectType)
const instanceContext = ref(null)
const instanceProjects = ref(null)
const ignoreInstanceLoaders = ref(false)
const ignoreInstanceGameVersions = ref(false)

const results = shallowRef([])
const pageCount = computed(() =>
  results.value ? Math.ceil(results.value.total_hits / results.value.limit) : 1,
)

function getArrayOrString(x) {
  if (typeof x === 'string' || x instanceof String) {
    return [x]
  } else {
    return x
  }
}

if (route.query.iv) {
  ignoreInstanceGameVersions.value = route.query.iv === 'true'
}
if (route.query.il) {
  ignoreInstanceLoaders.value = route.query.il === 'true'
}
if (route.query.i) {
  ;[instanceContext.value, instanceProjects.value] = await Promise.all([
    getInstance(route.query.i).catch(handleError),
    getInstanceProjects(route.query.i).catch(handleError),
  ])
}
if (route.query.q) {
  query.value = route.query.q
}
if (route.query.f) {
  facets.value = getArrayOrString(route.query.f)
}
if (route.query.g) {
  orFacets.value = getArrayOrString(route.query.g)
}
if (route.query.nf) {
  negativeFacets.value = getArrayOrString(route.query.nf)
}
if (route.query.v) {
  selectedVersions.value = getArrayOrString(route.query.v)
}
if (route.query.l) {
  onlyOpenSource.value = route.query.l === 'true'
}
if (route.query.h) {
  showSnapshots.value = route.query.h === 'true'
}
if (route.query.e) {
  selectedEnvironments.value = getArrayOrString(route.query.e)
}
if (route.query.s) {
  sortType.value.name = route.query.s

  switch (sortType.value.name) {
    case 'relevance':
      sortType.value.display = 'Relevance'
      break
    case 'downloads':
      sortType.value.display = 'Downloads'
      break
    case 'newest':
      sortType.value.display = 'Recently published'
      break
    case 'updated':
      sortType.value.display = 'Recently updated'
      break
    case 'follows':
      sortType.value.display = 'Follow count'
      break
  }
}

if (route.query.m) {
  maxResults.value = route.query.m
}
if (route.query.o) {
  currentPage.value = Math.ceil(route.query.o / maxResults.value) + 1
}
if (route.query.ai) {
  hideAlreadyInstalled.value = route.query.ai === 'true'
}

async function refreshSearch() {
  const params = [`limit=${maxResults.value}`, `index=${sortType.value.name}`]
  if (query.value.length > 0) {
    params.push(`query=${query.value.replace(/ /g, '+')}`)
  }
  if (instanceContext.value) {
    if (!ignoreInstanceLoaders.value && projectType.value === 'mod') {
      orFacets.value = [`categories:${encodeURIComponent(instanceContext.value.loader)}`]
    }
    if (!ignoreInstanceGameVersions.value) {
      selectedVersions.value = [instanceContext.value.game_version]
    }
  }
  if (
    facets.value.length > 0 ||
    orFacets.value.length > 0 ||
    negativeFacets.value.length > 0 ||
    selectedVersions.value.length > 0 ||
    selectedEnvironments.value.length > 0 ||
    projectType.value
  ) {
    let formattedFacets = []
    for (const facet of facets.value) {
      formattedFacets.push([facet])
    }

    for (const facet of negativeFacets.value) {
      formattedFacets.push([facet.replace(':', '!=')])
    }

    // loaders specifier
    if (orFacets.value.length > 0) {
      formattedFacets.push(orFacets.value)
    } else if (projectType.value === 'mod') {
      formattedFacets.push(
        ['forge', 'fabric', 'quilt', 'neoforge'].map(
          (x) => `categories:'${encodeURIComponent(x)}'`,
        ),
      )
    } else if (projectType.value === 'datapack') {
      formattedFacets.push(['datapack'].map((x) => `categories:'${encodeURIComponent(x)}'`))
    }

    if (selectedVersions.value.length > 0) {
      const versionFacets = []
      for (const facet of selectedVersions.value) {
        versionFacets.push('versions:' + facet)
      }
      formattedFacets.push(versionFacets)
    }
    if (onlyOpenSource.value) {
      formattedFacets.push(['open_source:true'])
    }

    if (selectedEnvironments.value.length > 0) {
      let environmentFacets = []
      const includesClient = selectedEnvironments.value.includes('client')
      const includesServer = selectedEnvironments.value.includes('server')
      if (includesClient && includesServer) {
        environmentFacets = [['client_side:required'], ['server_side:required']]
      } else {
        if (includesClient) {
          environmentFacets = [
            ['client_side:optional', 'client_side:required'],
            ['server_side:optional', 'server_side:unsupported'],
          ]
        }
        if (includesServer) {
          environmentFacets = [
            ['client_side:optional', 'client_side:unsupported'],
            ['server_side:optional', 'server_side:required'],
          ]
        }
      }

      formattedFacets = [...formattedFacets, ...environmentFacets]
    }

    if (projectType.value) {
      formattedFacets.push([
        `project_type:${projectType.value === 'datapack' ? 'mod' : projectType.value}`,
      ])
    }

    if (hideAlreadyInstalled.value) {
      const installedMods = Object.values(instanceProjects.value)
        .filter((x) => x.metadata)
        .map((x) => x.metadata.project_id)

      installedMods.map((x) => [`project_id != ${x}`]).forEach((x) => formattedFacets.push(x))
    }

    params.push(`facets=${JSON.stringify(formattedFacets)}`)
  }
  const offset = (currentPage.value - 1) * maxResults.value
  if (currentPage.value !== 1) {
    params.push(`offset=${offset}`)
  }
  let url = ''
  if (params.length > 0) {
    for (let i = 0; i < params.length; i++) {
      url += i === 0 ? `?${params[i]}` : `&${params[i]}`
    }
  }

  let rawResults = await get_search_results(`${url}`)
  if (!rawResults) {
    rawResults = {
      result: {
        hits: [],
        total_hits: 0,
        limit: 1,
      },
    }
  }
  if (instanceContext.value) {
    for (const val of rawResults.result.hits) {
      val.installed = Object.values(instanceProjects.value).some(
        (x) => x.metadata && x.metadata.project_id === val.project_id,
      )
    }
  }
  results.value = rawResults.result
}

async function onSearchChange(newPageNumber) {
  currentPage.value = newPageNumber

  if (query.value === null) {
    return
  }
  await refreshSearch()
  const obj = getSearchUrl((currentPage.value - 1) * maxResults.value, true)

  // Only replace in router if the query is different
  if (JSON.stringify(obj) != JSON.stringify(route.query)) {
    await router.replace({ path: route.path, query: obj })
    breadcrumbs.setContext({ name: 'Discover content', link: route.path, query: obj })
  }
}

const debouncedSearchChange = debounce(() => onSearchChange(1), 200)

const searchWrapper = ref(null)
async function onSearchChangeToTop(newPageNumber) {
  await onSearchChange(newPageNumber)
  await nextTick()
  searchWrapper.value.scrollTo({ top: 0, behavior: 'smooth' })
}

async function clearSearch() {
  query.value = ''
  await onSearchChange(1)
}

function getSearchUrl(offset, useObj) {
  const queryItems = []
  const obj = {}

  if (query.value) {
    queryItems.push(`q=${encodeURIComponent(query.value)}`)
    obj.q = query.value
  }
  if (offset > 0) {
    queryItems.push(`o=${offset}`)
    obj.o = offset
  }
  if (facets.value.length > 0) {
    queryItems.push(`f=${encodeURIComponent(facets.value)}`)
    obj.f = facets.value
  }
  if (orFacets.value.length > 0) {
    queryItems.push(`g=${encodeURIComponent(orFacets.value)}`)
    obj.g = orFacets.value
  }
  if (negativeFacets.value.length > 0) {
    queryItems.push(`nf=${encodeURIComponent(negativeFacets.value)}`)
    obj.nf = negativeFacets.value
  }
  if (selectedVersions.value.length > 0) {
    queryItems.push(`v=${encodeURIComponent(selectedVersions.value)}`)
    obj.v = selectedVersions.value
  }
  if (onlyOpenSource.value) {
    queryItems.push('l=true')
    obj.l = true
  }
  if (showSnapshots.value) {
    queryItems.push('h=true')
    obj.h = true
  }
  if (selectedEnvironments.value.length > 0) {
    queryItems.push(`e=${encodeURIComponent(selectedEnvironments.value)}`)
    obj.e = selectedEnvironments.value
  }
  if (sortType.value.name !== 'relevance') {
    queryItems.push(`s=${encodeURIComponent(sortType.value.name)}`)
    obj.s = sortType.value.name
  }
  if (maxResults.value !== 20) {
    queryItems.push(`m=${encodeURIComponent(maxResults.value)}`)
    obj.m = maxResults.value
  }
  if (instanceContext.value) {
    queryItems.push(`i=${encodeURIComponent(instanceContext.value.path)}`)
    obj.i = instanceContext.value.path
  }
  if (ignoreInstanceGameVersions.value) {
    queryItems.push('iv=true')
    obj.iv = true
  }
  if (ignoreInstanceLoaders.value) {
    queryItems.push('il=true')
    obj.il = true
  }
  if (hideAlreadyInstalled.value) {
    queryItems.push('ai=true')
    obj.ai = true
  }

  let url = `${route.path}`

  if (queryItems.length > 0) {
    url += `?${queryItems[0]}`

    for (let i = 1; i < queryItems.length; i++) {
      url += `&${queryItems[i]}`
    }
  }

  return useObj ? obj : url
}

const sortedCategories = computed(() => {
  const values = new Map()
  for (const category of categories.value.filter(
    (cat) => cat.project_type === (projectType.value === 'datapack' ? 'mod' : projectType.value),
  )) {
    if (!values.has(category.header)) {
      values.set(category.header, [])
    }
    values.get(category.header).push(category)
  }
  return values
})

// Sorts alphabetically, but correctly identifies 8x, 128x, 256x, etc
// identifier[0], then if it ties, identifier[1], etc
async function sortByNameOrNumber(sortable, identifiers) {
  sortable.sort((a, b) => {
    for (const identifier of identifiers) {
      const aNum = parseFloat(a[identifier])
      const bNum = parseFloat(b[identifier])
      if (isNaN(aNum) && isNaN(bNum)) {
        // Both are strings, sort alphabetically
        const stringComp = a[identifier].localeCompare(b[identifier])
        if (stringComp != 0) return stringComp
      } else if (!isNaN(aNum) && !isNaN(bNum)) {
        // Both are numbers, sort numerically
        const numComp = aNum - bNum
        if (numComp != 0) return numComp
      } else {
        // One is a number and one is a string, numbers go first
        const numStringComp = isNaN(aNum) ? 1 : -1
        if (numStringComp != 0) return numStringComp
      }
    }
    return 0
  })
  return sortable
}

async function clearFilters() {
  for (const facet of [...facets.value]) {
    await toggleFacet(facet, true)
  }
  for (const facet of [...orFacets.value]) {
    await toggleOrFacet(facet, true)
  }
  for (const facet of [...negativeFacets.value]) {
    await toggleNegativeFacet(facet, true)
  }
  onlyOpenSource.value = false
  selectedVersions.value = []
  selectedEnvironments.value = []
  await onSearchChangeToTop(1)
}

async function toggleFacet(elementName, doNotSendRequest = false) {
  const index = facets.value.indexOf(elementName)

  if (index !== -1) {
    facets.value.splice(index, 1)
  } else {
    facets.value.push(elementName)
  }

  if (!doNotSendRequest) {
    await onSearchChangeToTop(1)
  }
}

async function toggleOrFacet(elementName, doNotSendRequest) {
  const index = orFacets.value.indexOf(elementName)
  if (index !== -1) {
    orFacets.value.splice(index, 1)
  } else {
    orFacets.value.push(elementName)
  }

  if (!doNotSendRequest) {
    await onSearchChangeToTop(1)
  }
}

async function toggleNegativeFacet(elementName, doNotSendRequest) {
  const index = negativeFacets.value.indexOf(elementName)
  if (index !== -1) {
    negativeFacets.value.splice(index, 1)
  } else {
    negativeFacets.value.push(elementName)
  }

  if (!doNotSendRequest) {
    await onSearchChangeToTop(1)
  }
}

function toggleEnv(environment, sendRequest) {
  const index = selectedEnvironments.value.indexOf(environment)
  if (index !== -1) {
    selectedEnvironments.value.splice(index, 1)
  } else {
    selectedEnvironments.value.push(environment)
  }

  if (!sendRequest) {
    onSearchChangeToTop(1)
  }
}

watch(
  () => route.params.projectType,
  async (newType) => {
    // Check if the newType is not the same as the current value
    if (!newType || newType === projectType.value) return

    projectType.value = newType
    breadcrumbs.setContext({ name: 'Discover content', link: `/browse/${projectType.value}` })

    sortType.value = { display: 'Relevance', name: 'relevance' }
    query.value = ''

    loading.value = true
    await clearFilters()
    loading.value = false
  },
)

const [categories, loaders, availableGameVersions] = await Promise.all([
  get_categories()
    .catch(handleError)
    .then((s) => sortByNameOrNumber(s, ['header', 'name']))
    .then(ref),
  get_loaders().catch(handleError).then(ref),
  get_game_versions().catch(handleError).then(ref),
  refreshSearch(),
])

const filteredLoaders = computed(() => {
  return loaders.value.filter((loader) => {
    return projectType.value === 'mod' || projectType.value === 'modpack'
      ? loader.supported_project_types[0] === 'mod'
      : loader.supported_project_types[0] === projectType.value
  })
})

const selectableProjectTypes = computed(() => {
  let dataPacks = false,
    mods = false,
    modpacks = false

  if (instanceContext.value) {
    if (
      availableGameVersions.value.findIndex(
        (x) => x.version === instanceContext.value.game_version,
      ) <= availableGameVersions.value.findIndex((x) => x.version === '1.13')
    ) {
      dataPacks = true
    }

    if (instanceContext.value.loader !== 'vanilla') {
      mods = true
    }
  } else {
    dataPacks = true
    mods = true
    modpacks = true
  }

  return [
    { label: 'Modpacks', href: `/browse/modpack`, shown: modpacks },
    { label: 'Mods', href: `/browse/mod`, shown: mods },
    { label: 'Resource Packs', href: `/browse/resourcepack` },
    { label: 'Data Packs', href: `/browse/datapack`, shown: dataPacks },
    { label: 'Shaders', href: `/browse/shader` },
  ]
})

const showVersions = computed(
  () => instanceContext.value === null || ignoreInstanceGameVersions.value,
)

const isModProject = computed(() => ['modpack', 'mod'].includes(projectType.value))

function filterSelected(filter) {
  if (filter.type === 'or') {
    return orFacets.value.includes(filter.facet)
  } else if (filter.type === 'normal') {
    return facets.value.includes(filter.facet)
  } else if (filter.type === 'env') {
    return selectedEnvironments.value.includes(filter.name)
  } else if (filter.type === 'gameVersion') {
    return selectedVersions.value.includes(filter.name)
  } else if (filter.type === 'license') {
    return onlyOpenSource.value
  }
}

function negativeFilterSelected(filter) {
  if (filter.type === 'or' || filter.type === 'normal') {
    return negativeFacets.value.includes(filter.facet)
  }
}

function toggleNegativeFilter(filter) {
  const elementName = filter.facet

  if (filterSelected(filter)) {
    if (filter.type === 'or') {
      const index = orFacets.value.indexOf(elementName)
      orFacets.value.splice(index, 1)
    } else if (filter.type === 'normal') {
      const index = facets.value.indexOf(elementName)
      facets.value.splice(index, 1)
    }
  }

  if (filter.type === 'or' || filter.type === 'normal') {
    const index = negativeFacets.value.indexOf(elementName)
    if (index !== -1) {
      negativeFacets.value.splice(index, 1)
    } else {
      negativeFacets.value.push(elementName)
    }
  }

  onSearchChange(1)
}

function toggleFilter(filter, doNotSendRequest) {
  const elementName = filter.facet

  if (negativeFilterSelected(filter)) {
    const index = negativeFacets.value.indexOf(elementName)
    negativeFacets.value.splice(index, 1)
  }

  if (filter.type === 'or') {
    const index = orFacets.value.indexOf(elementName)
    if (index !== -1) {
      orFacets.value.splice(index, 1)
    } else {
      if (elementName === 'categories:purpur') {
        if (!orFacets.value.includes('categories:paper')) {
          orFacets.value.push('categories:paper')
        }
        if (!orFacets.value.includes('categories:spigot')) {
          orFacets.value.push('categories:spigot')
        }
        if (!orFacets.value.includes('categories:bukkit')) {
          orFacets.value.push('categories:bukkit')
        }
      } else if (elementName === 'categories:paper') {
        if (!orFacets.value.includes('categories:spigot')) {
          orFacets.value.push('categories:spigot')
        }
        if (!orFacets.value.includes('categories:bukkit')) {
          orFacets.value.push('categories:bukkit')
        }
      } else if (elementName === 'categories:spigot') {
        if (!orFacets.value.includes('categories:bukkit')) {
          orFacets.value.push('categories:bukkit')
        }
      } else if (elementName === 'categories:waterfall') {
        if (!orFacets.value.includes('categories:bungeecord')) {
          orFacets.value.push('categories:bungeecord')
        }
      }
      orFacets.value.push(elementName)
    }
  } else if (filter.type === 'normal') {
    const index = facets.value.indexOf(elementName)

    if (index !== -1) {
      facets.value.splice(index, 1)
    } else {
      facets.value.push(elementName)
    }
  } else if (filter.type === 'env') {
    const index = selectedEnvironments.value.indexOf(filter.name)
    if (index !== -1) {
      selectedEnvironments.value.splice(index, 1)
    } else {
      selectedEnvironments.value.push(filter.name)
    }
  } else if (filter.type === 'gameVersion') {
    const index = selectedVersions.value.indexOf(filter.name)
    if (index !== -1) {
      selectedVersions.value.splice(index, 1)
    } else {
      selectedVersions.value.push(filter.name)
    }
  } else if (filter.type === 'license') {
    onlyOpenSource.value = !onlyOpenSource.value
  }

  if (!doNotSendRequest) {
    onSearchChange(1)
  }
}
</script>

<template>
  <Teleport to="#sidebar-teleport-target">
    <div v-if="instanceContext" class="small-instance p-4">
      TODO: remove
      <router-link :to="`/instance/${encodeURIComponent(instanceContext.path)}`" class="instance">
        <Avatar
          :src="instanceContext.icon_path ? convertFileSrc(instanceContext.icon_path) : null"
          :alt="instanceContext.name"
          size="sm"
        />
        <div class="small-instance_info">
          <span class="title">{{
            instanceContext.name.length > 20
              ? instanceContext.name.substring(0, 20) + '...'
              : instanceContext.name
          }}</span>
          <span>
            {{ instanceContext.loader.charAt(0).toUpperCase() + instanceContext.loader.slice(1) }}
            {{ instanceContext.game_version }}
          </span>
        </div>
      </router-link>
      <Checkbox
        v-model="ignoreInstanceGameVersions"
        label="Override game versions"
        class="filter-checkbox"
        @update:model-value="onSearchChangeToTop(1)"
        @click.prevent.stop
      />
      <Checkbox
        v-model="ignoreInstanceLoaders"
        label="Override loaders"
        class="filter-checkbox"
        @update:model-value="onSearchChangeToTop(1)"
        @click.prevent.stop
      />
      <Checkbox
        v-model="hideAlreadyInstalled"
        label="Hide already installed"
        class="filter-checkbox"
        @update:model-value="onSearchChangeToTop(1)"
        @click.prevent.stop
      />
    </div>
    <BrowseFiltersPanel
      class="border-0 border-b-[1px] last:border-b-0 border-[--brand-gradient-border] border-solid"
      button-class="button-animation flex p-4 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg"
      content-class="mb-3"
      :game-versions="availableGameVersions"
      :platforms="
        loaders.map((x) => ({
          ...x,
          formatted_name: formatCategory(x.name),
          default: !['rift', 'liteloader', 'modloader'].includes(x.name),
        }))
      "
    >
      <template #header="{ filter }">
        <h3 class="text-lg m-0">{{ filter.formatted_name }}</h3>
      </template>
      <template #option="{ option }">
        <button
          class="px-4 py-1 flex items-center gap-2 bg-transparent border-none cursor-pointer hover:bg-button-bg w-full button-animation"
        >
          <div
            v-if="option.data.icon"
            class="contents text-sm text-secondary"
            v-html="option.data.icon"
          />
          <span class="font-medium text-sm text-secondary">{{ option.formatted_name }}</span>
        </button>
      </template>
    </BrowseFiltersPanel>
    <!--    <div class="p-4 border-0 border-b-[1px] border-[&#45;&#45;brand-gradient-border] border-solid">-->
    <!--      <h3 class="text-lg m-0">Game version</h3>-->
    <!--    </div>-->
    <!--    <div class="p-4 border-0 border-b-[1px] border-[&#45;&#45;brand-gradient-border] border-solid">-->
    <!--      <h3 class="text-lg m-0">Environment</h3>-->
    <!--    </div>-->
    <!--    <div class="p-4 border-0 border-[&#45;&#45;brand-gradient-border] border-solid">-->
    <!--      <h3 class="text-lg m-0">Category</h3>-->
    <!--    </div>-->
    <!--    <template v-if="false">-->
    <!--      <div-->
    <!--        v-for="(categories, header, index) in filters"-->
    <!--        :key="header"-->
    <!--        :class="`border-0 border-b border-solid border-button-bg py-2 last:border-b-0`"-->
    <!--      >-->
    <!--        <button-->
    <!--          class="flex !w-full bg-transparent border-none px-0 py-2 font-extrabold text-contrast transition-all active:scale-[0.98]"-->
    <!--          @click="-->
    <!--              () => {-->
    <!--                filterAccordions[index].isOpen-->
    <!--                  ? filterAccordions[index].close()-->
    <!--                  : filterAccordions[index].open();-->
    <!--              }-->
    <!--            "-->
    <!--        >-->
    <!--          <template v-if="header === 'gameVersion'"> Game versions </template>-->
    <!--          <template v-else>-->
    <!--            {{ formatCategoryHeader(header) }}-->
    <!--          </template>-->
    <!--          <DropdownIcon-->
    <!--            class="ml-auto h-5 w-5 transition-transform"-->
    <!--            :class="{ 'rotate-180': filterAccordions[index]?.isOpen }"-->
    <!--          />-->
    <!--        </button>-->
    <!--        <Accordion ref="filterAccordions" :open-by-default="true">-->
    <!--          <ScrollablePanel-->
    <!--            :class="{ 'h-[18rem]': categories.length >= 8 && header === 'gameVersion' }"-->
    <!--            :no-max-height="header !== 'gameVersion'"-->
    <!--          >-->
    <!--            <div class="mr-1 flex flex-col gap-1">-->
    <!--              <div v-for="category in categories" :key="category.name" class="group flex gap-1">-->
    <!--                <button-->
    <!--                  :class="`flex !w-full border-none items-center gap-2 truncate rounded-xl px-2 py-1 text-sm font-semibold transition-all active:scale-[0.98] ${filterSelected(category) ? 'bg-brand-highlight text-contrast hover:brightness-125' : negativeFilterSelected(category) ? 'bg-highlight-red text-contrast hover:brightness-125' : 'bg-transparent text-secondary hover:bg-button-bg'}`"-->
    <!--                  @click="-->
    <!--                      negativeFilterSelected(category)-->
    <!--                        ? toggleNegativeFilter(category)-->
    <!--                        : toggleFilter(category)-->
    <!--                    "-->
    <!--                >-->
    <!--                  <ClientIcon v-if="category.name === 'client'" class="h-4 w-4" />-->
    <!--                  <ServerIcon v-else-if="category.name === 'server'" class="h-4 w-4" />-->
    <!--                  <div v-if="category.icon" class="h-4" v-html="category.icon" />-->
    <!--                  <span class="truncate text-sm">{{ formatCategory(category.name) }}</span>-->
    <!--                  <BanIcon-->
    <!--                    v-if="negativeFilterSelected(category)"-->
    <!--                    :class="`ml-auto h-4 w-4 shrink-0 transition-opacity group-hover:opacity-100 ${negativeFilterSelected(category) ? '' : 'opacity-0'}`"-->
    <!--                    aria-hidden="true"-->
    <!--                  />-->
    <!--                  <CheckIcon-->
    <!--                    v-else-->
    <!--                    :class="`ml-auto h-4 w-4 shrink-0 transition-opacity group-hover:opacity-100 ${filterSelected(category) ? '' : 'opacity-0'}`"-->
    <!--                    aria-hidden="true"-->
    <!--                  />-->
    <!--                </button>-->
    <!--                <button-->
    <!--                  v-if="-->
    <!--                      (category.type === 'or' || category.type === 'normal') &&-->
    <!--                      !negativeFilterSelected(category)-->
    <!--                    "-->
    <!--                  v-tooltip="negativeFilterSelected(category) ? 'Include' : 'Exclude'"-->
    <!--                  class="flex items-center border-none justify-center gap-2 rounded-xl bg-transparent px-2 py-1 text-sm font-semibold text-secondary opacity-0 transition-all hover:bg-button-bg hover:text-red active:scale-[0.96] group-hover:opacity-100"-->
    <!--                  @click="toggleNegativeFilter(category)"-->
    <!--                >-->
    <!--                  <BanIcon class="h-4 w-4" aria-hidden="true" />-->
    <!--                </button>-->
    <!--              </div>-->
    <!--            </div>-->
    <!--          </ScrollablePanel>-->
    <!--          <Checkbox-->
    <!--            v-if="header === 'gameVersion'"-->
    <!--            v-model="showSnapshots"-->
    <!--            class="mx-2"-->
    <!--            :label="`Show all versions`"-->
    <!--          />-->
    <!--        </Accordion>-->
    <!--      </div>-->
    <!--    </template>-->
    <div v-if="false" class="search-panel-card">
      <Button
        role="button"
        :disabled="
          onlyOpenSource === false &&
          selectedEnvironments.length === 0 &&
          selectedVersions.length === 0 &&
          facets.length === 0 &&
          orFacets.length === 0
        "
        @click="clearFilters"
      >
        <ClearIcon /> Clear filters
      </Button>
      <div
        v-if="
          (isModProject && (ignoreInstanceLoaders || !instanceContext)) || projectType === 'shader'
        "
        class="loaders"
      >
        <h2>Loaders</h2>
        <div v-for="loader in filteredLoaders" :key="loader">
          <SearchFilter
            :active-filters="orFacets"
            :icon="loader.icon"
            :display-name="formatCategory(loader.name)"
            :facet-name="`categories:${encodeURIComponent(loader.name)}`"
            class="filter-checkbox"
            @toggle="toggleOrFacet"
          />
        </div>
      </div>
      <div v-if="showVersions" class="versions">
        <h2>Minecraft versions</h2>
        <Checkbox v-model="showSnapshots" class="filter-checkbox" label="Include snapshots" />
        <multiselect
          v-model="selectedVersions"
          :options="
            showSnapshots
              ? availableGameVersions.map((x) => x.version)
              : availableGameVersions
                  .filter((it) => it.version_type === 'release')
                  .map((x) => x.version)
          "
          :multiple="true"
          :searchable="true"
          :show-no-results="false"
          :close-on-select="false"
          :clear-search-on-select="false"
          :show-labels="false"
          placeholder="Choose versions..."
          @update:model-value="onSearchChangeToTop(1)"
        />
      </div>
      <div
        v-for="categoryList in Array.from(sortedCategories)"
        :key="categoryList[0]"
        class="categories"
      >
        <h2>{{ formatCategoryHeader(categoryList[0]) }}</h2>
        <div v-for="category in categoryList[1]" :key="category.name">
          <SearchFilter
            :active-filters="facets"
            :icon="category.icon"
            :display-name="formatCategory(category.name)"
            :facet-name="`categories:${encodeURIComponent(category.name)}`"
            class="filter-checkbox"
            @toggle="toggleFacet"
          />
        </div>
      </div>
      <div v-if="isModProject" class="environment">
        <h2>Environments</h2>
        <SearchFilter
          :active-filters="selectedEnvironments"
          display-name="Client"
          facet-name="client"
          class="filter-checkbox"
          @toggle="toggleEnv"
        >
          <ClientIcon aria-hidden="true" />
        </SearchFilter>
        <SearchFilter
          :active-filters="selectedEnvironments"
          display-name="Server"
          facet-name="server"
          class="filter-checkbox"
          @toggle="toggleEnv"
        >
          <ServerIcon aria-hidden="true" />
        </SearchFilter>
      </div>
      <div class="open-source">
        <h2>Open source</h2>
        <Checkbox
          v-model="onlyOpenSource"
          label="Open source only"
          class="filter-checkbox"
          @update:model-value="onSearchChangeToTop(1)"
        />
      </div>
    </div>
  </Teleport>
  <div ref="searchWrapper" class="flex flex-col gap-3 p-6">
    <template v-if="instanceContext">
      <div
        class="flex justify-between items-center border-0 border-b border-solid border-button-bg pb-4"
      >
        <router-link
          :to="`/instance/${encodeURIComponent(instanceContext.path)}`"
          tabindex="-1"
          class="flex flex-col gap-4 text-primary"
        >
          <span class="flex items-center gap-2">
            <Avatar
              :src="instanceContext.icon_path ? convertFileSrc(instanceContext.icon_path) : null"
              :alt="instanceContext.name"
              size="48px"
            />
            <span class="flex flex-col gap-2">
              <span class="font-extrabold bold text-contrast">
                {{ instanceContext.name }}
              </span>
              <span class="text-secondary flex items-center gap-2 font-semibold">
                <GameIcon class="h-5 w-5 text-secondary" />
                {{ formatCategory(instanceContext.loader) }} {{ instanceContext.game_version }}
              </span>
            </span>
          </span>
        </router-link>
        <ButtonStyled>
          <router-link :to="`/instance/${encodeURIComponent(instanceContext.path)}`">
            <LeftArrowIcon /> Back to instance
          </router-link>
        </ButtonStyled>
      </div>
      <h1 class="m-0 mb-1 text-xl">Install content to instance</h1>
    </template>
    <h1 v-else class="m-0 mb-1 text-2xl">Discover content</h1>
    <NavTabs :links="selectableProjectTypes" />
    <div class="iconified-input">
      <SearchIcon aria-hidden="true" class="text-lg" />
      <input
        v-model="query"
        class="h-12"
        autocomplete="off"
        spellcheck="false"
        type="text"
        :placeholder="`Search ${projectType}s...`"
        @input="debouncedSearchChange()"
      />
      <Button class="r-btn" @click="() => clearSearch()">
        <XIcon />
      </Button>
    </div>
    <div class="flex gap-2">
      <DropdownSelect
        v-slot="{ selected }"
        v-model="sortType"
        class="max-w-[16rem]"
        name="Sort by"
        :options="sortTypes"
        :display-name="(option) => option?.display"
        @change="onSearchChange(1)"
      >
        <span class="font-semibold text-primary">Sort by: </span>
        <span class="font-semibold text-secondary">{{ selected }}</span>
      </DropdownSelect>
      <DropdownSelect
        v-slot="{ selected }"
        v-model="maxResults"
        name="Max results"
        :options="[5, 10, 15, 20, 50, 100]"
        :default-value="maxResults"
        :model-value="maxResults"
        class="max-w-[14rem]"
        @change="onSearchChange(1)"
      >
        <span class="font-semibold text-primary">View: </span>
        <span class="font-semibold text-secondary">{{ selected }}</span>
      </DropdownSelect>
      <Pagination
        :page="currentPage"
        :count="pageCount"
        :link-function="(x) => getSearchUrl(x <= 1 ? 0 : (x - 1) * maxResults)"
        class="ml-auto"
        @switch-page="onSearchChange"
      />
    </div>
    <div class="search">
      <section v-if="loading" class="offline">
        <LoadingIndicator />
      </section>
      <section v-else-if="offline && results.total_hits === 0" class="offline">
        You are currently offline. Connect to the internet to browse Modrinth!
      </section>
      <section v-else class="project-list display-mode--list instance-results" role="list">
        <SearchCard
          v-for="result in results.hits"
          :key="result?.project_id"
          :project="result"
          :instance="instanceContext"
          :categories="[
            ...categories.filter(
              (cat) =>
                result?.display_categories.includes(cat.name) && cat.project_type === projectType,
            ),
            ...loaders.filter(
              (loader) =>
                result?.display_categories.includes(loader.name) &&
                loader.supported_project_types?.includes(projectType),
            ),
          ]"
          :installed="result.installed"
        />
      </section>
      <div class="flex justify-end">
        <pagination
          :page="currentPage"
          :count="pageCount"
          :link-function="(x) => getSearchUrl(x <= 1 ? 0 : (x - 1) * maxResults)"
          class="pagination-after"
          @switch-page="onSearchChangeToTop"
        />
      </div>
    </div>
  </div>
</template>

<style src="vue-multiselect/dist/vue-multiselect.css"></style>
<style lang="scss">
.small-instance {
  min-height: unset !important;

  .instance {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;

    .title {
      font-weight: 600;
      color: var(--color-contrast);
    }
  }

  .small-instance_info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    justify-content: space-between;
    padding: 0.25rem 0;
  }
}

.filter-checkbox {
  margin-bottom: 0.3rem;
  font-size: 1rem;

  svg {
    display: flex;
    align-self: center;
    justify-self: center;
  }

  button.checkbox {
    border: none;
  }
}
</style>
<style lang="scss" scoped>
.project-type-dropdown {
  width: 100% !important;
}

.promotion {
  margin-top: 1rem;
}

.project-type-container {
  display: flex;
  flex-direction: column;
  width: 100%;
}

.search-panel-card {
  display: flex;
  flex-direction: column;
  margin-bottom: 0 !important;
  min-height: min-content !important;
}

.iconified-input {
  input {
    max-width: none !important;
    flex-basis: auto;
  }
}

.search-panel-container {
  display: inline-flex;
  flex-direction: row;
  align-items: center;
  flex-wrap: wrap;
  width: 100%;
  padding: 1rem !important;
  white-space: nowrap;
  gap: 1rem;

  .inline-option {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;

    .sort-dropdown {
      max-width: 12.25rem;
    }

    .limit-dropdown {
      width: 5rem;
    }
  }

  .iconified-input {
    flex-grow: 1;
  }

  .filter-panel {
    button {
      display: flex;
      align-items: center;
      justify-content: space-evenly;

      svg {
        margin-right: 0.4rem;
      }
    }
  }
}

.search-container {
  display: flex;
  overflow-y: auto;
  scroll-behavior: smooth;

  .filter-panel {
    position: fixed;
    padding: 1rem 0.5rem 1rem 1rem;
    display: flex;
    flex-direction: column;
    height: fit-content;
    min-height: calc(100vh - 3.25rem);
    max-height: calc(100vh - 3.25rem);
    width: 20rem;
    overflow-y: auto;
    -ms-overflow-style: none;
    scrollbar-width: none;

    &::-webkit-scrollbar {
      width: 0;
      background: transparent;
    }

    h2 {
      color: var(--color-contrast);
      margin-top: 1rem;
      margin-bottom: 0.5rem;
      font-size: 1.16rem;
    }
  }

  .search {
    padding: 1rem;

    .offline {
      margin: 1rem;
      text-align: center;
    }

    .loading {
      margin: 2rem;
      text-align: center;
    }
  }
}
</style>
