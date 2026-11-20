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
	if (options?.queryParam) syncRefToRouteQuery(searchQuery, options.queryParam, '')
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

export function syncRefToRouteQuery<T extends string>(
	value: Ref<T>,
	param: string,
	defaultValue: T,
	isValid: (candidate: string) => candidate is T = (candidate): candidate is T => true,
) {
	const route = useRoute()
	const router = useRouter()
	const path = route.path
	const initial = route.query[param]
	if (typeof initial === 'string' && isValid(initial)) value.value = initial

	watch(value, (next) => {
		if (route.path !== path) return
		router.replace({
			query: { ...route.query, [param]: next === defaultValue ? undefined : next },
		})
	})
}
