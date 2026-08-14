import { useStorage } from '@vueuse/core'
import type { Ref } from 'vue'

import { type FilterType, type FilterValue, findFilterOption, flattenFilterOptions } from './search'

export const ADVANCED_PREFS_KEY = 'modrinth-advanced-exclusion-filters'

export function useAdvancedPrefs(): Ref<string[]> {
	return useStorage<string[]>(ADVANCED_PREFS_KEY, [])
}

export function getAdvancedOptionIds(filters: readonly FilterValue[]): string[] {
	return filters
		.filter((filter) => filter.type === 'advanced')
		.map((filter) => filter.option)
		.sort((a, b) => a.localeCompare(b))
}

export function sameOptionIds(a: readonly string[], b: readonly string[]): boolean {
	if (a.length !== b.length) {
		return false
	}
	const other = new Set(b)
	return a.every((id) => other.has(id))
}

export function getAvailableAdvancedIds(filterTypes: readonly FilterType[]): ReadonlySet<string> {
	const advancedType = filterTypes.find((filterType) => filterType.id === 'advanced')
	if (!advancedType) {
		return new Set()
	}
	return new Set(flattenFilterOptions(advancedType.options).map((option) => option.id))
}

export function compatibleAdvancedFilters(
	prefs: readonly string[],
	filterTypes: readonly FilterType[],
): FilterValue[] {
	const advancedType = filterTypes.find((filterType) => filterType.id === 'advanced')
	if (!advancedType) {
		return []
	}

	return prefs
		.filter((id) => findFilterOption(advancedType.options, id))
		.map((id) => ({
			type: 'advanced',
			option: id,
			negative: true,
		}))
}

export function replaceAdvancedFilters(
	filters: readonly FilterValue[],
	advancedFilters: readonly FilterValue[],
): FilterValue[] {
	return [...filters.filter((filter) => filter.type !== 'advanced'), ...advancedFilters]
}

export function mergeAdvancedPrefs(
	prefs: readonly string[],
	selected: readonly string[],
	available: ReadonlySet<string>,
): string[] {
	const selectedSet = new Set(selected)
	const next = prefs.filter((id) => !available.has(id) || selectedSet.has(id))

	for (const id of selectedSet) {
		if (!next.includes(id)) {
			next.push(id)
		}
	}

	return next.sort((a, b) => a.localeCompare(b))
}
