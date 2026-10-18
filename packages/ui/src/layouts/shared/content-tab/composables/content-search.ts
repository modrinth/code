import Fuse from 'fuse.js'
import type { Ref } from 'vue'
import { ref, watch, watchSyncEffect } from 'vue'
import { useRoute, useRouter } from 'vue-router'

export function useContentSearch<T>(
	items: Ref<T[]>,
	keys: string[],
	options?: { threshold?: number; distance?: number; queryParam?: string },
) {
	const searchQuery = ref('')
	if (options?.queryParam) syncSearchQueryToRoute(searchQuery, options.queryParam)
	const fuse = new Fuse<T>([], {
		keys,
		threshold: options?.threshold ?? 0.4,
		distance: options?.distance ?? 100,
	})
	watchSyncEffect(() => fuse.setCollection(items.value))

	function search(source: T[]): T[] {
		const query = searchQuery.value.trim()
		if (!query) return source
		return fuse.search(query).map(({ item }) => item)
	}

	return { searchQuery, search }
}

/**
 * Keeps the search in the route query so back navigation restores it, while fresh visits to the
 * page start with an empty search.
 */
function syncSearchQueryToRoute(searchQuery: Ref<string>, param: string) {
	const route = useRoute()
	const router = useRouter()
	const path = route.path
	const initial = route.query[param]
	if (typeof initial === 'string') searchQuery.value = initial

	watch(searchQuery, (value) => {
		if (route.path !== path) return
		router.replace({ query: { ...route.query, [param]: value || undefined } })
	})
}
