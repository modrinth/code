import { type Ref, ref, watch } from 'vue'

import type { FileZipTreeNode, LazyDataAccess } from '#ui/helpers/toolbox/zip'

export interface NodeLazyValue<T> {
	value: Ref<T | null>
	loading: Ref<boolean>
}

/**
 * A value computed by the zip worker for a file node (e.g. a hash), loaded whenever the node
 * changes. Results for a node that's no longer current are dropped.
 */
export function useNodeLazyValue<K extends keyof LazyDataAccess>(
	node: () => FileZipTreeNode | undefined,
	key: K,
): NodeLazyValue<Awaited<ReturnType<LazyDataAccess[K]['get']>>> {
	type Value = Awaited<ReturnType<LazyDataAccess[K]['get']>>
	const value = ref<Value | null>(null) as Ref<Value | null>
	const loading = ref(false)

	watch(
		node,
		async (current, _, onCleanup) => {
			let stale = false
			onCleanup(() => (stale = true))

			value.value = null
			if (!current) {
				loading.value = false
				return
			}

			loading.value = true
			const result = await (current[key].get() as Promise<Value>).catch(() => null)
			if (stale) return
			value.value = result
			loading.value = false
		},
		{ immediate: true },
	)

	return { value, loading }
}
