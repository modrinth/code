import { type Ref, type Component, computed, readonly, ref } from 'vue'
import { type LocationQueryRaw, type LocationQueryValue, useRoute } from 'vue-router'
import { defineMessage, useVIntl } from '@vintl/vintl'
import { formatCategory, formatCategoryHeader, sortByNameOrNumber } from '@modrinth/utils'
import { ClientIcon, ServerIcon } from '@modrinth/assets'

type BaseOption = {
  id: string
  formatted_name?: string
  toggle_group?: string
  icon?: string | Component
  query_value?: string
}

export type FilterOption = BaseOption &
  (
    | { method: 'or' | 'and'; value: string }
    | { method: 'environment'; environment: 'client' | 'server' }
  )

export type FilterType = {
  id: string
  formatted_name: string
  options: FilterOption[]
  supported_project_types: ProjectType[]
  query_param: string
  supports_negative_filter: boolean
  toggle_groups?: {
    id: string
    formatted_name: string
    query_param?: string
  }[]
  searchable: boolean
  allows_custom_options?: 'and' | 'or'
  ordering?: number
} & (
  | {
      display: 'all' | 'scrollable' | 'none'
    }
  | {
      display: 'expandable'
      default_values: string[]
    }
)

export type FilterValue = {
  type: string
  option: string
  negative?: boolean
}

export interface GameVersion {
  version: string
  version_type: 'release' | 'snapshot' | 'alpha' | 'beta'
  date: string
  major: boolean
}

export type ProjectType = 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'datapack' | 'plugin'

const ALL_PROJECT_TYPES: ProjectType[] = [
  'mod',
  'modpack',
  'resourcepack',
  'shader',
  'datapack',
  'plugin',
]

export interface Platform {
  name: string
  icon: string
  supported_project_types: ProjectType[]
  default: boolean
  formatted_name: string
}

export interface Category {
  icon: string
  name: string
  project_type: ProjectType
  header: string
}

export interface Tags {
  gameVersions: GameVersion[]
  loaders: Platform[]
  categories: Category[]
}

export interface SortType {
  display: string
  name: string
}

export function useSearch(
  projectTypes: Ref<ProjectType[]>,
  tags: Ref<Tags>,
  providedFilters: Ref<FilterValue[]>,
) {
  const query = ref('')
  const maxResults = ref(20)

  const sortTypes: readonly SortType[] = readonly([
    { display: 'Relevance', name: 'relevance' },
    { display: 'Downloads', name: 'downloads' },
    { display: 'Followers', name: 'follows' },
    { display: 'Date published', name: 'newest' },
    { display: 'Date updated', name: 'updated' },
  ])

  const currentSortType: Ref<SortType> = ref({ name: 'relevance', display: 'Relevance' })

  const route = useRoute()
  const currentPage = ref(1)

  const currentFilters: Ref<FilterValue[]> = ref<FilterValue[]>([])
  const toggledGroups = ref<string[]>([])
  const overriddenProvidedFilterTypes = ref<string[]>([])

  const { formatMessage } = useVIntl()

  const filters = computed(() => {
    const categoryFilters: Record<string, FilterType> = {}
    for (const category of sortByNameOrNumber(tags.value.categories.slice(), ['header', 'name'])) {
      const filterTypeId = `category_${category.project_type}_${category.header}`
      if (!categoryFilters[filterTypeId]) {
        categoryFilters[filterTypeId] = {
          id: filterTypeId,
          formatted_name: formatCategoryHeader(category.header),
          supported_project_types:
            category.project_type === 'mod'
              ? ['mod', 'plugin', 'datapack']
              : [category.project_type],
          display: 'all',
          query_param: category.header === 'resolutions' ? 'g' : 'f',
          supports_negative_filter: true,
          searchable: false,
          options: [],
        }
      }
      categoryFilters[filterTypeId].options.push({
        id: category.name,
        formatted_name: formatCategory(category.name),
        icon: category.icon,
        value: `categories:${category.name}`,
        method: category.header === 'resolutions' ? 'or' : 'and',
      })
    }

    const filterTypes: FilterType[] = [
      ...Object.values(categoryFilters),
      {
        id: 'environment',
        formatted_name: formatMessage(
          defineMessage({ id: 'search.filter_type.environment', defaultMessage: 'Environment' }),
        ),
        supported_project_types: ['mod', 'modpack'],
        display: 'all',
        query_param: 'e',
        supports_negative_filter: false,
        searchable: false,
        options: [
          {
            id: 'client',
            formatted_name: formatMessage(
              defineMessage({
                id: 'search.filter_type.environment.client',
                defaultMessage: 'Client',
              }),
            ),
            icon: ClientIcon,
            method: 'environment',
            environment: 'client',
          },
          {
            id: 'server',
            formatted_name: formatMessage(
              defineMessage({
                id: 'search.filter_type.environment.server',
                defaultMessage: 'Server',
              }),
            ),
            icon: ServerIcon,
            method: 'environment',
            environment: 'server',
          },
        ],
      },
      {
        id: 'game_version',
        formatted_name: formatMessage(
          defineMessage({ id: 'search.filter_type.game_version', defaultMessage: 'Game version' }),
        ),
        supported_project_types: ALL_PROJECT_TYPES,
        display: 'scrollable',
        query_param: 'v',
        supports_negative_filter: false,
        toggle_groups: [
          {
            id: 'all_versions',
            formatted_name: formatMessage(
              defineMessage({
                id: 'search.filter_type.game_version.all_versions',
                defaultMessage: 'Show all versions',
              }),
            ),
            query_param: 'h',
          },
        ],
        searchable: true,
        options: tags.value.gameVersions.map((gameVersion) => ({
          id: gameVersion.version,
          toggle_group: gameVersion.version_type !== 'release' ? 'all_versions' : undefined,
          value: `versions:${gameVersion.version}`,
          query_value: gameVersion.version,
          method: 'or',
        })),
        ordering: projectTypes.value.includes('mod') ? 2 : undefined,
      },
      {
        id: 'mod_loader',
        formatted_name: formatMessage(
          defineMessage({ id: 'search.filter_type.mod_loader', defaultMessage: 'Loader' }),
        ),
        supported_project_types: ['mod'],
        display: 'expandable',
        query_param: 'g',
        supports_negative_filter: true,
        default_values: ['fabric', 'forge', 'neoforge', 'quilt'],
        searchable: false,
        options: tags.value.loaders
          .filter(
            (loader) =>
              loader.supported_project_types.includes('mod') &&
              !loader.supported_project_types.includes('plugin') &&
              !loader.supported_project_types.includes('datapack'),
          )
          .map((loader) => {
            return {
              id: loader.name,
              formatted_name: formatCategory(loader.name),
              icon: loader.icon,
              method: 'or',
              value: `categories:${loader.name}`,
            }
          }),
        ordering: projectTypes.value.includes('mod') ? 1 : undefined,
      },
      {
        id: 'modpack_loader',
        formatted_name: formatMessage(
          defineMessage({ id: 'search.filter_type.modpack_loader', defaultMessage: 'Loader' }),
        ),
        supported_project_types: ['modpack'],
        display: 'all',
        query_param: 'g',
        supports_negative_filter: true,
        searchable: false,
        options: tags.value.loaders
          .filter((loader) => loader.supported_project_types.includes('modpack'))
          .map((loader) => {
            return {
              id: loader.name,
              formatted_name: formatCategory(loader.name),
              icon: loader.icon,
              method: 'or',
              value: `categories:${loader.name}`,
            }
          }),
      },
      {
        id: 'plugin_loader',
        formatted_name: formatMessage(
          defineMessage({ id: 'search.filter_type.plugin_loader', defaultMessage: 'Loader' }),
        ),
        supported_project_types: ['plugin'],
        display: 'all',
        query_param: 'g',
        supports_negative_filter: true,
        searchable: false,
        options: tags.value.loaders
          .filter(
            (loader) =>
              loader.supported_project_types.includes('plugin') &&
              !['bungeecord', 'waterfall', 'velocity'].includes(loader.name),
          )
          .map((loader) => {
            return {
              id: loader.name,
              formatted_name: formatCategory(loader.name),
              icon: loader.icon,
              method: 'or',
              value: `categories:${loader.name}`,
            }
          }),
      },
      {
        id: 'plugin_platform',
        formatted_name: formatMessage(
          defineMessage({ id: 'search.filter_type.plugin_platform', defaultMessage: 'Platform' }),
        ),
        supported_project_types: ['plugin'],
        display: 'all',
        query_param: 'g',
        supports_negative_filter: true,
        searchable: false,
        options: tags.value.loaders
          .filter((loader) => ['bungeecord', 'waterfall', 'velocity'].includes(loader.name))
          .map((loader) => {
            return {
              id: loader.name,
              formatted_name: formatCategory(loader.name),
              icon: loader.icon,
              method: 'or',
              value: `categories:${loader.name}`,
            }
          }),
      },
      {
        id: 'shader_loader',
        formatted_name: formatMessage(
          defineMessage({ id: 'search.filter_type.shader_loader', defaultMessage: 'Loader' }),
        ),
        supported_project_types: ['shader'],
        display: 'all',
        query_param: 'g',
        supports_negative_filter: true,
        searchable: false,
        options: tags.value.loaders
          .filter((loader) => loader.supported_project_types.includes('shader'))
          .map((loader) => {
            return {
              id: loader.name,
              formatted_name: formatCategory(loader.name),
              icon: loader.icon,
              method: 'or',
              value: `categories:${loader.name}`,
            }
          }),
      },
      {
        id: 'license',
        formatted_name: formatMessage(
          defineMessage({ id: 'search.filter_type.license', defaultMessage: 'License' }),
        ),
        supported_project_types: ['mod', 'modpack', 'resourcepack', 'shader', 'plugin', 'datapack'],
        query_param: 'l',
        supports_negative_filter: true,
        display: 'all',
        searchable: false,
        options: [
          {
            id: 'open_source',
            formatted_name: formatMessage(
              defineMessage({
                id: 'search.filter_type.license.open_source',
                defaultMessage: 'Open source',
              }),
            ),
            method: 'and',
            value: 'open_source:true',
          },
        ],
      },
      {
        id: 'project_id',
        formatted_name: formatMessage(
          defineMessage({ id: 'search.filter_type.project_id', defaultMessage: 'Project ID' }),
        ),
        supported_project_types: ALL_PROJECT_TYPES,
        query_param: 'pid',
        supports_negative_filter: true,
        display: 'none',
        searchable: false,
        options: [],
        allows_custom_options: 'and',
      },
    ]

    return filterTypes
      .filter((filterType) =>
        filterType.supported_project_types.some((projectType) =>
          projectTypes.value.includes(projectType),
        ),
      )
      .sort((a, b) => (b.ordering ?? 0) - (a.ordering ?? 0))
  })

  const facets = computed(() => {
    const validProvidedFilters = providedFilters.value.filter(
      (providedFilter) => !overriddenProvidedFilterTypes.value.includes(providedFilter.type),
    )
    const filteredFilters = currentFilters.value.filter(
      (userFilter) =>
        !validProvidedFilters.some((providedFilter) => providedFilter.type === userFilter.type),
    )
    const filterValues = [...filteredFilters, ...validProvidedFilters]

    const andFacets: string[][] = []
    const orFacets: Record<string, string[]> = {}
    for (const filterValue of filterValues) {
      const type = filters.value.find((type) => type.id === filterValue.type)
      if (!type) {
        console.error(`Filter type ${filterValue.type} not found`)
        continue
      }
      let option = type?.options.find((option) => option.id === filterValue.option)
      if (!option && type.allows_custom_options) {
        option = {
          id: filterValue.option,
          formatted_name: filterValue.option,
          icon: undefined,
          method: type.allows_custom_options,
          value: filterValue.option,
        }
      } else if (!option) {
        console.error(`Filter option ${filterValue.option} not found`)
        continue
      }

      if (option.method === 'or' || option.method === 'and') {
        if (filterValue.negative) {
          andFacets.push([option.value.replace(':', '!=')])
        } else {
          if (option.method === 'or') {
            if (!orFacets[type.id]) {
              orFacets[type.id] = []
            }
            orFacets[type.id].push(option.value)
          } else if (option.method === 'and') {
            andFacets.push([option.value])
          }
        }
      }
    }

    Object.values(orFacets).forEach((facets) => andFacets.push(facets))

    /*
       Add environment facets, separate from the rest because it oddly depends on the combination
       of filters selected to determine which facets to add.
     */
    const client = currentFilters.value.some(
      (filter) => filter.type === 'environment' && filter.option === 'client',
    )
    const server = currentFilters.value.some(
      (filter) => filter.type === 'environment' && filter.option === 'server',
    )
    andFacets.push(...createEnvironmentFacets(client, server))

    const projectType = projectTypes.value.map((projectType) => `project_type:${projectType}`)
    if (andFacets.length > 0) {
      return [projectType, ...andFacets]
    } else {
      return [projectType]
    }
  })

  const requestParams: Ref<string> = computed(() => {
    const params = [`limit=${maxResults.value}`, `index=${currentSortType.value.name}`]

    if (query.value.length > 0) {
      params.push(`query=${encodeURIComponent(query.value)}`)
    }

    params.push(`facets=${encodeURIComponent(JSON.stringify(facets.value))}`)

    const offset = (currentPage.value - 1) * maxResults.value
    if (currentPage.value !== 1) {
      params.push(`offset=${offset}`)
    }

    return `?${params.join('&')}`
  })

  readQueryParams()

  function readQueryParams() {
    const readParams = new Set<string>()

    // Load legacy params
    loadQueryParam(['l'], (openSource) => {
      if (
        openSource === 'true' &&
        !currentFilters.value.some(
          (filter) => filter.type === 'license' && filter.option === 'open_source',
        )
      ) {
        currentFilters.value.push({
          type: 'license',
          option: 'open_source',
          negative: false,
        })
        readParams.add('l')
      }
    })
    loadQueryParam(['nf'], (filter) => {
      const set = typeof filter === 'string' ? new Set([filter]) : new Set(filter)

      typesLoop: for (const type of filters.value) {
        for (const option of type.options) {
          const value = getOptionValue(option, false)
          if (
            set.has(value) &&
            !currentFilters.value.some(
              (filter) => filter.type === type.id && filter.option === option.id,
            )
          ) {
            currentFilters.value.push({
              type: type.id,
              option: option.id,
              negative: true,
            })
            readParams.add(type.query_param)

            set.delete(value)

            if (set.size === 0) {
              break typesLoop
            }
          }
        }
      }
    })
    loadQueryParam(['s'], (sort) => {
      currentSortType.value = sortTypes.find((sortType) => sortType.name === sort) ?? sortTypes[0]
      readParams.add('s')
    })
    loadQueryParam(['m'], (count) => {
      maxResults.value = Number(count)
      readParams.add('m')
    })
    loadQueryParam(['o'], (offset) => {
      currentPage.value = Math.ceil(Number(offset) / maxResults.value) + 1
      readParams.add('o')
    })
    loadQueryParam(['page'], (page) => {
      currentPage.value = Number(page)
      readParams.add('page')
    })
    loadQueryParam(['q'], (queryVal) => {
      query.value = String(queryVal)
      readParams.add('q')
    })

    for (const key of Object.keys(route.query).filter((key) => !readParams.has(key))) {
      const type = filters.value.find((type) => type.query_param === key)
      if (type) {
        const values = getParamValuesAsArray(route.query[key])

        for (const value of values) {
          const negative = !value.includes(':') && value.includes('!=')
          const option = type.options.find((option) => getOptionValue(option, negative) === value)

          if (!option && type.allows_custom_options) {
            currentFilters.value.push({
              type: type.id,
              option: value.replace('!=', ':'),
              negative: negative,
            })
          } else if (option) {
            currentFilters.value.push({
              type: type.id,
              option: option.id,
              negative: negative,
            })
          } else {
            console.error(`Unknown filter option: ${value}`)
          }
        }
      } else {
        console.error(`Unknown filter type: ${key}`)
      }
    }
  }

  function createPageParams(): LocationQueryRaw {
    const items: Record<string, string[]> = {}

    if (query.value) {
      items.q = [query.value]
    }

    currentFilters.value.forEach((filterValue) => {
      const type = filters.value.find((type) => type.id === filterValue.type)
      const option = type?.options.find((option) => option.id === filterValue.option)
      if (type && option) {
        const value = getOptionValue(option, filterValue.negative)
        if (items[type.query_param]) {
          items[type.query_param].push(value)
        } else {
          items[type.query_param] = [value]
        }
      }
    })

    toggledGroups.value.forEach((groupId) => {
      const group = filters.value
        .flatMap((filter) => filter.toggle_groups)
        .find((group) => group && group.id === groupId)

      if (group && 'query_param' in group && group.query_param) {
        items[group.query_param] = [String(true)]
      }
    })

    if (currentSortType.value.name !== 'relevance') {
      items.s = [currentSortType.value.name]
    }
    if (maxResults.value !== 20) {
      items.m = [String(maxResults.value)]
    }
    if (currentPage.value > 1) {
      items.page = [String(currentPage.value)]
    }

    return items
  }

  function createPageParamsString(
    pageParams: Record<string, string | string[] | boolean | number>,
  ) {
    let url = ``

    Object.entries(pageParams).forEach(([key, value]) => {
      if (Array.isArray(value)) {
        value.forEach((value) => {
          url = addQueryParam(url, key, value)
        })
      } else {
        url = addQueryParam(url, key, value)
      }
    })

    return url
  }

  function loadQueryParam(
    params: string[],
    provider: (param: LocationQueryValue | LocationQueryValue[]) => void,
  ) {
    for (const param of params) {
      if (param in route.query) {
        provider(route.query[param])
        return
      }
    }
  }

  return {
    // Selections
    query,
    currentSortType,
    currentFilters,
    toggledGroups,
    currentPage,
    maxResults,
    overriddenProvidedFilterTypes,

    // Lists
    sortTypes,
    filters,

    // Computed
    facets,
    requestParams,

    // Functions
    createPageParams,
    createPageParamsString,
  }
}

export function createEnvironmentFacets(client: boolean, server: boolean): string[][] {
  const facets: string[][] = []
  if (client && server) {
    facets.push(['client_side:required'], ['server_side:required'])
  } else if (client) {
    facets.push(
      ['client_side:optional', 'client_side:required'],
      ['server_side:optional', 'server_side:unsupported'],
    )
  } else if (server) {
    facets.push(
      ['client_side:optional', 'client_side:unsupported'],
      ['server_side:optional', 'server_side:required'],
    )
  }
  return facets
}

function getOptionValue(option: FilterOption, negative?: boolean): string {
  let value = option.method === 'or' || option.method === 'and' ? option.value : option.id
  if (negative === true) {
    value = value.replace(':', '!=')
  }
  if (option.query_value) {
    value = option.query_value
  }
  return value
}

function addQueryParam(existing: string, key: string, value: string | number | boolean) {
  return existing + `${!existing ? '?' : '&'}${key}=${encodeURIComponent(value)}`
}

function getParamValuesAsArray(x: LocationQueryValue | LocationQueryValue[]): string[] {
  if (x === null) {
    return []
  } else if (typeof x === 'string') {
    return [x]
  } else {
    return x.filter((x) => x !== null)
  }
}
