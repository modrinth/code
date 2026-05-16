import { useSessionStorage } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonProjectTypeCategoryMessages, normalizeProjectType } from '#ui/utils/common-messages'

import type { ClientWarningType, ContentItem } from '../types'

const CLIENT_ONLY_ENVIRONMENTS = new Set(['client_only', 'singleplayer_only'])

export function isClientOnlyEnvironment(env?: string | null): boolean {
	return !!env && CLIENT_ONLY_ENVIRONMENTS.has(env)
}

export function getClientWarningType(item: ContentItem): ClientWarningType | null {
	if (item.pack_client_retained) return 'retained'
	if (item.pack_client_depends) return 'depends'
	if (isClientOnlyEnvironment(item.environment)) return 'environment'
	return null
}

export interface ContentFilterOption {
	id: string
	label: string
}

export interface ContentFilterConfig {
	showTypeFilters?: boolean
	showUpdateFilter?: boolean
	showWarningsFilter?: boolean
	isPackLocked?: Ref<boolean>
	persistKey?: string
}

const messages = defineMessages({
	updates: {
		id: 'content.filter.updates',
		defaultMessage: 'Updates',
	},
	warnings: {
		id: 'content.filter.warnings',
		defaultMessage: 'Warnings',
	},
	enabled: {
		id: 'content.filter.enabled',
		defaultMessage: 'Enabled',
	},
	disabled: {
		id: 'content.filter.disabled',
		defaultMessage: 'Disabled',
	},
})

export function useContentFilters(items: Ref<ContentItem[]>, config?: ContentFilterConfig) {
	const { formatMessage } = useVIntl()

	const selectedFilters = config?.persistKey
		? useSessionStorage<string[]>(`content-filters:${config.persistKey}`, [])
		: ref<string[]>([])

	const availableStatusFilters = computed<Array<'enabled' | 'disabled'>>(() => {
		const hasEnabledContent = items.value.some((m) => m.enabled)
		const hasDisabledContent = items.value.some((m) => !m.enabled)

		return hasEnabledContent && hasDisabledContent ? ['enabled', 'disabled'] : []
	})

	const filterOptions = computed<ContentFilterOption[]>(() => {
		const options: ContentFilterOption[] = []

		if (config?.showTypeFilters) {
			const frequency = items.value.reduce((map: Record<string, number>, item) => {
				const normalized = normalizeProjectType(item.project_type)
				map[normalized] = (map[normalized] || 0) + 1
				return map
			}, {})
			const types = Object.keys(frequency).sort((a, b) => frequency[b] - frequency[a])
			for (const type of types) {
				const msg =
					commonProjectTypeCategoryMessages[type as keyof typeof commonProjectTypeCategoryMessages]
				const label = msg ? formatMessage(msg) : type.charAt(0).toUpperCase() + type.slice(1) + 's'
				options.push({ id: type, label })
			}
		}

		if (config?.showUpdateFilter && items.value.some((m) => m.has_update)) {
			options.push({ id: 'updates', label: formatMessage(messages.updates) })
		}

		if (config?.showWarningsFilter && items.value.some((m) => getClientWarningType(m) !== null)) {
			options.push({ id: 'warnings', label: formatMessage(messages.warnings) })
		}

		for (const status of availableStatusFilters.value) {
			options.push({
				id: status,
				label: formatMessage(status === 'enabled' ? messages.enabled : messages.disabled),
			})
		}

		return options
	})

	watch(
		filterOptions,
		() => {
			selectedFilters.value = selectedFilters.value.filter((f) =>
				filterOptions.value.some((opt) => opt.id === f),
			)
		},
		{ immediate: true },
	)

	function toggleFilter(filterId: string) {
		if (filterId === 'enabled' || filterId === 'disabled') {
			const index = selectedFilters.value.indexOf(filterId)
			const otherStatusFilter = filterId === 'enabled' ? 'disabled' : 'enabled'
			if (index === -1) {
				selectedFilters.value = [
					...selectedFilters.value.filter((filter) => filter !== otherStatusFilter),
					filterId,
				]
			} else {
				selectedFilters.value.splice(index, 1)
			}
			return
		}

		const index = selectedFilters.value.indexOf(filterId)
		if (index === -1) {
			selectedFilters.value.push(filterId)
		} else {
			selectedFilters.value.splice(index, 1)
		}
	}

	function applyFilters(source: ContentItem[]): ContentItem[] {
		if (selectedFilters.value.length === 0) return source

		const attributeFilters = new Set(['updates', 'enabled', 'disabled', 'warnings'])
		const typeFilters = selectedFilters.value.filter((f) => !attributeFilters.has(f))
		const activeAttributes = selectedFilters.value.filter((f) => attributeFilters.has(f))

		return source.filter((item) => {
			if (
				typeFilters.length > 0 &&
				!typeFilters.includes(normalizeProjectType(item.project_type))
			) {
				return false
			}

			for (const filter of activeAttributes) {
				if (filter === 'updates' && !item.has_update) return false
				if (filter === 'enabled' && !item.enabled) return false
				if (filter === 'disabled' && item.enabled) return false
				if (filter === 'warnings' && getClientWarningType(item) === null) return false
			}

			return true
		})
	}

	return { selectedFilters, filterOptions, toggleFilter, applyFilters }
}
