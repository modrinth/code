import { getCategoryIcon } from '@modrinth/assets'
import { computed, type Ref,ref } from 'vue'

import { useVIntl } from '../composables/i18n'
import type { FilterType, FilterValue, SortType, Tags } from './search'
import { formatCategory } from './tag-messages'

export const SERVER_COUNTRIES = [
	{ code: 'US', name: 'United States' },
	{ code: 'GB', name: 'United Kingdom' },
	{ code: 'DE', name: 'Germany' },
	{ code: 'FR', name: 'France' },
	{ code: 'NL', name: 'Netherlands' },
	{ code: 'PL', name: 'Poland' },
	{ code: 'RU', name: 'Russia' },
	{ code: 'BR', name: 'Brazil' },
	{ code: 'CA', name: 'Canada' },
	{ code: 'AU', name: 'Australia' },
	{ code: 'SE', name: 'Sweden' },
	{ code: 'FI', name: 'Finland' },
	{ code: 'SG', name: 'Singapore' },
	{ code: 'JP', name: 'Japan' },
	{ code: 'KR', name: 'South Korea' },
	{ code: 'TR', name: 'Turkey' },
	{ code: 'IN', name: 'India' },
	{ code: 'ZA', name: 'South Africa' },
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
	server_category: 'categories',
	server_game_version: 'versions',
	server_country: 'minecraft_server.country',
	server_language: 'minecraft_server.languages',
}

export function useServerSearch(opts: {
	tags: Ref<Tags>
	query: Ref<string>
	maxResults: Ref<number>
	currentPage: Ref<number>
}) {
	const { tags, query, maxResults, currentPage } = opts

	const { formatMessage } = useVIntl()

	const serverCurrentSortType = ref<SortType>(SERVER_SORT_TYPES[0])
	const serverCurrentFilters = ref<FilterValue[]>([])
	const serverToggledGroups = ref<string[]>([])

	const serverFilterTypes = computed<FilterType[]>(() => [
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
				{ id: 'modpack', formatted_name: 'Modpack', method: 'or', value: 'modpack' },
			],
		},
		{
			id: 'server_category',
			formatted_name: 'Category',
			supported_project_types: ['server'],
			display: 'all',
			query_param: 'sc',
			supports_negative_filter: false,
			searchable: false,
			options: (tags.value?.categories ?? [])
				.filter((c) => c.project_type === 'server')
				.map((c) => ({
					id: c.name,
					formatted_name: formatCategory(formatMessage, c.name),
					icon: getCategoryIcon(c.name),
					method: 'or' as const,
					value: c.name,
				})),
		},
		{
			id: 'server_game_version',
			formatted_name: 'Game Version',
			supported_project_types: ['server'],
			display: 'scrollable',
			query_param: 'sgv',
			supports_negative_filter: false,
			searchable: true,
			toggle_groups: [{ id: 'all_versions', formatted_name: 'Show all versions', query_param: 'sh' }],
			options: (tags.value?.gameVersions ?? []).map((gv) => ({
				id: gv.version,
				toggle_group: gv.version_type !== 'release' ? 'all_versions' : undefined,
				method: 'or' as const,
				value: gv.version,
				query_value: gv.version,
			})),
		},
		{
			id: 'server_country',
			formatted_name: 'Country',
			supported_project_types: ['server'],
			display: 'scrollable',
			query_param: 'sco',
			supports_negative_filter: false,
			searchable: true,
			options: SERVER_COUNTRIES.map((c) => ({
				id: c.code,
				formatted_name: c.name,
				method: 'or' as const,
				value: c.code,
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
	])

	const newFilters = computed(() => {
		const parts = ['project_types = minecraft_java_server']

		for (const [filterId, field] of Object.entries(FILTER_FIELD_MAP)) {
			const matched = serverCurrentFilters.value.filter((f) => f.type === filterId)
			if (matched.length > 0) {
				const values = matched.map((f) => `"${f.option}"`).join(', ')
				parts.push(`${field} IN [${values}]`)
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

	return {
		serverCurrentSortType,
		serverCurrentFilters,
		serverToggledGroups,
		serverSortTypes: SERVER_SORT_TYPES,
		serverFilterTypes,
		newFilters,
		serverRequestParams,
	}
}
