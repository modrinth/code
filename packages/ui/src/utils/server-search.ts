import { getCategoryIcon, SERVER_CATEGORY_ICON_MAP } from '@modrinth/assets'
import { computed, type Ref, ref } from 'vue'
import { useRoute } from 'vue-router'

import { useVIntl } from '../composables/i18n'
import type { FilterType, FilterValue, SortType, Tags } from './search'
import { formatCategory, formatCategoryHeader } from './tag-messages'

export const SERVER_REGIONS = [
	{ code: 'us_east', name: 'US East' },
	{ code: 'us_west', name: 'US West' },
	{ code: 'europe', name: 'Europe' },
	{ code: 'asia', name: 'Asia' },
	{ code: 'australia', name: 'Australia' },
	{ code: 'south_america', name: 'South America' },
	{ code: 'middle_east', name: 'Middle East' },
	{ code: 'russia', name: 'Russia' },
]

export const SERVER_LANGUAGES = [
	{ code: 'en', name: 'English' },
	{ code: 'de', name: 'German' },
	{ code: 'fr', name: 'French' },
	{ code: 'es', name: 'Spanish' },
	{ code: 'pt', name: 'Portuguese' },
	{ code: 'ru', name: 'Russian' },
	{ code: 'zh', name: 'Chinese' },
	{ code: 'ja', name: 'Japanese' },
	{ code: 'ko', name: 'Korean' },
	{ code: 'nl', name: 'Dutch' },
	{ code: 'pl', name: 'Polish' },
	{ code: 'it', name: 'Italian' },
	{ code: 'tr', name: 'Turkish' },
	{ code: 'sv', name: 'Swedish' },
	{ code: 'fi', name: 'Finnish' },
]

export const SERVER_SORT_TYPES: SortType[] = [
	{ display: 'Relevance', name: 'relevance' },
	{ display: 'Verified Plays', name: 'minecraft_java_server.verified_plays_2w' },
	{ display: 'Players', name: 'minecraft_java_server.ping.data.players_online' },
	{ display: 'Followers', name: 'follows' },
	{ display: 'Date Published', name: 'date_created' },
	{ display: 'Date Updated', name: 'date_modified' },
]

const FILTER_FIELD_MAP: Record<string, string> = {
	server_content_type: 'minecraft_java_server.content.kind',
	server_game_version: 'minecraft_java_server.content.supported_game_versions',
	server_status: 'minecraft_java_server.ping.data',
	server_region: 'minecraft_server.region',
	server_language: 'minecraft_server.languages',
}

function getFilterField(filterId: string): string | undefined {
	if (filterId.startsWith('server_category_')) return 'categories'
	return FILTER_FIELD_MAP[filterId]
}

export function useServerSearch(opts: {
	tags: Ref<Tags>
	query: Ref<string>
	maxResults: Ref<number>
	currentPage: Ref<number>
}) {
	const { tags, query, maxResults, currentPage } = opts

	const { formatMessage } = useVIntl()

	const route = useRoute()

	const serverCurrentSortType = ref<SortType>(SERVER_SORT_TYPES[0])
	const serverCurrentFilters = ref<FilterValue[]>([{ type: 'server_status', option: 'online' }])
	const serverToggledGroups = ref<string[]>([])

	const serverFilterTypes = computed<FilterType[]>(() => {
		const categoryFilters: Record<string, FilterType> = {}
		for (const c of (tags.value?.categories ?? []).filter(
			(c) => c.project_type === 'minecraft_java_server',
		)) {
			const filterTypeId = `server_category_${c.header}`
			if (!categoryFilters[filterTypeId]) {
				categoryFilters[filterTypeId] = {
					id: filterTypeId,
					formatted_name: formatCategoryHeader(formatMessage, c.header),
					supported_project_types: ['server'],
					display: 'all',
					query_param: 'sc',
					supports_negative_filter: true,
					searchable: false,
					options: [],
				}
			}
			categoryFilters[filterTypeId].options.push({
				id: c.name,
				formatted_name: formatCategory(formatMessage, c.name),
				icon: getCategoryIcon(SERVER_CATEGORY_ICON_MAP[c.name] ?? c.name),
				method: 'or' as const,
				value: c.name,
			})
		}

		const featuresFilter = categoryFilters['server_category_minecraft_server_features']
		if (featuresFilter) {
			featuresFilter.options.sort((a, b) => {
				if (a.id === 'pokemon') return -1
				if (b.id === 'pokemon') return 1
				return 0
			})
		}

		return [
			{
				id: 'server_content_type',
				formatted_name: 'Type',
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sct',
				supports_negative_filter: false,
				searchable: false,
				options: [
					{ id: 'vanilla', formatted_name: 'Vanilla', method: 'or', value: 'vanilla' },
					{ id: 'modpack', formatted_name: 'Modded', method: 'or', value: 'modpack' },
				],
			},
			...[
				'minecraft_server_features',
				'minecraft_server_gameplay',
				'minecraft_server_meta',
				'minecraft_server_community',
			]
				.map((h) => categoryFilters[`server_category_${h}`])
				.filter(Boolean),
			{
				id: 'server_game_version',
				formatted_name: 'Game Version',
				supported_project_types: ['server'],
				display: 'scrollable',
				query_param: 'sgv',
				supports_negative_filter: false,
				searchable: true,
				options: (tags.value?.gameVersions ?? []).map((gv) => ({
					id: gv.version,
					toggle_group: gv.version_type !== 'release' ? 'all_versions' : undefined,
					method: 'or' as const,
					value: gv.version,
					query_value: gv.version,
				})),
			},
			{
				id: 'server_region',
				formatted_name: 'Region',
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sr',
				supports_negative_filter: true,
				searchable: false,
				options: SERVER_REGIONS.map((r) => ({
					id: r.code,
					formatted_name: r.name,
					method: 'or' as const,
					value: r.code,
				})),
			},
			{
				id: 'server_language',
				formatted_name: 'Language',
				supported_project_types: ['server'],
				display: 'scrollable',
				query_param: 'sl',
				supports_negative_filter: false,
				searchable: true,
				options: SERVER_LANGUAGES.map((l) => ({
					id: l.code,
					formatted_name: l.name,
					method: 'or' as const,
					value: l.code,
				})),
			},
			{
				id: 'server_status',
				formatted_name: 'Status',
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sst',
				supports_negative_filter: false,
				searchable: false,
				options: [
					{ id: 'online', formatted_name: 'Online', method: 'or', value: 'online' },
					{ id: 'offline', formatted_name: 'Offline', method: 'or', value: 'offline' },
				],
			},
		]
	})

	const newFilters = computed(() => {
		const parts = ['project_types = minecraft_java_server']

		for (const filterType of serverFilterTypes.value) {
			const field = getFilterField(filterType.id)
			if (!field) continue
			const matched = serverCurrentFilters.value.filter((f) => f.type === filterType.id)
			if (matched.length === 0) continue

			if (filterType.id === 'server_status') {
				const selected = matched[0]?.option
				if (selected === 'online') {
					parts.push(`${field} EXISTS`)
				} else if (selected === 'offline') {
					parts.push(`${field} NOT EXISTS`)
				}
				continue
			}

			const included = matched.filter((f) => !f.negative)
			const excluded = matched.filter((f) => f.negative)
			if (included.length > 0) {
				const values = included.map((f) => `"${f.option}"`).join(', ')
				parts.push(`${field} IN [${values}]`)
			}
			if (excluded.length > 0) {
				const values = excluded.map((f) => `"${f.option}"`).join(', ')
				parts.push(`${field} NOT IN [${values}]`)
			}
		}

		return parts.join(' AND ')
	})

	const serverRequestParams = computed(() => {
		const params = [`limit=${maxResults.value}`, `index=${serverCurrentSortType.value.name}`]
		if (query.value) params.push(`query=${encodeURIComponent(query.value)}`)
		const offset = (currentPage.value - 1) * maxResults.value
		if (offset > 0) params.push(`offset=${offset}`)
		params.push(`new_filters=${encodeURIComponent(newFilters.value)}`)
		return `?${params.join('&')}`
	})

	function readServerQueryParams() {
		const q = route.query

		if (q.q) {
			query.value = String(q.q)
		}

		if (q.ss) {
			serverCurrentSortType.value =
				SERVER_SORT_TYPES.find((s) => s.name === String(q.ss)) ?? SERVER_SORT_TYPES[0]
		}

		if (q.m) {
			maxResults.value = Number(q.m)
		}

		if (q.page) {
			currentPage.value = Number(q.page)
		}

		for (const filterType of serverFilterTypes.value) {
			const paramValue = q[filterType.query_param]
			if (!paramValue) continue

			const values =
				typeof paramValue === 'string'
					? [paramValue]
					: paramValue.filter((v): v is string => v !== null)

			for (const value of values) {
				const isNegative = value.startsWith('!')
				const cleanValue = isNegative ? value.slice(1) : value
				const option = filterType.options.find((o) => o.id === cleanValue)
				if (option) {
					serverCurrentFilters.value.push({
						type: filterType.id,
						option: option.id,
						negative: isNegative,
					})
				}
			}
		}
	}

	function createServerPageParams(): Record<string, string | string[]> {
		const items: Record<string, string[]> = {}

		if (query.value) {
			items.q = [query.value]
		}

		for (const filterValue of serverCurrentFilters.value) {
			const type = serverFilterTypes.value.find((t) => t.id === filterValue.type)
			if (type) {
				const value = filterValue.negative ? `!${filterValue.option}` : filterValue.option
				if (items[type.query_param]) {
					items[type.query_param].push(value)
				} else {
					items[type.query_param] = [value]
				}
			}
		}

		if (serverCurrentSortType.value.name !== 'relevance') {
			items.ss = [serverCurrentSortType.value.name]
		}

		if (maxResults.value !== 20) {
			items.m = [String(maxResults.value)]
		}

		if (currentPage.value > 1) {
			items.page = [String(currentPage.value)]
		}

		return items
	}

	readServerQueryParams()

	return {
		serverCurrentSortType,
		serverCurrentFilters,
		serverToggledGroups,
		serverSortTypes: SERVER_SORT_TYPES,
		serverFilterTypes,
		newFilters,
		serverRequestParams,
		createServerPageParams,
	}
}
