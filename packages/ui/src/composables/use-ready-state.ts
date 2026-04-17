import type { Ref } from 'vue'
import { computed } from 'vue'

/**
 * Minimal shape of a TanStack `useQuery` return value that `useReadyState` reads.
 * Kept structural so consumers can pass either a real `UseQueryReturnType` or a
 * computed wrapper without an extra dependency on `@tanstack/vue-query` types.
 */
export interface ReadyStateQueryLike<TData> {
	isLoading: Readonly<Ref<boolean>>
	data: Readonly<Ref<TData | undefined>>
}

/**
 * Returns true while a query is loading for the FIRST time (no cached data yet).
 *
 * Excludes background refetches and refetch-on-window-focus by design — those
 * have `isLoading === false` once data exists in the cache, so `ReadyTransition`
 * stays open and the loading bar stays silent.
 *
 * Pair with `<ReadyTransition :pending="useReadyState(query)" />`.
 */
export function useReadyState<TData>(query: ReadyStateQueryLike<TData>): Readonly<Ref<boolean>> {
	return computed(() => query.isLoading.value && query.data.value === undefined)
}
