import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import type { ContentItem } from '../../components/instances/types'

const CLIENT_ONLY_ENVIRONMENTS = new Set([
	'client_only',
	'client_only_server_optional',
	'singleplayer_only',
])

export function isClientOnlyEnvironment(env?: string | null): boolean {
	return !!env && CLIENT_ONLY_ENVIRONMENTS.has(env)
}

export interface FilterOption {
	id: string
	label: string
}

export interface ContentFilterConfig {
	showTypeFilters?: boolean
	showUpdateFilter?: boolean
	showClientOnlyFilter?: boolean
	isPackLocked?: Ref<boolean>
	formatProjectType?: (type: string) => string
}

export function useContentFilters(items: Ref<ContentItem[]>, config?: ContentFilterConfig) {
	const selectedFilters = ref<string[]>([])

	const filterOptions = computed<FilterOption[]>(() => {
		const options: FilterOption[] = []

		if (config?.showTypeFilters) {
			const frequency = items.value.reduce((map: Record<string, number>, item) => {
				map[item.project_type] = (map[item.project_type] || 0) + 1
				return map
			}, {})
			const types = Object.keys(frequency).sort((a, b) => frequency[b] - frequency[a])
			for (const type of types) {
				const label = config.formatProjectType ? config.formatProjectType(type) + 's' : type + 's'
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
		return source.filter((item) => {
			for (const filter of selectedFilters.value) {
				if (filter === 'updates' && item.has_update) return true
				if (filter === 'disabled' && !item.enabled) return true
				if (filter === 'client-only' && isClientOnlyEnvironment(item.environment)) return true
				if (item.project_type === filter) return true
			}
			return false
		})
	}

	return { selectedFilters, filterOptions, toggleFilter, applyFilters }
}
