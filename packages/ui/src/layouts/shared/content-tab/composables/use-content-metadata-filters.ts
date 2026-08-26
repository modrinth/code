import { useSessionStorage } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import type {
	DropdownFilterBarCategory,
	DropdownFilterBarOption,
} from '#ui/components/base/DropdownFilterBar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { ContentItem } from '../types'
import { getClientWarningType } from './content-filtering'

export type ContentMetadataFilterValue = Record<string, string[]>

interface MetadataFilterDefinition {
	key: string
	label: string
	searchable?: boolean
	direct?: boolean
	options?: DropdownFilterBarOption[]
	values: (item: ContentItem) => DropdownFilterBarOption[]
}

interface ContentMetadataFilterConfig {
	showSharedContent?: Ref<boolean> | Readonly<Ref<boolean>>
	showEnvironmentWarnings?: boolean
}

const openSourceLicenseIds = new Set([
	'0BSD',
	'AFL-3.0',
	'AGPL-3.0',
	'Apache-2.0',
	'Artistic-2.0',
	'BSD-2-Clause',
	'BSD-3-Clause',
	'BSL-1.0',
	'CDDL-1.0',
	'ECL-2.0',
	'EPL-1.0',
	'EPL-2.0',
	'EUPL-1.1',
	'EUPL-1.2',
	'GPL-2.0',
	'GPL-3.0',
	'ISC',
	'LGPL-2.1',
	'LGPL-3.0',
	'MIT',
	'MPL-2.0',
	'NCSA',
	'OSL-3.0',
	'PostgreSQL',
	'Python-2.0',
	'Unlicense',
	'UPL-1.0',
	'Zlib',
])

type EnvironmentFilterValue = 'client' | 'server' | 'client_and_server' | 'singleplayer'

function getEnvironmentFilterValue(
	environment?: ContentItem['environment'],
): EnvironmentFilterValue | undefined {
	switch (environment) {
		case 'client_only':
			return 'client'
		case 'server_only':
		case 'dedicated_server_only':
			return 'server'
		case 'client_and_server':
		case 'client_only_server_optional':
		case 'server_only_client_optional':
		case 'client_or_server':
		case 'client_or_server_prefers_both':
			return 'client_and_server'
		case 'singleplayer_only':
			return 'singleplayer'
		default:
			return undefined
	}
}

const messages = defineMessages({
	author: {
		id: 'content.metadata-filter.author',
		defaultMessage: 'Author',
	},
	openSource: {
		id: 'content.metadata-filter.open-source',
		defaultMessage: 'Open source',
	},
	environment: {
		id: 'content.metadata-filter.environment',
		defaultMessage: 'Environment',
	},
	clientSideOnly: {
		id: 'project.settings.environment.client_only.title',
		defaultMessage: 'Client-side only',
	},
	serverSideOnly: {
		id: 'project.settings.environment.server_only.title',
		defaultMessage: 'Server-side only',
	},
	clientAndServer: {
		id: 'project.settings.environment.client_and_server.title',
		defaultMessage: 'Client and server',
	},
	singleplayerOnly: {
		id: 'project.settings.environment.singleplayer.title',
		defaultMessage: 'Singleplayer only',
	},
	state: {
		id: 'content.metadata-filter.state',
		defaultMessage: 'State',
	},
	warnings: {
		id: 'content.metadata-filter.warnings',
		defaultMessage: 'Warnings',
	},
	enabled: {
		id: 'content.metadata-filter.state.enabled',
		defaultMessage: 'Enabled',
	},
	disabled: {
		id: 'content.metadata-filter.state.disabled',
		defaultMessage: 'Disabled',
	},
	updateAvailable: {
		id: 'content.metadata-filter.update.available',
		defaultMessage: 'Update available',
	},
	clientRetained: {
		id: 'content.metadata-filter.warning.client-retained',
		defaultMessage: 'Client file retained',
	},
	clientDepends: {
		id: 'content.metadata-filter.warning.client-depends',
		defaultMessage: 'Client depends on file',
	},
	clientOnly: {
		id: 'content.metadata-filter.warning.client-only',
		defaultMessage: 'Client-only content',
	},
	noWarnings: {
		id: 'content.metadata-filter.warning.none',
		defaultMessage: 'No warnings',
	},
	external: {
		id: 'content.metadata-filter.source.external',
		defaultMessage: 'External',
	},
	sharedContent: {
		id: 'content.metadata-filter.shared-content',
		defaultMessage: 'Shared content',
	},
})

export function useContentMetadataFilters(
	items: Ref<ContentItem[]>,
	persistKey?: string,
	config?: ContentMetadataFilterConfig,
) {
	const { formatMessage } = useVIntl()
	const selectedMetadataFilters = persistKey
		? useSessionStorage<ContentMetadataFilterValue>(`content-metadata-filters:${persistKey}`, {})
		: ref<ContentMetadataFilterValue>({})

	function option(value: string, label: string, searchTerms?: string[]): DropdownFilterBarOption {
		return { value, label, searchTerms }
	}

	function isOpenSource(item: ContentItem) {
		const licenseId = item.project?.license?.id.replace(/-(?:only|or-later)$/, '')
		return !!licenseId && openSourceLicenseIds.has(licenseId)
	}

	function isExternal(item: ContentItem) {
		return item.external || !item.project?.license
	}

	function getEnvironmentFilterLabel(value: EnvironmentFilterValue) {
		switch (value) {
			case 'client':
				return formatMessage(messages.clientSideOnly)
			case 'server':
				return formatMessage(messages.serverSideOnly)
			case 'client_and_server':
				return formatMessage(messages.clientAndServer)
			case 'singleplayer':
				return formatMessage(messages.singleplayerOnly)
		}
	}

	const definitions = computed<MetadataFilterDefinition[]>(() => [
		{
			key: 'author',
			label: formatMessage(messages.author),
			searchable: true,
			values: (item) =>
				item.owner
					? [option(`${item.owner.type}:${item.owner.id}`, item.owner.name, [item.owner.id])]
					: [],
		},
		{
			key: 'environment',
			label: formatMessage(messages.environment),
			options: [
				option('client', getEnvironmentFilterLabel('client')),
				option('server', getEnvironmentFilterLabel('server')),
				option('client_and_server', getEnvironmentFilterLabel('client_and_server')),
				option('singleplayer', getEnvironmentFilterLabel('singleplayer')),
			],
			values: (item) => {
				const value = getEnvironmentFilterValue(item.environment)
				return value ? [option(value, getEnvironmentFilterLabel(value))] : []
			},
		},
		{
			key: 'state',
			label: formatMessage(messages.state),
			values: (item) =>
				item.enabled === undefined
					? []
					: [
							item.enabled
								? option('enabled', formatMessage(messages.enabled))
								: option('disabled', formatMessage(messages.disabled)),
						],
		},
		{
			key: 'warnings',
			label: formatMessage(messages.warnings),
			values: (item) => {
				const warning = getClientWarningType(item, config?.showEnvironmentWarnings)
				switch (warning) {
					case 'retained':
						return [option(warning, formatMessage(messages.clientRetained))]
					case 'depends':
						return [option(warning, formatMessage(messages.clientDepends))]
					case 'environment':
						return [option(warning, formatMessage(messages.clientOnly))]
					default:
						return [option('none', formatMessage(messages.noWarnings))]
				}
			},
		},
		{
			key: 'updates',
			label: formatMessage(messages.updateAvailable),
			direct: true,
			values: (item) =>
				item.has_update ? [option('available', formatMessage(messages.updateAvailable))] : [],
		},
		{
			key: 'open_source',
			label: formatMessage(messages.openSource),
			direct: true,
			values: (item) =>
				isOpenSource(item) ? [option('open_source', formatMessage(messages.openSource))] : [],
		},
		{
			key: 'external',
			label: formatMessage(messages.external),
			direct: true,
			values: (item) =>
				isExternal(item) ? [option('external', formatMessage(messages.external))] : [],
		},
		...(config?.showSharedContent?.value
			? [
					{
						key: 'shared_content',
						label: formatMessage(messages.sharedContent),
						direct: true,
						values: (item: ContentItem) =>
							['server_project', 'shared_instance'].includes(item.source_kind ?? '')
								? [option('shared_content', formatMessage(messages.sharedContent))]
								: [],
					},
				]
			: []),
	])

	const metadataFilterCategories = computed<DropdownFilterBarCategory[]>(() =>
		definitions.value
			.map((definition) => {
				let visibleOptions = definition.options
				if (!visibleOptions) {
					const options = new Map<string, DropdownFilterBarOption>()
					const optionMatchCounts = new Map<string, number>()
					for (const item of items.value) {
						const itemValues = new Map(
							definition.values(item).map((value) => [value.value, value] as const),
						)
						for (const value of itemValues.values()) {
							if (!options.has(value.value)) options.set(value.value, value)
							optionMatchCounts.set(value.value, (optionMatchCounts.get(value.value) ?? 0) + 1)
						}
					}

					visibleOptions = [...options.values()]
						.filter((option) => optionMatchCounts.get(option.value) !== items.value.length)
						.sort((a, b) => a.label.localeCompare(b.label, undefined, { numeric: true }))
				}

				return {
					key: definition.key,
					label: definition.label,
					direct: definition.direct,
					searchable: definition.searchable,
					options: visibleOptions,
				}
			})
			.filter((category) => category.options.some((option) => !('type' in option))),
	)

	watch(
		metadataFilterCategories,
		(categories) => {
			if (items.value.length === 0) return
			const availableValues = new Map(
				categories.map((category) => [
					category.key,
					new Set(
						category.options
							.filter((item): item is DropdownFilterBarOption => !('type' in item))
							.map((item) => item.value),
					),
				]),
			)
			const nextFilters: ContentMetadataFilterValue = {}
			for (const [key, values] of Object.entries(selectedMetadataFilters.value)) {
				const validValues = values.filter((value) => availableValues.get(key)?.has(value))
				if (validValues.length > 0) nextFilters[key] = validValues
			}
			if (JSON.stringify(nextFilters) !== JSON.stringify(selectedMetadataFilters.value)) {
				selectedMetadataFilters.value = nextFilters
			}
		},
		{ immediate: true },
	)

	function applyMetadataFilters(source: ContentItem[]) {
		const activeFilters = Object.entries(selectedMetadataFilters.value).filter(
			([, values]) => values.length > 0,
		)
		if (activeFilters.length === 0) return source

		const definitionsByKey = new Map(
			definitions.value.map((definition) => [definition.key, definition]),
		)
		return source.filter((item) =>
			activeFilters.every(([key, selectedValues]) => {
				const definition = definitionsByKey.get(key)
				if (!definition) return true
				const itemValues = definition.values(item).map((value) => value.value)
				return itemValues.some((value) => selectedValues.includes(value))
			}),
		)
	}

	return {
		selectedMetadataFilters,
		metadataFilterCategories,
		applyMetadataFilters,
	}
}
