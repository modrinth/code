import { useSessionStorage } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import {
	commonProjectTypeCategoryMessages,
	normalizeProjectType,
} from '#ui/utils/common-messages'

import type { ContentItem } from '../types'

const CLIENT_ONLY_ENVIRONMENTS = new Set(['client_only', 'singleplayer_only'])

export function isClientOnlyEnvironment(env?: string | null): boolean {
	return !!env && CLIENT_ONLY_ENVIRONMENTS.has(env)
}

export interface ContentFilterOption {
	id: string
	label: string
}

export interface ContentFilterConfig {
	showTypeFilters?: boolean
	showUpdateFilter?: boolean
	showClientOnlyFilter?: boolean
	isPackLocked?: Ref<boolean>
	persistKey?: string
}

export function useContentFilters(items: Ref<ContentItem[]>, config?: ContentFilterConfig) {
	const { formatMessage } = useVIntl()

	const selectedFilters = config?.persistKey
		? useSessionStorage<string[]>(`content-filters:${config.persistKey}`, [])
		: ref<string[]>([])

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
					commonProjectTypeCategoryMessages[
						type as keyof typeof commonProjectTypeCategoryMessages
					]
				const label = msg ? formatMessage(msg) : type.charAt(0).toUpperCase() + type.slice(1) + 's'
				options.push({ id: type, label })
			}
		}

		if (
			config?.showUpdateFilter &&
			!config?.isPackLocked?.value &&
			items.value.some((m) => m.has_update)
		) {
			options.push({ id: 'updates', label: 'Updates' })
		}

		if (
			config?.showClientOnlyFilter &&
			items.value.some((m) => isClientOnlyEnvironment(m.environment))
		) {
			options.push({ id: 'client-only', label: 'Client-only' })
		}

		if (items.value.some((m) => !m.enabled)) {
			options.push({ id: 'disabled', label: 'Disabled' })
		}

		return options
	})

	watch(filterOptions, () => {
		selectedFilters.value = selectedFilters.value.filter((f) =>
			filterOptions.value.some((opt) => opt.id === f),
		)
	})

	function toggleFilter(filterId: string) {
		const index = selectedFilters.value.indexOf(filterId)
		if (index === -1) {
			selectedFilters.value.push(filterId)
		} else {
			selectedFilters.value.splice(index, 1)
		}
	}

	function applyFilters(source: ContentItem[]): ContentItem[] {
		if (selectedFilters.value.length === 0) return source

		const attributeFilters = new Set(['updates', 'disabled', 'client-only'])
		const typeFilters = selectedFilters.value.filter((f) => !attributeFilters.has(f))
		const activeAttributes = selectedFilters.value.filter((f) => attributeFilters.has(f))

		return source.filter((item) => {
			if (typeFilters.length > 0 && !typeFilters.includes(normalizeProjectType(item.project_type))) {
				return false
			}

			for (const filter of activeAttributes) {
				if (filter === 'updates' && !item.has_update) return false
				if (filter === 'disabled' && item.enabled) return false
				if (filter === 'client-only' && !isClientOnlyEnvironment(item.environment)) return false
			}

			return true
		})
	}

	return { selectedFilters, filterOptions, toggleFilter, applyFilters }
}
