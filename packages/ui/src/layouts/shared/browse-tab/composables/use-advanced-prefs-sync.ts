import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import {
	compatibleAdvancedFilters,
	getAdvancedOptionIds,
	getAvailableAdvancedIds,
	mergeAdvancedPrefs,
	replaceAdvancedFilters,
	sameOptionIds,
	useAdvancedPrefs,
} from '#ui/utils/advanced-filter-preferences'
import type { FilterType, FilterValue } from '#ui/utils/search'

export interface AdvancedPrefsSyncOptions {
	filters: Ref<FilterValue[]>
	filterTypes: Readonly<Ref<FilterType[]>>
	hasLinkQuery: () => boolean
}

export function useAdvancedPrefsSync({
	filters,
	filterTypes,
	hasLinkQuery,
}: AdvancedPrefsSyncOptions) {
	const advancedPrefs = useAdvancedPrefs()
	const linkOverridesAdvancedPrefs = ref(false)
	const selectedAdvancedIds = computed(() => getAdvancedOptionIds(filters.value))

	function compatiblePrefs(types: readonly FilterType[] = filterTypes.value) {
		return compatibleAdvancedFilters(advancedPrefs.value, types)
	}

	function applyPrefs(prefs: FilterValue[]) {
		filters.value = replaceAdvancedFilters(filters.value, prefs)
	}

	function withSavedPrefs(current: FilterValue[], types: readonly FilterType[]) {
		const prefs = compatiblePrefs(types)
		return prefs.length > 0 ? replaceAdvancedFilters(current, prefs) : current
	}

	function init() {
		const prefs = compatiblePrefs()

		if (!hasLinkQuery()) {
			if (prefs.length > 0) applyPrefs(prefs)
			linkOverridesAdvancedPrefs.value = false
			return
		}

		const prefIds = getAdvancedOptionIds(prefs)
		linkOverridesAdvancedPrefs.value =
			prefIds.length > 0 && !sameOptionIds(selectedAdvancedIds.value, prefIds)
	}

	function applySavedAdvancedPrefs() {
		applyPrefs(compatiblePrefs())
		linkOverridesAdvancedPrefs.value = false
	}

	watch(selectedAdvancedIds, (selected, previous) => {
		if (previous && sameOptionIds(selected, previous)) return

		const nextPrefs = mergeAdvancedPrefs(
			advancedPrefs.value,
			selected,
			getAvailableAdvancedIds(filterTypes.value),
		)
		if (!sameOptionIds(advancedPrefs.value, nextPrefs)) {
			advancedPrefs.value = nextPrefs
		}
		linkOverridesAdvancedPrefs.value = false
	})

	init()

	return { linkOverridesAdvancedPrefs, applySavedAdvancedPrefs, withSavedPrefs, init }
}
