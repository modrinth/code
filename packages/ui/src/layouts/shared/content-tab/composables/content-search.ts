import Fuse from 'fuse.js'
import type { Ref } from 'vue'
import { ref, watchSyncEffect } from 'vue'

export function useContentSearch<T>(
	items: Ref<T[]>,
	keys: string[],
	options?: { threshold?: number; distance?: number },
) {
	const searchQuery = ref('')
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
