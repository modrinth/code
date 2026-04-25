import type { DefaultError, UseQueryReturnType } from '@tanstack/vue-query'
import type { Ref } from 'vue'
import { computed } from 'vue'

/** Subset of {@link UseQueryReturnType} passed to {@link useReadyState}. */
export type ReadyStateQuery<TData, TError = DefaultError> = Pick<
	UseQueryReturnType<TData, TError>,
	'isLoading' | 'data'
>

/**
 * Returns true while a query is loading for the FIRST time (no cached data yet).
 *
 * Excludes background refetches and refetch-on-window-focus by design — those
 * have `isLoading === false` once data exists in the cache, so `ReadyTransition`
 * stays open and the loading bar stays silent.
 *
 * Pair with `<ReadyTransition :pending="var which is useReadyState(query)" />`.
 */
export function useReadyState<TData, TError = DefaultError>(
	query: ReadyStateQuery<TData, TError>,
): Readonly<Ref<boolean>> {
	return computed(() => query.isLoading.value && query.data.value === undefined)
}
