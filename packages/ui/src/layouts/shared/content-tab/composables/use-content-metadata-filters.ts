import {
	BoxIcon,
	BracesIcon,
	GlassesIcon,
	type IconComponent,
	PaintbrushIcon,
	PlugIcon,
} from '@modrinth/assets'
import { useSessionStorage } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import type {
	DropdownFilterBarCategory,
	DropdownFilterBarItem,
	DropdownFilterBarOption,
} from '#ui/components/base/DropdownFilterBar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { getProjectTypeCategoryMessage, normalizeProjectType } from '#ui/utils/common-messages'
import { formatCategory } from '#ui/utils/tag-messages'

import type { ContentItem } from '../types'
import { getClientWarningType } from './content-filtering'

export type ContentMetadataFilterValue = Record<string, string[]>

interface MetadataFilterSection {
	key: string
	label: string
	icon: IconComponent
	order: number
}

interface ResolvedFilterValue extends DropdownFilterBarOption {
	section?: MetadataFilterSection
}

interface MetadataFilterDefinition {
	key: string
	label: string
	searchable?: boolean
	direct?: boolean
	values: (item: ContentItem) => ResolvedFilterValue[]
}

interface ContentMetadataFilterConfig {
	showSharedContent?: Ref<boolean> | Readonly<Ref<boolean>>
}

const projectTypeOrder = ['mod', 'plugin', 'datapack', 'resourcepack', 'shader', 'project']
const projectTypeIcons: Record<string, IconComponent> = {
	mod: BoxIcon,
	plugin: PlugIcon,
	datapack: BracesIcon,
	resourcepack: PaintbrushIcon,
	shader: GlassesIcon,
	project: BoxIcon,
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

const messages = defineMessages({
	author: {
		id: 'content.metadata-filter.author',
		defaultMessage: 'Author',
	},
	openSource: {
		id: 'content.metadata-filter.open-source',
		defaultMessage: 'Open source',
	},
	category: {
		id: 'content.metadata-filter.category',
		defaultMessage: 'Category',
	},
	state: {
		id: 'content.metadata-filter.state',
		defaultMessage: 'State',
	},
	updates: {
		id: 'content.metadata-filter.updates',
		defaultMessage: 'Updates',
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
	upToDate: {
		id: 'content.metadata-filter.update.up-to-date',
		defaultMessage: 'Up to date',
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

	function option(
		value: string,
		label: string,
		searchTerms?: string[],
		section?: MetadataFilterSection,
	): ResolvedFilterValue {
		return { value, label, searchTerms, section }
	}

	function isOpenSource(item: ContentItem) {
		const licenseId = item.project?.license?.id.replace(/-(?:only|or-later)$/, '')
		return !!licenseId && openSourceLicenseIds.has(licenseId)
	}

	function isExternal(item: ContentItem) {
		return item.external || !item.project?.license
	}

	function getCategorySection(item: ContentItem): MetadataFilterSection {
		const normalizedType = normalizeProjectType(item.project_type)
		const key = projectTypeIcons[normalizedType] ? normalizedType : 'project'
		return {
			key,
			label: formatMessage(getProjectTypeCategoryMessage(key)),
			icon: projectTypeIcons[key],
			order: projectTypeOrder.indexOf(key),
		}
	}

	function buildCategoryOptions(options: ResolvedFilterValue[]): DropdownFilterBarItem[] {
		if (!options.some((option) => option.section)) {
			return options.map(({ section: _section, ...option }) => option)
		}

		const sections = new Map<string, MetadataFilterSection>()
		for (const option of options) {
			if (option.section) sections.set(option.section.key, option.section)
		}

		return [...sections.values()]
			.sort((a, b) => a.order - b.order)
			.flatMap((section, index) => [
				{
					type: 'section-header' as const,
					key: section.key,
					label: section.label,
					icon: section.icon,
					dividerBefore: index > 0,
				},
				...options
					.filter((option) => option.section?.key === section.key)
					.map(({ section: _section, ...option }) => option),
			])
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
			key: 'category',
			label: formatMessage(messages.category),
			searchable: true,
			values: (item) => {
				const section = getCategorySection(item)
				return [
					...(item.project?.categories ?? []),
					...(item.project?.additional_categories ?? []),
				].map((value) =>
					option(`${section.key}:${value}`, formatCategory(formatMessage, value), [value], section),
				)
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
			key: 'updates',
			label: formatMessage(messages.updates),
			values: (item) => [
				item.has_update
					? option('available', formatMessage(messages.updateAvailable))
					: option('current', formatMessage(messages.upToDate)),
			],
		},
		{
			key: 'warnings',
			label: formatMessage(messages.warnings),
			values: (item) => {
				const warning = getClientWarningType(item)
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
				const options = new Map<string, ResolvedFilterValue>()
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

				const visibleOptions = [...options.values()]
					.filter((option) => optionMatchCounts.get(option.value) !== items.value.length)
					.sort((a, b) => a.label.localeCompare(b.label, undefined, { numeric: true }))

				return {
					key: definition.key,
					label: definition.label,
					direct: definition.direct,
					searchable: definition.searchable,
					options: buildCategoryOptions(visibleOptions),
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
