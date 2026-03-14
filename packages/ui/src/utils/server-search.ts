import type { Labrinth } from '@modrinth/api-client'
import { getCategoryIcon, SERVER_CATEGORY_ICON_MAP } from '@modrinth/assets'
import { sortedCategories } from '@modrinth/utils'
import { computed, type Ref, ref } from 'vue'
import { useRoute } from 'vue-router'

import { defineMessage, useVIntl } from '../composables/i18n'
import type { FilterType, FilterValue, SortType, Tags } from './search'
import { formatCategory, formatCategoryHeader } from './tag-messages'

export const SERVER_REGIONS = {
	us_east: defineMessage({ id: 'project.server.region.us_east', defaultMessage: 'US East' }),
	us_west: defineMessage({ id: 'project.server.region.us_west', defaultMessage: 'US West' }),
	europe: defineMessage({ id: 'project.server.region.europe', defaultMessage: 'Europe' }),
	asia: defineMessage({ id: 'project.server.region.asia', defaultMessage: 'Asia' }),
	australia: defineMessage({ id: 'project.server.region.australia', defaultMessage: 'Australia' }),
	south_america: defineMessage({
		id: 'project.server.region.south_america',
		defaultMessage: 'South America',
	}),
	middle_east: defineMessage({
		id: 'project.server.region.middle_east',
		defaultMessage: 'Middle East',
	}),
	russia: defineMessage({ id: 'project.server.region.russia', defaultMessage: 'Russia' }),
}

export const SERVER_LANGUAGES = {
	en: defineMessage({ id: 'project.server.language.en', defaultMessage: 'English' }),
	es: defineMessage({ id: 'project.server.language.es', defaultMessage: 'Spanish' }),
	pt: defineMessage({ id: 'project.server.language.pt', defaultMessage: 'Portuguese' }),
	fr: defineMessage({ id: 'project.server.language.fr', defaultMessage: 'French' }),
	de: defineMessage({ id: 'project.server.language.de', defaultMessage: 'German' }),
	it: defineMessage({ id: 'project.server.language.it', defaultMessage: 'Italian' }),
	nl: defineMessage({ id: 'project.server.language.nl', defaultMessage: 'Dutch' }),
	ru: defineMessage({ id: 'project.server.language.ru', defaultMessage: 'Russian' }),
	uk: defineMessage({ id: 'project.server.language.uk', defaultMessage: 'Ukrainian' }),
	pl: defineMessage({ id: 'project.server.language.pl', defaultMessage: 'Polish' }),
	cs: defineMessage({ id: 'project.server.language.cs', defaultMessage: 'Czech' }),
	sk: defineMessage({ id: 'project.server.language.sk', defaultMessage: 'Slovak' }),
	hu: defineMessage({ id: 'project.server.language.hu', defaultMessage: 'Hungarian' }),
	ro: defineMessage({ id: 'project.server.language.ro', defaultMessage: 'Romanian' }),
	bg: defineMessage({ id: 'project.server.language.bg', defaultMessage: 'Bulgarian' }),
	hr: defineMessage({ id: 'project.server.language.hr', defaultMessage: 'Croatian' }),
	sr: defineMessage({ id: 'project.server.language.sr', defaultMessage: 'Serbian' }),
	el: defineMessage({ id: 'project.server.language.el', defaultMessage: 'Greek' }),
	tr: defineMessage({ id: 'project.server.language.tr', defaultMessage: 'Turkish' }),
	ar: defineMessage({ id: 'project.server.language.ar', defaultMessage: 'Arabic' }),
	he: defineMessage({ id: 'project.server.language.he', defaultMessage: 'Hebrew' }),
	hi: defineMessage({ id: 'project.server.language.hi', defaultMessage: 'Hindi' }),
	bn: defineMessage({ id: 'project.server.language.bn', defaultMessage: 'Bengali' }),
	ur: defineMessage({ id: 'project.server.language.ur', defaultMessage: 'Urdu' }),
	zh: defineMessage({ id: 'project.server.language.zh', defaultMessage: 'Chinese' }),
	ja: defineMessage({ id: 'project.server.language.ja', defaultMessage: 'Japanese' }),
	ko: defineMessage({ id: 'project.server.language.ko', defaultMessage: 'Korean' }),
	th: defineMessage({ id: 'project.server.language.th', defaultMessage: 'Thai' }),
	vi: defineMessage({ id: 'project.server.language.vi', defaultMessage: 'Vietnamese' }),
	id: defineMessage({ id: 'project.server.language.id', defaultMessage: 'Indonesian' }),
	ms: defineMessage({ id: 'project.server.language.ms', defaultMessage: 'Malay' }),
	tl: defineMessage({ id: 'project.server.language.tl', defaultMessage: 'Filipino' }),
	sv: defineMessage({ id: 'project.server.language.sv', defaultMessage: 'Swedish' }),
	no: defineMessage({ id: 'project.server.language.no', defaultMessage: 'Norwegian' }),
	da: defineMessage({ id: 'project.server.language.da', defaultMessage: 'Danish' }),
	fi: defineMessage({ id: 'project.server.language.fi', defaultMessage: 'Finnish' }),
	lt: defineMessage({ id: 'project.server.language.lt', defaultMessage: 'Lithuanian' }),
	lv: defineMessage({ id: 'project.server.language.lv', defaultMessage: 'Latvian' }),
	et: defineMessage({ id: 'project.server.language.et', defaultMessage: 'Estonian' }),
	af: defineMessage({ id: 'project.server.language.af', defaultMessage: 'Afrikaans' }),
	am: defineMessage({ id: 'project.server.language.am', defaultMessage: 'Amharic' }),
	az: defineMessage({ id: 'project.server.language.az', defaultMessage: 'Azerbaijani' }),
	be: defineMessage({ id: 'project.server.language.be', defaultMessage: 'Belarusian' }),
	bs: defineMessage({ id: 'project.server.language.bs', defaultMessage: 'Bosnian' }),
	ca: defineMessage({ id: 'project.server.language.ca', defaultMessage: 'Catalan' }),
	eo: defineMessage({ id: 'project.server.language.eo', defaultMessage: 'Esperanto' }),
	eu: defineMessage({ id: 'project.server.language.eu', defaultMessage: 'Basque' }),
	fa: defineMessage({ id: 'project.server.language.fa', defaultMessage: 'Persian' }),
	ga: defineMessage({ id: 'project.server.language.ga', defaultMessage: 'Irish' }),
	gl: defineMessage({ id: 'project.server.language.gl', defaultMessage: 'Galician' }),
	hy: defineMessage({ id: 'project.server.language.hy', defaultMessage: 'Armenian' }),
	is: defineMessage({ id: 'project.server.language.is', defaultMessage: 'Icelandic' }),
	ka: defineMessage({ id: 'project.server.language.ka', defaultMessage: 'Georgian' }),
	kk: defineMessage({ id: 'project.server.language.kk', defaultMessage: 'Kazakh' }),
	km: defineMessage({ id: 'project.server.language.km', defaultMessage: 'Khmer' }),
	kn: defineMessage({ id: 'project.server.language.kn', defaultMessage: 'Kannada' }),
	lo: defineMessage({ id: 'project.server.language.lo', defaultMessage: 'Lao' }),
	mk: defineMessage({ id: 'project.server.language.mk', defaultMessage: 'Macedonian' }),
	ml: defineMessage({ id: 'project.server.language.ml', defaultMessage: 'Malayalam' }),
	mn: defineMessage({ id: 'project.server.language.mn', defaultMessage: 'Mongolian' }),
	mr: defineMessage({ id: 'project.server.language.mr', defaultMessage: 'Marathi' }),
	my: defineMessage({ id: 'project.server.language.my', defaultMessage: 'Burmese' }),
	ne: defineMessage({ id: 'project.server.language.ne', defaultMessage: 'Nepali' }),
	pa: defineMessage({ id: 'project.server.language.pa', defaultMessage: 'Punjabi' }),
	si: defineMessage({ id: 'project.server.language.si', defaultMessage: 'Sinhala' }),
	sl: defineMessage({ id: 'project.server.language.sl', defaultMessage: 'Slovenian' }),
	sq: defineMessage({ id: 'project.server.language.sq', defaultMessage: 'Albanian' }),
	sw: defineMessage({ id: 'project.server.language.sw', defaultMessage: 'Swahili' }),
	ta: defineMessage({ id: 'project.server.language.ta', defaultMessage: 'Tamil' }),
	te: defineMessage({ id: 'project.server.language.te', defaultMessage: 'Telugu' }),
	uz: defineMessage({ id: 'project.server.language.uz', defaultMessage: 'Uzbek' }),
	yo: defineMessage({ id: 'project.server.language.yo', defaultMessage: 'Yoruba' }),
	zu: defineMessage({ id: 'project.server.language.zu', defaultMessage: 'Zulu' }),
}

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

	const { formatMessage, locale } = useVIntl()
	const formatCategoryName = (categoryName: string) => {
		return formatCategory(formatMessage, categoryName)
	}

	const route = useRoute()

	const serverCurrentSortType = ref<SortType>(SERVER_SORT_TYPES[0])
	const serverCurrentFilters = ref<FilterValue[]>([{ type: 'server_status', option: 'online' }])
	const serverToggledGroups = ref<string[]>([])

	const serverFilterTypes = computed<FilterType[]>(() => {
		const categoryFilters: Record<string, FilterType> = {}
		for (const c of sortedCategories(tags.value, formatCategoryName, locale.value).filter(
			(c: Labrinth.Tags.v2.Category) => c.project_type === 'minecraft_java_server',
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

		const sortedRegions = Object.entries(SERVER_REGIONS).sort(([_, a], [__, b]) => {
			const aFormatted = formatMessage(a)
			const bFormatted = formatMessage(b)
			return aFormatted.localeCompare(bFormatted, locale.value)
		})

		const sortedLanguages = Object.entries(SERVER_LANGUAGES).sort(([_, a], [__, b]) => {
			const aFormatted = formatMessage(a)
			const bFormatted = formatMessage(b)
			return aFormatted.localeCompare(bFormatted, locale.value)
		})

		return [
			{
				id: 'server_content_type',
				formatted_name: formatMessage(
					defineMessage({
						id: 'search.filter_type.server_content_type',
						defaultMessage: 'Type',
					}),
				),
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sct',
				supports_negative_filter: false,
				searchable: false,
				options: [
					{
						id: 'vanilla',
						formatted_name: formatMessage(
							defineMessage({
								id: 'search.server_content_type.vanilla',
								defaultMessage: 'Vanilla',
							}),
						),
						method: 'or',
						value: 'vanilla',
					},
					{
						id: 'modpack',
						formatted_name: formatMessage(
							defineMessage({
								id: 'search.server_content_type.modpack',
								defaultMessage: 'Modded',
							}),
						),
						method: 'or',
						value: 'modpack',
					},
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
				formatted_name: formatMessage(
					defineMessage({
						id: 'search.filter_type.game_version',
						defaultMessage: 'Game version',
					}),
				),
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
				formatted_name: formatMessage(
					defineMessage({
						id: 'search.filter_type.server_region',
						defaultMessage: 'Region',
					}),
				),
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sr',
				supports_negative_filter: true,
				searchable: false,
				options: sortedRegions.map(([code, name]) => ({
					id: code,
					formatted_name: formatMessage(name),
					method: 'or' as const,
					value: code,
				})),
			},
			{
				id: 'server_language',
				formatted_name: formatMessage(
					defineMessage({
						id: 'search.filter_type.server_language',
						defaultMessage: 'Language',
					}),
				),
				supported_project_types: ['server'],
				display: 'scrollable',
				query_param: 'sl',
				supports_negative_filter: false,
				searchable: true,
				options: sortedLanguages.map(([code, name]) => ({
					id: code,
					formatted_name: formatMessage(name),
					method: 'or' as const,
					value: code,
				})),
			},
			{
				id: 'server_status',
				formatted_name: formatMessage(
					defineMessage({
						id: 'search.filter_type.server_status',
						defaultMessage: 'Status',
					}),
				),
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sst',
				supports_negative_filter: false,
				searchable: false,
				options: [
					{
						id: 'online',
						formatted_name: formatMessage(
							defineMessage({
								id: 'project.server.status.online',
								defaultMessage: 'Online',
							}),
						),
						method: 'or',
						value: 'online',
					},
					{
						id: 'offline',
						formatted_name: formatMessage(
							defineMessage({
								id: 'project.server.status.offline',
								defaultMessage: 'Offline',
							}),
						),
						method: 'or',
						value: 'offline',
					},
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
